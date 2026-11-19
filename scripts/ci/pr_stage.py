#!/usr/bin/env python3
"""Report PR development stage. This cannot authorize merging or skip native CI."""
import argparse
import json
import re
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
    if not isinstance(base, dict) or base.get("ref") != "main":
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--event", required=True, type=Path)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--summary", type=Path)
    args = parser.parse_args()
    status = classify(json.loads(args.event.read_text(encoding="utf-8")), args.repo)
    stage = status["stage"]
    print(f"PR stage: {stage}; source HEAD: {status['head_sha']}")
    print("Full native acceptance: NOT VERIFIED by this check")
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
