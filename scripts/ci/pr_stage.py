#!/usr/bin/env python3
"""Report PR development stage. This cannot authorize merging or skip native CI."""
import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

SHA = re.compile(r"[a-f0-9]{40}\Z")
REPOSITORY = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")
ACTIONS = {"opened", "reopened", "synchronize", "ready_for_review",
           "converted_to_draft", "edited"}


class AdmissionError(ValueError):
    pass


def classify(event, expected_repo):
    if not isinstance(event, dict) or not REPOSITORY.fullmatch(expected_repo):
        raise AdmissionError("invalid event or expected repository")
    if event.get("action") not in ACTIONS:
        raise AdmissionError("unsupported pull request action")
    repository, pr = event.get("repository"), event.get("pull_request")
    if (not isinstance(repository, dict)
            or repository.get("full_name") != expected_repo
            or not isinstance(pr, dict)):
        raise AdmissionError("pull request event belongs to another repository")
    if pr.get("state") != "open" or not isinstance(pr.get("draft"), bool):
        raise AdmissionError("pull request state or draft readiness is unknown")
    base, head = pr.get("base"), pr.get("head")
    if (not isinstance(base, dict) or base.get("ref") != "main"
            or not isinstance(base.get("sha"), str)
            or not SHA.fullmatch(base["sha"])):
        raise AdmissionError("base is not main")
    if not isinstance(head, dict) or not isinstance(head.get("repo"), dict):
        raise AdmissionError("head repository is missing")
    sha = head.get("sha")
    if not isinstance(sha, str) or not SHA.fullmatch(sha):
        raise AdmissionError("head commit is absent or invalid")
    if ((event["action"] == "ready_for_review" and pr["draft"])
            or (event["action"] == "converted_to_draft" and not pr["draft"])):
        raise AdmissionError("event action contradicts draft state")
    return {"stage": "development" if pr["draft"] else "candidate",
            "head_sha": sha, "merge_qualified": False}


def advisory_impact_from_trusted_base(repo_root, base_sha, head_sha, summary=None):
    """Run ONLY base-commit classifier code; PR head and files remain untrusted data.

    During the first bootstrap PR the base does not contain this classifier.
    Report FULL_UNKNOWN instead of executing the PR's new classifier or
    treating a missing policy as permission to skip work.
    """
    fallback = {"route": "full-unknown", "selected_development_suites":
                ["docs", "browser", "rust", "model-proof", "trust"],
                "native_acceptance_required": True, "merge_qualified": False,
                "advisory_only": True}
    if not SHA.fullmatch(base_sha) or not SHA.fullmatch(head_sha):
        return fallback
    try:
        policy = subprocess.run(
            ["git", "-C", str(repo_root), "show", base_sha + ":scripts/ci/change_impact.py"],
            check=True, capture_output=True, timeout=15,
        ).stdout
        with tempfile.TemporaryDirectory() as temp:
            trusted_script = Path(temp) / "change_impact.py"
            trusted_script.write_bytes(policy)
            command = [sys.executable, "-I", str(trusted_script), "--repo", str(repo_root),
                       "--base", base_sha, "--head", head_sha]
            if summary:
                command.extend(["--summary", str(summary)])
            result = subprocess.run(command, check=True, capture_output=True,
                                    text=True, timeout=45)
            impact = json.loads(result.stdout)
        if (impact.get("base_sha") != base_sha or impact.get("head_sha") != head_sha
                or impact.get("native_acceptance_required") is not True
                or impact.get("merge_qualified") is not False
                or impact.get("advisory_only") is not True
                or not isinstance(impact.get("selected_development_suites"), list)):
            return fallback
        return impact
    except (OSError, ValueError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return fallback

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--event", required=True, type=Path)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--summary", type=Path)
    args = parser.parse_args()
    event = json.loads(args.event.read_text(encoding="utf-8"))
    status = classify(event, args.repo)
    stage = status["stage"]
    print(f"PR stage: {stage}; source HEAD: {status['head_sha']}")
    print("Full native acceptance: NOT VERIFIED by this check")
    impact = advisory_impact_from_trusted_base(
        Path(__file__).resolve().parents[2], event["pull_request"]["base"]["sha"],
        status["head_sha"], args.summary)
    print("Advisory change impact (not a check skip): " + impact["route"])
    if args.summary and impact["route"] == "full-unknown":
        with args.summary.open("a", encoding="utf-8") as out:
            out.write("\nAdvisory impact: **FULL / UNVERIFIED**. No native gate was waived.\n")
    if args.output:
        with args.output.open("a", encoding="utf-8") as out:
            out.write(f"stage={stage}\nhead_sha={status['head_sha']}\n")
    if args.summary:
        with args.summary.open("a", encoding="utf-8") as out:
            out.write(f"### PR development preflight\n\nStage: {stage}. "
                      f"Source HEAD: {status['head_sha']}.\n\n"
                      "NOT merge acceptance: complete native just vv must still "
                      "pass for the actual merge candidate.\n")


if __name__ == "__main__":
    main()
