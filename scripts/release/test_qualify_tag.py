"""Negative exact-tag acceptance receipt fixtures; no network, no publishing."""
import importlib.util
import unittest
from copy import deepcopy
from pathlib import Path

spec = importlib.util.spec_from_file_location("qualify_tag", Path(__file__).with_name("qualify_tag.py"))
q = importlib.util.module_from_spec(spec)
spec.loader.exec_module(q)
REPO, TAG, SHA = "afflom/UORust", "v0.4.0", "a" * 40

def evidence():
    run = {"id": 123, "head_sha": SHA, "path": ".github/workflows/bootstrap.yml",
           "event": "push", "head_branch": "main", "status": "completed",
           "conclusion": "success", "run_attempt": 1, "repository": {"full_name": REPO}}
    jobs = {"total_count": 2, "jobs": [{"name": name, "status": "completed",
              "conclusion": "success"} for name in sorted(q.NATIVE)]}
    return {"total_count": 1, "workflow_runs": [run]}, jobs

def select(data, jobs):
    return q.select_receipt(data, lambda _: jobs, REPO, TAG, SHA)

class QualificationTest(unittest.TestCase):
    def test_exact_tag_and_main_receipts_pass(self):
        data, jobs = evidence()
        self.assertEqual(select(data, jobs), 123)
        data["workflow_runs"][0]["head_branch"] = TAG
        self.assertEqual(select(data, jobs), 123)

    def test_wrong_sha_event_ref_path_repo_and_run_status_refuse(self):
        for attr, bad in [("head_sha", "b" * 40), ("event", "pull_request"),
                ("head_branch", "feature"), ("status", "in_progress"),
                ("conclusion", "failure"), ("conclusion", "cancelled"),
                ("path", ".github/workflows/other.yml"), ("id", None)]:
            data, jobs = evidence()
            data["workflow_runs"][0][attr] = bad
            with self.subTest(attr=attr), self.assertRaises(q.Refused):
                select(data, jobs)
        data, jobs = evidence()
        data["workflow_runs"][0]["repository"]["full_name"] = "another/repo"
        with self.assertRaises(q.Refused): select(data, jobs)

    def test_missing_failed_duplicate_and_partial_native_jobs_refuse(self):
        changes = [
            lambda j: j["jobs"].pop(),
            lambda j: j["jobs"][0].update(conclusion="failure"),
            lambda j: j["jobs"][1].update(status="in_progress"),
            lambda j: j["jobs"][1].update(name=j["jobs"][0]["name"]),
            lambda j: j.update(total_count=3)]
        for modify in changes:
            data, jobs = evidence()
            modify(jobs)
            with self.assertRaises(q.Refused): select(data, jobs)

    def test_missing_and_paginated_run_evidence_refuses(self):
        data, jobs = evidence()
        data["total_count"] = 2
        with self.assertRaises(q.Refused): select(data, jobs)
        with self.assertRaises(q.Refused): select({"total_count": 0, "workflow_runs": []}, jobs)

    def test_one_failed_run_does_not_hide_another_completed_run(self):
        data, jobs = evidence()
        bad = deepcopy(data["workflow_runs"][0])
        bad["id"] = 122
        bad["conclusion"] = "failure"
        data["workflow_runs"].insert(0, bad)
        data["total_count"] = 2
        self.assertEqual(select(data, jobs), 123)

if __name__ == "__main__":
    unittest.main()
