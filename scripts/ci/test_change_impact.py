"""Planted-fault tests for the advisory, trusted-base PR impact classifier."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

SOURCE = Path(__file__).with_name("change_impact.py")
spec = importlib.util.spec_from_file_location("change_impact", SOURCE)
impact = importlib.util.module_from_spec(spec)
spec.loader.exec_module(impact)

GRAPH = '''[sets]
sdk = ["prismpm.lock"]
model = ["README.md", "model/", "site/app.json"]

[[obligation]]
name = "vv"
sets = ["sdk"]
inputs = ["Justfile"]

[[obligation]]
name = "self-model-check"
sets = ["model"]
inputs = []
'''


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()


def commit(repo, name, content):
    target = repo / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")
    git(repo, "add", "-A")
    git(repo, "commit", "-qm", "change")
    return git(repo, "rev-parse", "HEAD")


class ChangeImpactTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        git(self.repo, "init", "-q")
        git(self.repo, "config", "user.name", "Verifier")
        git(self.repo, "config", "user.email", "verified@example.invalid")
        self.base = commit(self.repo, "prism-factory/graph.toml", GRAPH)
        self.deps = impact.dependency_inputs(self.repo, self.base)

    def test_passive_docs_only_when_not_graph_inputs(self):
        for path in ("docs/guides/intro.md", "docs/tutorials/setup.rst", "docs/examples/demo.md"):
            with self.subTest(path=path):
                self.assertEqual(impact.classify([path], self.deps)["route"], "docs")
        for path in ("README.md", "docs/README.md", "SECURITY.md", "PAGES-VERIFICATION.md"):
            with self.subTest(path=path):
                self.assertNotEqual(impact.classify([path], self.deps)["route"], "docs")
        self.assertIn("model-proof", impact.classify(["README.md"], self.deps)["selected_development_suites"])

    def test_high_impact_and_mixed_changes_preserve_coverage(self):
        cases = {
            "site/app.json": {"browser", "model-proof"},
            "crates/uorust/src/lib.rs": {"rust"},
            "app/src/Features.lex.tex": {"model-proof"},
            ".github/workflows/bootstrap.yml": {"trust"},
            "Cargo.lock": {"rust", "trust"},
            "prism-factory/graph.toml": {"model-proof"},
            "scripts/ci/change_impact.py": {"trust"},
        }
        for path, required in cases.items():
            with self.subTest(path=path):
                self.assertTrue(required.issubset(set(impact.classify([path], self.deps)["selected_development_suites"])))
        result = impact.classify(["docs/guides/use.md", "crates/uorust/src/lib.rs"], self.deps)
        self.assertEqual(set(result["selected_development_suites"]), {"docs", "rust"})
        self.assertTrue(result["native_acceptance_required"])
        self.assertFalse(result["merge_qualified"])

    def test_unknown_missing_and_unsafe_paths_fail_closed(self):
        full = set(impact.CATEGORIES)
        for paths in ([], ["unknown.dat"], ["../README.md"], ["__UNVERIFIED_DIFF__"], ["bad\\name.md"]):
            with self.subTest(paths=paths):
                self.assertEqual(set(impact.classify(paths, self.deps)["selected_development_suites"]), full)

    def test_exact_git_refs_and_renames(self):
        head = commit(self.repo, "docs/guides/example.md", "documentation\n")
        result = impact.report(self.repo, self.base, head)
        self.assertEqual(result["route"], "docs")
        self.assertFalse(result["merge_qualified"])
        (self.repo / "docs/guides/example.md").rename(self.repo / "runtime.rs")
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-qm", "rename")
        renamed_head = git(self.repo, "rev-parse", "HEAD")
        self.assertIn("runtime.rs", impact.compare(self.repo, head, renamed_head))
        self.assertNotEqual(impact.report(self.repo, head, renamed_head)["route"], "docs")
        self.assertEqual(impact.report(self.repo, "main", renamed_head)["route"], "full-unknown")

    def test_unsafe_modes_and_wrong_dependency_map_fail_closed(self):
        doc = self.repo / "docs/guides/link.md"
        doc.parent.mkdir(parents=True, exist_ok=True)
        doc.symlink_to("../unknown.md")
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-qm", "link")
        sha = git(self.repo, "rev-parse", "HEAD")
        paths = impact.compare(self.repo, self.base, sha)
        self.assertIn("__NONREGULAR_FILE_OR_MODE__", paths)
        self.assertEqual(impact.report(self.repo, self.base, sha)["route"], "full-unknown")
        self.assertEqual(impact.report(self.repo, sha, sha)["route"], "full-unknown")

    def test_ordered_factory_exclusion_and_later_reinclusion(self):
        graph = {
            "test": [
                "crates/", "!crates/uorust/tests/fixtures/",
                "crates/uorust/tests/fixtures/nist-sha256/",
            ],
        }
        self.assertEqual(
            impact.consumers("crates/uorust/tests/fixtures/other/lib.rs", graph), [])
        self.assertEqual(
            impact.consumers("crates/uorust/tests/fixtures/nist-sha256/vector.rs", graph),
            ["test"])
        self.assertEqual(
            impact.consumers("crates/uorust/src/lib.rs", graph), ["test"])

    def test_graph_consumed_docs_are_not_downgraded(self):
        graph = GRAPH.replace("inputs = []", 'inputs = ["docs/guides/"]')
        baseline = commit(self.repo, "prism-factory/graph.toml", graph)
        head = commit(self.repo, "docs/guides/use.md", "Text\n")
        result = impact.report(self.repo, baseline, head)
        self.assertIn("model-proof", result["selected_development_suites"])
        self.assertNotEqual(result["route"], "docs")


if __name__ == "__main__":
    unittest.main()
