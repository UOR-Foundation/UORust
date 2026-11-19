#!/usr/bin/env python3
"""Refuse publishing a tag without successful native bootstrap on its exact SHA."""
import argparse
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from urllib.request import Request, urlopen

NATIVE = {"acceptance / ubuntu-24.04", "acceptance / ubuntu-24.04-arm"}
REPO = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")
TAG = re.compile(r"v[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.+-]+)?\Z")
SHA = re.compile(r"[0-9a-f]{40}\Z")

class Refused(ValueError):
    pass

def select_receipt(data, get_jobs, repo, tag, sha):
    runs = data.get("workflow_runs") if isinstance(data, dict) else None
    if not isinstance(runs, list) or data.get("total_count") != len(runs):
        raise Refused("missing or paginated bootstrap run evidence")
    for run in runs:
        if not isinstance(run, dict):
            continue
        if (run.get("head_sha") != sha or
            run.get("path") != ".github/workflows/bootstrap.yml" or
            run.get("event") not in ("push", "workflow_dispatch") or
            run.get("head_branch") not in ("main", tag) or
            run.get("status") != "completed" or run.get("conclusion") != "success" or
            not isinstance(run.get("run_attempt"), int) or run["run_attempt"] < 1 or
            not isinstance(run.get("repository"), dict) or
            run["repository"].get("full_name") != repo or
            not isinstance(run.get("id"), int) or run["id"] < 1):
            continue
        try:
            jobs_data = get_jobs(run["id"])
        except (ValueError, OSError):
            continue
        jobs = jobs_data.get("jobs") if isinstance(jobs_data, dict) else None
        if not isinstance(jobs, list) or jobs_data.get("total_count") != len(jobs):
            continue
        native = [j for j in jobs if isinstance(j, dict) and j.get("name") in NATIVE]
        if (len(native) == 2 and {j["name"] for j in native} == NATIVE and
            all(j.get("status") == "completed" and j.get("conclusion") == "success" for j in native)):
            return run["id"]
    raise Refused("no successful complete two-architecture bootstrap on exact tagged SHA")

def get_api(endpoint, token):
    req = Request("https://api.github.com/" + endpoint, headers={
        "Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28"})
    with urlopen(req, timeout=20) as response:
        body = response.read(4_000_001)
    if len(body) > 4_000_000:
        raise Refused("receipt API response is oversized")
    obj = json.loads(body)
    if not isinstance(obj, dict):
        raise Refused("receipt API response is not an object")
    return obj

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--tag", required=True)
    args = parser.parse_args()
    if not REPO.fullmatch(args.repo) or not TAG.fullmatch(args.tag):
        raise Refused("invalid tag or repository")
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    tagged = subprocess.check_output(["git", "rev-parse", "-q", "--verify",
                                      "refs/tags/" + args.tag + "^{commit}"], text=True).strip()
    version = tomllib.loads(Path("Cargo.toml").read_text())["workspace"]["package"]["version"]
    if not SHA.fullmatch(head) or tagged != head or args.tag != "v" + version:
        raise Refused("tag, workspace version and checked-out source do not agree")
    token = os.environ.get("GH_TOKEN")
    if not token:
        raise Refused("missing actions:read credential")
    root = "repos/" + args.repo
    runs = get_api(root + "/actions/workflows/bootstrap.yml/runs?head_sha=" + head + "&per_page=100", token)
    receipt = select_receipt(runs, lambda run: get_api(root + "/actions/runs/" + str(run) +
                             "/jobs?per_page=100", token), args.repo, args.tag, head)
    print("tag accepted at " + head + " by complete native bootstrap run #" + str(receipt))

if __name__ == "__main__":
    try:
        main()
    except (Refused, OSError, ValueError, subprocess.CalledProcessError, json.JSONDecodeError) as exc:
        print("release refused: " + str(exc), file=sys.stderr)
        sys.exit(1)
