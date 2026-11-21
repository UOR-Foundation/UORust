#!/usr/bin/env python3
"""Advisory, fail-closed UORust PR change-impact report.

Run ONLY from a trusted base revision, treating the PR as data. The report
never grants merge qualification or changes the universal just-vv requirement.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tomllib

SHA = re.compile(r"[0-9a-f]{40}\\Z")
DOC_EXT = {".md", ".markdown", ".rst", ".adoc"}
PASSIVE_DOC_PREFIXES = ("docs/guides/", "docs/tutorials/", "docs/examples/")
MAX_CHANGED = 2000
CATEGORIES = ("docs", "browser", "rust", "model-proof", "trust")


class UnverifiedDiff(ValueError):
    pass


def git(repo: Path, *args: str) -> bytes:
    try:
        p = subprocess.run(["git", "-C", str(repo), "-c", "core.hooksPath=/dev/null", *args],
                           capture_output=True, check=True, timeout=20)
        return p.stdout
    except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as exc:
        raise UnverifiedDiff("git evidence missing or unreadable") from exc


def compare(repo: Path, base: str, head: str) -> list[str]:
    """Inspect exact commit objects and all add/delete/mode/rename paths."""
    if not SHA.fullmatch(base) or not SHA.fullmatch(head):
        raise UnverifiedDiff("base/head must be exact lowercase 40-character commits")
    for rev in (base, head):
        actual = git(repo, "rev-parse", "--verify", rev + "^{commit}").strip().decode("ascii")
        if actual != rev:
            raise UnverifiedDiff("requested revision does not identify the checked commit")
    raw = git(repo, "diff", "--no-ext-diff", "--no-renames", "--raw", "-z", base, head)
    entries = raw.split(b"\\0")
    if entries and entries[-1] == b"":
        entries.pop()
    if len(entries) % 2:
        raise UnverifiedDiff("malformed raw diff")
    paths: list[str] = []
    nonregular = False
    for i in range(0, len(entries), 2):
        fields = entries[i].decode("ascii", errors="strict").split()
        if len(fields) != 5 or not fields[0].startswith(":"):
            raise UnverifiedDiff("malformed raw diff record")
        before, after, action = fields[0][1:], fields[1], fields[4]
        if action not in {"A", "M", "D", "T"} or any(
            mode not in {"000000", "100644"} for mode in (before, after)
        ):
            nonregular = True
        paths.append(entries[i + 1].decode("utf-8", errors="strict"))
    if len(paths) > MAX_CHANGED:
        return ["__UNVERIFIED_TOO_MANY_FILES__"]
    if nonregular:
        paths.append("__NONREGULAR_FILE_OR_MODE__")
    return paths


def dependency_inputs(repo: Path, base: str) -> dict[str, list[str]]:
    """Read the trusted, pinned factory input map, never executable PR code."""
    graph = tomllib.loads(git(repo, "show", base + ":prism-factory/graph.toml").decode("utf-8"))
    sets = graph.get("sets")
    obligations = graph.get("obligation")
    if not isinstance(sets, dict) or not isinstance(obligations, list):
        raise UnverifiedDiff("factory dependency map is unavailable")
    result: dict[str, list[str]] = {}
    for obligation in obligations:
        name, keys = obligation.get("name"), obligation.get("sets", [])
        inputs = obligation.get("inputs", [])
        if not isinstance(name, str) or not isinstance(keys, list) or not isinstance(inputs, list):
            raise UnverifiedDiff("factory dependency map is invalid")
        expanded = []
        for key in keys:
            values = sets.get(key)
            if not isinstance(values, list):
                raise UnverifiedDiff("factory dependency map references an unknown set")
            expanded.extend(values)
        expanded.extend(inputs)
        if not all(isinstance(x, str) and x for x in expanded):
            raise UnverifiedDiff("factory dependency input is invalid")
        result[name] = expanded
    if "vv" not in result or "self-model-check" not in result:
        raise UnverifiedDiff("factory dependency boundary is incomplete")
    return result


def match_input(path: str, expression: str) -> bool:
    entry = expression.removeprefix("@base:")
    return path.startswith(entry) if entry.endswith("/") else path == entry


def consumers(path: str, dependency_map: dict[str, list[str]]) -> list[str]:
    names = []
    for name, inputs in dependency_map.items():
        positive = any(match_input(path, x) for x in inputs if not x.startswith("!"))
        excluded = any(match_input(path, x[1:]) for x in inputs if x.startswith("!"))
        if positive and not excluded:
            names.append(name)
    return sorted(names)


def safe_path(path: str) -> bool:
    return bool(path and not path.startswith(("/", "../")) and "\\\\" not in path
                and not any(s in {".", "..", ""} for s in path.split("/"))
                and all(ord(c) >= 32 for c in path))


def impact_of(path: str, deps: dict[str, list[str]]) -> set[str]:
    """Conservative union: unknown paths and any unsafe modes are FULL."""
    if not safe_path(path):
        return set(CATEGORIES)
    suffix = PurePosixPath(path).suffix.lower()
    doc = suffix in DOC_EXT
    affected = consumers(path, deps)
    if path.startswith(PASSIVE_DOC_PREFIXES) and doc and not affected:
        return {"docs"}
    impact = set()
    if (path.startswith(("site/", "try/", "frame/", "identity/", "bootstrap/", "scripts/content/"))
            or path in {"package.json", "package-lock.json", "frame.lock"}):
        impact.add("browser")
    if (path.startswith(("crates/", "xtask/", "features/", "scripts/mir/", "scripts/hooks/"))
            or suffix == ".rs" or path in {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml"}):
        impact.add("rust")
    if (path.startswith(("app/", "model/", "brand/", "prism-factory/", "scripts/self-model/", "scripts/experience/"))
            or path in {"README.md", "CONFORMANCE.md", "frontier.toml", "lift-verdicts.toml", "Justfile", "site/app.json", "frame.lock"}):
        impact.add("model-proof")
    if (path.startswith((".github/", ".cargo/", ".qortara/", "scripts/ci/", "scripts/release/", "scripts/hooks/"))
            or path in {"AGENTS.md", "CLAUDE.md", "GOVERNANCE.md", "SECURITY.md", "VERIFICATION.md",
                        "ARCHITECTURE_INDEX.md", "PAGES-VERIFICATION.md", "template.lock", "prismpm.lock",
                        "template-contract.json", "TEMPLATE-CONTRACT.md", "TEMPLATE-VERIFICATION.md",
                        "Cargo.lock", "Cargo.toml", "rust-toolchain.toml", "Justfile", "package-lock.json",
                        "LICENSE", "LICENSE.md", "deny.toml", "clippy.toml"}):
        impact.add("trust")
    if affected and doc:
        impact.add("model-proof")
    if not impact:
        return set(CATEGORIES)
    return impact


def classify(paths: list[str], deps: dict[str, list[str]]) -> dict:
    if not paths or len(paths) > MAX_CHANGED or not deps:
        paths = ["__UNVERIFIED_DIFF__"]
    selected: set[str] = set()
    for path in set(paths):
        selected.update(impact_of(path, deps))
    full = selected == set(CATEGORIES)
    return {"route": "full-unknown" if full else "+".join(x for x in CATEGORIES if x in selected),
            "selected_development_suites": [x for x in CATEGORIES if x in selected],
            "changed_count": len(set(paths)), "native_acceptance_required": True,
            "merge_qualified": False, "advisory_only": True}


def report(repo: Path, base: str, head: str) -> dict:
    """Missing evidence must not become a low-risk classification."""
    try:
        result = classify(compare(repo, base, head), dependency_inputs(repo, base))
    except (UnverifiedDiff, OSError, UnicodeError, ValueError):
        result = classify(["__UNVERIFIED_DIFF__"], {"vv": ["Justfile"]})
        result["reason"] = "unverified-base-head-or-dependency-map"
    result.update({"base_sha": base, "head_sha": head})
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--summary", type=Path)
    args = parser.parse_args()
    result = report(args.repo.resolve(), args.base, args.head)
    print(json.dumps(result, sort_keys=True))
    if args.summary:
        with args.summary.open("a", encoding="utf-8") as output:
            output.write("\\n### Advisory PR impact classification (not merge acceptance)\\n\\n"
                         f"Exact base: `{result['base_sha']}`  \\nExact head: `{result['head_sha']}`  \\n"
                         f"Impact route: **{result['route']}**  \\n"
                         f"Development suites: {', '.join(result['selected_development_suites'])}  \\n"
                         "Full native `just vv` remains mandatory before merge; "
                         "this report does not skip, authorize, or claim execution of a job.\\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
