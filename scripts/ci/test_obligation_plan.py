"""CI-02: planted bad plans and a trusted dependency-graph fixture."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

SOURCE = Path(__file__).with_name("change_impact.py")
spec = importlib.util.spec_from_file_location("change_impact", SOURCE)
impact = importlib.util.module_from_spec(spec)
spec.loader.exec_module(impact)

PARTS = impact.VV_PARTS
JUST = "vv: " + " ".join(PARTS) + "\n"
GRAPH = {
    "sets": {
        "rust": ["crates/"],
        "model": ["README.md", "model/", "site/app.json"],
        "sdk": ["prismpm.lock"],
    },
    "obligation": [
        {"name": "vv", "needs": list(PARTS)},
        *[
            {"name": name, "recipe": name, "sets":
             ["rust"] if name in ("fmt-check", "lint", "test", "uorust") else
             ["model"] if name in ("model", "self-model-check", "app") else
             ["sdk"], "needs": ["model"] if name == "app" else []}
            for name in PARTS
        ],
    ],
}


class ObligationPlanCi02Tests(unittest.TestCase):
    def plan(self, paths, graph=GRAPH, just=JUST):
        return impact.obligation_plan(paths, graph, just)

    def test_readme_is_not_passive_documentation_ci_02(self):
        p = self.plan(["README.md"])
        self.assertEqual(p["mode"], "selective")
        self.assertEqual(set(p["required_obligations"]), {"model", "self-model-check", "app"})
        self.assertTrue(p["native_acceptance_required"])
        self.assertFalse(p["merge_qualified"])

    def test_actual_passive_documentation_avoids_sdk_ci_02(self):
        p = self.plan(["docs/guides/onboarding.md"])
        self.assertEqual(p["required_obligations"], [])
        self.assertEqual(p["omitted_obligations"], list(PARTS))
        self.assertFalse(p["merge_qualified"])

    def test_rust_closure_and_mixed_changes_ci_02(self):
        p = self.plan(["crates/uorust/src/lib.rs", "site/app.json"])
        self.assertEqual(
            set(p["required_obligations"]),
            {"fmt-check", "lint", "test", "uorust", "model", "self-model-check", "app"},
        )
        self.assertIn("template-check", p["omitted_obligations"])

    def test_browser_only_requires_existing_browser_acceptance_ci_02(self):
        p = self.plan(["site/app.mjs"])
        self.assertEqual(p["required_obligations"], [])
        self.assertEqual(p["reason"], "trusted-factory-input-closure")

    def test_policy_unsafe_unknown_and_empty_are_full_ci_02(self):
        for changed in (
            [], ["../docs/escape.md"], ["__NONREGULAR_FILE_OR_MODE__"],
            ["Justfile"], ["prismpm.lock"], ["Cargo.lock"],
            ["prism-factory/graph.toml"], [".github/workflows/bootstrap.yml"],
            ["scripts/ci/change_impact.py"], ["untracked/random.unknown"],
        ):
            with self.subTest(paths=changed):
                p = self.plan(changed)
                self.assertEqual(p["mode"], "full")
                self.assertEqual(p["required_obligations"], list(PARTS))
                self.assertFalse(p["merge_qualified"])

    def test_missing_graph_or_changed_justfile_cannot_select_ci_02(self):
        bad = [
            ({}, JUST),
            ({"sets": GRAPH["sets"], "obligation": []}, JUST),
            (GRAPH, "vv: app\n"),
            (GRAPH, JUST + "vv: app\n"),
            (dict(GRAPH, obligation=GRAPH["obligation"][:-1]), JUST),
            (dict(GRAPH, obligation=GRAPH["obligation"] * 2), JUST),
        ]
        for graph, just in bad:
            with self.subTest(graph=graph, just=just):
                with self.assertRaises(impact.UnverifiedDiff):
                    self.plan(["docs/guides/read.md"], graph, just)

    def test_malformed_dependencies_are_not_trusted_ci_02(self):
        for bad in (["ghost"], ["vv"]):
            nodes = [dict(x) for x in GRAPH["obligation"]]
            nodes[1]["needs"] = bad
            if bad == ["vv"]:
                # VV's recipe does not qualify as a runnable component.
                pass
            with self.assertRaises(impact.UnverifiedDiff):
                self.plan(["crates/uorust/src/lib.rs"],
                          dict(GRAPH, obligation=nodes))

    def test_actual_git_comparison_binds_exact_commits_ci_02(self):
        def git(repo, *args):
            return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            git(repo, "init", "-q")
            git(repo, "config", "user.name", "CI Plan")
            git(repo, "config", "user.email", "ci@example.invalid")
            graph = repo / "prism-factory/graph.toml"
            graph.parent.mkdir(parents=True)
            # Construct just enough actual TOML to represent the trusted graph.
            graph.write_text(
                '[sets]\nrust = ["crates/"]\nmodel = ["README.md", "model/", "site/app.json"]\n'
                'sdk = ["prismpm.lock"]\n'
                '[[obligation]]\nname = "vv"\nneeds = [' +
                ','.join('"' + name + '"' for name in PARTS) + ']\n' +
                ''.join('[[obligation]]\nname = "' + name + '"\nrecipe = "' + name +
                        '"\nsets = ["' +
                        ('rust' if name in ("fmt-check", "lint", "test", "uorust") else
                         'model' if name in ("model", "self-model-check", "app") else 'sdk')
                        + '"]\n' for name in PARTS),
                encoding="utf-8",
            )
            (repo / "Justfile").write_text(JUST, encoding="utf-8")
            git(repo, "add", "-A")
            git(repo, "commit", "-qm", "trusted base")
            base = git(repo, "rev-parse", "HEAD")
            changed = repo / "docs/guides/read.md"
            changed.parent.mkdir(parents=True)
            changed.write_text("Hello\n")
            git(repo, "add", "-A")
            git(repo, "commit", "-qm", "docs")
            head = git(repo, "rev-parse", "HEAD")
            p = impact.report_obligation_plan(repo, base, head)
            self.assertEqual(p["mode"], "selective")
            self.assertEqual(p["required_obligations"], [])
            self.assertEqual((p["base_sha"], p["head_sha"]), (base, head))
            self.assertEqual(
                impact.report_obligation_plan(repo, "main", head)["required_obligations"],
                list(PARTS),
            )


    def test_execution_receipt_runs_selected_checks_or_full_ci_02(self):
        selected = self.plan(["README.md"])
        full = self.plan(["scripts/ci/change_impact.py"])
        for plan, expected in (
            (selected, [["just", "model"], ["just", "self-model-check"], ["just", "app"]]),
            (full, [["just", "vv"]]),
        ):
            plan = dict(plan, base_sha="a" * 40, head_sha="b" * 40)
            invoked = []
            def run(command, **kwargs):
                invoked.append(command)
                return subprocess.CompletedProcess(command, 0)
            with mock.patch.object(impact, "git", return_value=(b"b" * 40 + b"\n")), \
                 mock.patch.object(impact.subprocess, "run", side_effect=run):
                receipt = impact.execute_obligation_plan(Path("."), plan)
            self.assertEqual(invoked, expected)
            self.assertTrue(receipt["components_passed"])
            self.assertFalse(receipt["merge_qualified"])
            self.assertEqual([x["command"] for x in receipt["observed"]], expected)

    def test_failed_selected_component_never_yields_success_ci_02(self):
        plan = dict(self.plan(["README.md"]), base_sha="a" * 40, head_sha="b" * 40)
        with mock.patch.object(impact, "git", return_value=(b"b" * 40 + b"\n")), \
             mock.patch.object(impact.subprocess, "run",
                               side_effect=lambda command, **kwargs:
                               subprocess.CompletedProcess(command, 7)):
            receipt = impact.execute_obligation_plan(Path("."), plan)
        self.assertFalse(receipt["components_passed"])
        self.assertEqual(len(receipt["observed"]), 1)
        self.assertEqual(receipt["observed"][0]["exit_code"], 7)

    def test_stale_head_or_forged_receipt_is_refused_ci_02(self):
        plan = dict(self.plan(["README.md"]), base_sha="a" * 40, head_sha="b" * 40)
        malformed = [
            dict(plan, required_obligations=["app", "model", "self-model-check"]),
            dict(plan, required_obligations=["model", "model"]),
            dict(plan, omitted_obligations=[]),
            dict(plan, mode="full"),
            dict(plan, merge_qualified=True),
            dict(plan, head_sha="c" * 40),
        ]
        for candidate in malformed:
            with self.subTest(candidate=candidate), \
                 mock.patch.object(impact, "git", return_value=(b"b" * 40 + b"\n")), \
                 self.assertRaises(impact.UnverifiedDiff):
                impact.execute_obligation_plan(Path("."), candidate)


if __name__ == "__main__":
    unittest.main()
