"""Execute both checked-in Pages identity gates with positive and planted negative inputs.

This tests each entire workflow step, including shell, run and conditional
configuration, plus its actual Node body. A missing, skipped or ignored guard
therefore cannot silently pass.
No network access, Docker image or GitHub token is required.
"""
import json
import os
import re
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
STEP_NAME = "      - name: Refuse stale Pages repository identity before costly build work\n"
STEP = STEP_NAME + (
    "        shell: bash\n"
    "        run: |\n"
    "          node -e '\n"
)
END = "          '\n"


def workflow_guard(path, source=None):
    """Reject missing, conditional, ignored or malformed identity steps.

    Inspect the entire YAML step before extracting its Node body. The exact
    shape rejects extra keys such as if or continue-on-error after the script.
    """
    if source is None:
        source = path.read_text(encoding="utf-8")
    if source.count(STEP_NAME) != 1:
        raise AssertionError(f"{path}: missing or duplicate identity check")
    after = source.split(STEP_NAME, 1)[1]
    next_step = re.search(r"(?m)^      - name: ", after)
    if not next_step:
        raise AssertionError(f"{path}: identity guard is not followed by a step")
    full_step = STEP_NAME + after[:next_step.start()]
    if not full_step.startswith(STEP):
        raise AssertionError(f"{path}: identity guard has unexpected step configuration")
    body_plus_end = full_step[len(STEP):]
    if not body_plus_end.endswith(END) or body_plus_end.count(END) != 1:
        raise AssertionError(f"{path}: identity guard has trailing configuration or malformed Node script")
    body = body_plus_end[:-len(END)]
    if not body.startswith("            const ") or any(
        not line.startswith("            ") for line in body.splitlines()
    ):
        raise AssertionError(f"{path}: identity guard has non-script or malformed lines")
    return textwrap.dedent(body)

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

    def test_planted_conditional_or_ignored_step_is_rejected(self):
        # Plant bypasses after the closing Node quote, which checks inspecting
        # only the Node script would miss.
        for path in WORKFLOWS:
            source = path.read_text(encoding="utf-8")
            start = source.index(STEP_NAME)
            end = source.index(END, start) + len(END)
            for bypass in ("        if: false\n", "        continue-on-error: true\n"):
                with self.subTest(workflow=str(path), bypass=bypass.strip()):
                    planted = source[:end] + bypass + source[end:]
                    with self.assertRaisesRegex(AssertionError, "trailing configuration"):
                        workflow_guard(path, planted)

    def test_planted_pre_run_condition_is_rejected(self):
        for path in WORKFLOWS:
            source = path.read_text(encoding="utf-8")
            planted = source.replace(
                STEP_NAME, STEP_NAME + "        if: false\n", 1
            )
            with self.subTest(workflow=str(path)):
                with self.assertRaisesRegex(AssertionError, "step configuration"):
                    workflow_guard(path, planted)

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


# An ordinary unittest command can report success even when all tests are
# skipped. Full acceptance requires the exact test identities AND execution.
REQUIRED_PREFLIGHT = (
    "test_both_workflows_have_identical_non_vacuous_guard",
    "test_missing_and_invalid_repository_are_refused",
    "test_planted_conditional_or_ignored_step_is_rejected",
    "test_planted_pre_run_condition_is_rejected",
    "test_planted_pre_transfer_repository_is_refused_by_both",
    "test_restored_correct_repository_passes_in_both_workflows",
)
REQUIRED_RESULT_GATE = (
    "test_complete_success_is_accepted",
    "test_missing_failed_or_skipped_controls_are_refused",
    "test_workflows_run_strict_gate_before_identity_preflight",
)
REQUIRED_NAMES = REQUIRED_PREFLIGHT + REQUIRED_RESULT_GATE


def complete_result(result, actual_names):
    return (
        tuple(actual_names) == REQUIRED_NAMES
        and result.testsRun == len(REQUIRED_NAMES)
        and result.wasSuccessful()
        and not result.skipped
        and not result.expectedFailures
        and not result.unexpectedSuccesses
    )


class PagesIdentityResultGateTests(unittest.TestCase):
    def test_complete_success_is_accepted(self):
        clean = unittest.TestResult()
        clean.testsRun = len(REQUIRED_NAMES)
        self.assertTrue(complete_result(clean, REQUIRED_NAMES))

    def test_missing_failed_or_skipped_controls_are_refused(self):
        clean = unittest.TestResult()
        clean.testsRun = len(REQUIRED_NAMES)
        self.assertFalse(complete_result(clean, REQUIRED_NAMES[:-1]))
        clean.testsRun -= 1
        self.assertFalse(complete_result(clean, REQUIRED_NAMES))
        clean.testsRun += 1
        clean.skipped.append(("PagesIdentityPreflightTests", "seeded skip"))
        self.assertFalse(complete_result(clean, REQUIRED_NAMES))
        clean.skipped.clear()
        clean.failures.append(("PagesIdentityPreflightTests", "seeded failure"))
        self.assertFalse(complete_result(clean, REQUIRED_NAMES))
        clean.failures.clear()
        clean.expectedFailures.append(("PagesIdentityPreflightTests", "seeded expected failure"))
        self.assertFalse(complete_result(clean, REQUIRED_NAMES))

    def test_workflows_run_strict_gate_before_identity_preflight(self):
        exact = (
            "      - name: Verify Pages identity gates against planted wrong metadata\n"
            "        run: python3 -I scripts/ci/test_pages_identity.py\n"
            "      - name: Refuse stale Pages repository identity before costly build work\n"
        )
        for path in WORKFLOWS:
            with self.subTest(workflow=str(path)):
                source = path.read_text(encoding="utf-8")
                self.assertEqual(source.count(exact), 1)
                self.assertEqual(source.count("      - name: Verify Pages identity gates"), 1)


def main():
    test_suites = [
        unittest.defaultTestLoader.loadTestsFromTestCase(PagesIdentityPreflightTests),
        unittest.defaultTestLoader.loadTestsFromTestCase(PagesIdentityResultGateTests),
    ]
    actual_names = [test._testMethodName for suite in test_suites for test in suite]
    result = unittest.TextTestRunner(verbosity=2).run(unittest.TestSuite(test_suites))
    if not complete_result(result, actual_names):
        import sys
        print(
            "PAGES IDENTITY GATE REFUSED: expected nine named, passing, non-skipped tests",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
