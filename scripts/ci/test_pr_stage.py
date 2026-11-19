"""Negative controls for the development-only PR classifier."""
import importlib.util
import unittest
from pathlib import Path

SOURCE = Path(__file__).with_name("pr_stage.py")
spec = importlib.util.spec_from_file_location("pr_stage", SOURCE)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
REPO = "afflom/UORust"
HEAD = "a" * 40


def event(draft=True, action="synchronize"):
    return {
        "action": action,
        "repository": {"full_name": REPO},
        "pull_request": {
            "state": "open", "draft": draft,
            "base": {"ref": "main"},
            "head": {"sha": HEAD, "repo": {"full_name": "contributor/UORust"}},
            "body": "- [x] I am ready",
            "labels": [{"name": "merge-approved"}],
        },
    }


class StageTest(unittest.TestCase):
    def test_draft_with_forged_body_or_labels_is_development_only(self):
        result = mod.classify(event(), REPO)
        self.assertEqual(result["stage"], "development")
        self.assertFalse(result["merge_qualified"])

    def test_ready_pr_is_candidate_not_native_acceptance(self):
        result = mod.classify(event(False, "ready_for_review"), REPO)
        self.assertEqual(result["stage"], "candidate")
        self.assertEqual(result["head_sha"], HEAD)
        self.assertFalse(result["merge_qualified"])

    def test_missing_or_wrong_inputs_fail_closed(self):
        changes = (
            lambda e: e["pull_request"].pop("draft"),
            lambda e: e["pull_request"].__setitem__("draft", "false"),
            lambda e: e["pull_request"]["base"].__setitem__("ref", "release"),
            lambda e: e["pull_request"]["head"].__setitem__("sha", "b" * 12),
            lambda e: e["repository"].__setitem__("full_name", "other/repo"),
            lambda e: e.__setitem__("action", "closed"),
            lambda e: e["pull_request"].__setitem__("state", "closed"),
            lambda e: e.__setitem__("action", "ready_for_review"),
        )
        for change in changes:
            obj = event()
            change(obj)
            with self.subTest(change=change), self.assertRaises(mod.AdmissionError):
                mod.classify(obj, REPO)

    def test_draft_transition_must_match_event(self):
        self.assertEqual(mod.classify(event(True, "converted_to_draft"), REPO)["stage"], "development")
        with self.assertRaises(mod.AdmissionError):
            mod.classify(event(False, "converted_to_draft"), REPO)

    def test_workflow_is_non_skipping_and_non_authorizing(self):
        workflow = (SOURCE.parents[2] / ".github/workflows/pr-stage.yml").read_text()
        self.assertIn("  pull_request:", workflow)
        self.assertNotIn("    paths:", workflow)
        self.assertNotIn("    paths-ignore:", workflow)
        self.assertNotIn("    if:", workflow)
        self.assertIn("fetch-depth: 0", workflow)
        self.assertIn("persist-credentials: false", workflow)
        self.assertIn("git diff --check HEAD^1 HEAD", workflow)
        self.assertIn("NOT MERGE ACCEPTANCE", workflow)
        self.assertIn("scripts/ci/pr_stage.py", workflow)


if __name__ == "__main__":
    unittest.main()
