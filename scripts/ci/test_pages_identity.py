"""Execute both checked-in Pages identity gates with positive and planted negative inputs.

This tests the actual Node bodies embedded in each workflow, not a copy of
their logic. A missing/changed/bypassed step therefore cannot silently pass.
No network access, Docker image or GitHub token is required.
"""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = (
    ROOT / ".github/workflows/pages.yml",
    ROOT / ".github/workflows/try-page.yml",
)
ACTUAL_REPO = "UOR-Foundation/UORust"
CANONICAL = "https://github.com/" + ACTUAL_REPO
STALE = "https://github.com/afflom/uorust"
STEP = (
    "      - name: Refuse stale Pages repository identity before costly build work\n"
    "        shell: bash\n"
    "        run: |\n"
    "          node -e '\n"
)
END = "\n          '\n"


def workflow_guard(path):
    """Extract the exact workflow preflight script, with a fail-closed shape."""
    source = path.read_text(encoding="utf-8")
    if source.count(STEP) != 1:
        raise AssertionError(f"{path}: missing or duplicate identity check")
    after = source.split(STEP, 1)[1]
    body, terminator, _ = after.partition(END)
    if not terminator or not body.startswith("            const "):
        raise AssertionError(f"{path}: identity check is not a Node script")
    return textwrap.dedent(body) + "\n"


def run_guard(script, value, repository=ACTUAL_REPO):
    with tempfile.TemporaryDirectory() as tmp:
        site = Path(tmp) / "site"
        site.mkdir()
        (site / "app.json").write_text(
            json.dumps({} if value is None else {"repository": value}),
            encoding="utf-8",
        )
        env = {**os.environ, "GITHUB_REPOSITORY": repository}
        return subprocess.run(
            ["node", "-e", script],
            cwd=tmp, env=env, text=True, capture_output=True,
            timeout=15, check=False,
        )


class PagesIdentityPreflightTests(unittest.TestCase):
    def test_both_workflows_have_identical_non_vacuous_guard(self):
        guards = [workflow_guard(path) for path in WORKFLOWS]
        self.assertEqual(guards[0], guards[1])

    def test_restored_correct_repository_passes_in_both_workflows(self):
        for path in WORKFLOWS:
            with self.subTest(workflow=str(path)):
                result = run_guard(workflow_guard(path), CANONICAL)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_planted_pre_transfer_repository_is_refused_by_both(self):
        for path in WORKFLOWS:
            with self.subTest(workflow=str(path)):
                result = run_guard(workflow_guard(path), STALE)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"site/app.json repository mismatch: {STALE} != {CANONICAL}",
                    result.stderr,
                )

    def test_missing_and_invalid_repository_are_refused(self):
        for path in WORKFLOWS:
            guard = workflow_guard(path)
            for case in (None, 123, ""):
                with self.subTest(workflow=str(path), metadata=case):
                    result = run_guard(guard, case)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("site/app.json repository mismatch", result.stderr)


if __name__ == "__main__":
    unittest.main()
