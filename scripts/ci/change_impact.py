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

SHA = re.compile(r"[0-9a-f]{40}\Z")
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
    entries = raw.split(b"\0")
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
    return path.startswith(entry)


def consumers(path: str, dependency_map: dict[str, list[str]]) -> list[str]:
    """Mirror factory graph covers(): longest inclusion beats exclusion.

    @base: inputs conservatively count as normal paths for changed-base checks.
    """
    names = []
    for name, inputs in dependency_map.items():
        includes = [
            len(x.removeprefix("@base:").encode()) + 1
            for x in inputs if not x.startswith("!") and match_input(path, x)
        ]
        excludes = [
            len(x[1:].removeprefix("@base:").encode()) + 1
            for x in inputs if x.startswith("!") and match_input(path, x[1:])
        ]
        if max(includes, default=0) > max(excludes, default=0):
            names.append(name)
    return sorted(names)


def safe_path(path: str) -> bool:
    return bool(path and not path.startswith(("/", "../")) and "\\" not in path
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



# CI-02: independently testable obligation selection, not a merge-time waiver.
# The authoritative list of checks and their inputs is the trusted base
# factory graph plus that revision's Justfile. Never parse PR-head policy.
FULL_REASON_PREFIXES = (
    ".github/", ".cargo/", ".qortara/", "prism-factory/",
    "scripts/ci/", "scripts/release/", "scripts/hooks/",
)
FULL_REASON_FILES = {
    "Justfile", "prismpm.lock", "template.lock", "template-contract.json",
    "AGENTS.md", "GOVERNANCE.md", "SECURITY.md", "CLAUDE.md",
    "rust-toolchain.toml", "Cargo.lock", "Cargo.toml", "deny.toml",
    "TEMPLATE-CONTRACT.md", "TEMPLATE-VERIFICATION.md",
}
BROWSER_ONLY_PREFIXES = ("site/", "try/", "frame/", "identity/")
VV_PARTS = ("fmt-check", "model", "template-check", "self-model-check",
            "lint", "test", "deny", "app", "uorust")


def obligation_plan(paths: list[str], graph: dict, justfile: str) -> dict:
    """Derive a conservative per-check execution plan from trusted base inputs.

    This is an executable selection contract for later template adoption.
    Current bootstrap still executes the full integrated just vv. Until that
    change is governed, this plan is NOT merge qualification.
    """
    nodes = graph.get("obligation")
    sets = graph.get("sets")
    if not isinstance(nodes, list) or not isinstance(sets, dict):
        raise UnverifiedDiff("factory graph incomplete")
    by_name = {}
    for node in nodes:
        if not isinstance(node, dict) or not isinstance(node.get("name"), str):
            raise UnverifiedDiff("invalid factory obligation")
        name = node["name"]
        if name in by_name:
            raise UnverifiedDiff("duplicate obligation")
        by_name[name] = node
    vv = by_name.get("vv", {})
    parts = vv.get("needs")
    vv_recipes = [line.split("#", 1)[0].strip() for line in justfile.splitlines()
                  if line.startswith("vv:")]
    if (len(vv_recipes) != 1 or not isinstance(parts, list)
            or tuple(parts) != VV_PARTS
            or vv_recipes[0] != "vv: " + " ".join(parts)
            or any(by_name.get(name, {}).get("recipe") != name for name in parts)):
        raise UnverifiedDiff("factory graph and normative vv diverge")

    inputs = {}
    for name in parts:
        node = by_name[name]
        keys, extra, needs = node.get("sets", []), node.get("inputs", []), node.get("needs", [])
        if not all(isinstance(v, list) for v in (keys, extra, needs)):
            raise UnverifiedDiff("malformed factory inputs or dependencies")
        if any(not isinstance(k, str) or k not in sets for k in keys):
            raise UnverifiedDiff("missing factory input set")
        expanded = [p for key in keys for p in sets[key]] + extra
        if not all(isinstance(x, str) and x for x in expanded):
            raise UnverifiedDiff("invalid factory input")
        if any(dep not in parts for dep in needs):
            raise UnverifiedDiff("unregistered acceptance dependency")
        inputs[name] = expanded

    full = set(parts)
    def result(selected, reason):
        ordered = [part for part in parts if part in selected]
        return {
            "schema": "uor/ci-obligation-plan/1",
            "mode": "full" if selected == full else "selective",
            "required_obligations": ordered,
            "omitted_obligations": [part for part in parts if part not in selected],
            "reason": reason,
            "native_acceptance_required": True,
            "merge_qualified": False,
            "advisory_only": True,
        }

    if not paths or len(paths) > MAX_CHANGED:
        return result(full, "empty-or-unverified-diff")
    touched = set()
    for path in set(paths):
        if not safe_path(path):
            return result(full, "unsafe-path-or-mode")
        if path in FULL_REASON_FILES or path.startswith(FULL_REASON_PREFIXES):
            return result(full, "trust-or-execution-policy-input")
        affected = set(consumers(path, inputs))
        if not affected:
            if (path.startswith(PASSIVE_DOC_PREFIXES)
                    and PurePosixPath(path).suffix.lower() in DOC_EXT):
                continue
            # An existing separate browser acceptance workflow covers this
            # tree. Its removal/change is in the full-sensitive .github/ path.
            if path.startswith(BROWSER_ONLY_PREFIXES):
                continue
            return result(full, "unmapped-input")
        touched.update(affected)

    # An affected obligation must never run without its declared prerequisites.
    pending = list(touched)
    while pending:
        current = pending.pop()
        for need in by_name[current].get("needs", []):
            if need not in touched:
                touched.add(need)
                pending.append(need)
    return result(touched, "trusted-factory-input-closure")


def report_obligation_plan(repo: Path, base: str, head: str) -> dict:
    """Trusted-base graph and Justfile; unavailable evidence routes full."""
    try:
        paths = compare(repo, base, head)
        graph = tomllib.loads(git(repo, "show", base + ":prism-factory/graph.toml").decode())
        justfile = git(repo, "show", base + ":Justfile").decode()
        planned = obligation_plan(paths, graph, justfile)
    except (UnverifiedDiff, OSError, UnicodeError, ValueError, KeyError, TypeError):
        planned = {
            "schema": "uor/ci-obligation-plan/1",
            "mode": "full",
            "required_obligations": list(VV_PARTS),
            "omitted_obligations": [],
            "reason": "unverified-policy-or-diff",
            "native_acceptance_required": True,
            "merge_qualified": False,
            "advisory_only": True,
        }
    planned["base_sha"] = base
    planned["head_sha"] = head
    return planned


def execute_obligation_plan(repo: Path, plan: dict) -> dict:
    """Run exactly the selected, fixed Justfile commands on the checked head.

    A trusted-base invocation may use this for scoped development feedback.
    Do not equate this receipt with whole-PR merge qualification: independent
    required checks and branch protection must separately validate evidence.
    """
    if (plan.get("schema") != "uor/ci-obligation-plan/1"
            or plan.get("native_acceptance_required") is not True
            or plan.get("merge_qualified") is not False
            or plan.get("advisory_only") is not True):
        raise UnverifiedDiff("untrusted or incomplete obligation plan")
    head = git(repo, "rev-parse", "--verify", "HEAD^{commit}").strip().decode("ascii")
    if head != plan.get("head_sha") or not SHA.fullmatch(head):
        raise UnverifiedDiff("checked revision differs from obligation plan")
    parts = plan.get("required_obligations")
    if (not isinstance(parts, list) or len(parts) != len(set(parts))
            or any(part not in VV_PARTS for part in parts)
            or parts != [part for part in VV_PARTS if part in parts]
            or plan.get("omitted_obligations") !=
            [part for part in VV_PARTS if part not in parts]):
        raise UnverifiedDiff("obligation plan is missing, duplicated or reordered")
    mode = plan.get("mode")
    if mode not in ("full", "selective") or (mode == "full") != (parts == list(VV_PARTS)):
        raise UnverifiedDiff("plan mode and selected obligations disagree")

    # A SHA labels committed content, not a dirty worktree or index.
    # Reject both modified tracked and untracked nonignored inputs.
    if git(repo, "status", "--porcelain=v1", "-z", "--untracked-files=normal"):
        raise UnverifiedDiff("uncommitted files would invalidate component receipt")
    # Cheap whitespace fault must still be refused even for passive docs.
    git(repo, "diff", "--check", plan["base_sha"], head)
    commands = [["just", "vv"]] if mode == "full" else [["just", name] for name in parts]
    observed = []
    for command in commands:
        proc = subprocess.run(command, cwd=repo, check=False)
        observed.append({"command": command, "exit_code": proc.returncode})
        if proc.returncode != 0:
            break
    if git(repo, "status", "--porcelain=v1", "-z", "--untracked-files=normal"):
        raise UnverifiedDiff("component execution changed tracked or untracked inputs")
    passed = len(observed) == len(commands) and all(
        row["exit_code"] == 0 for row in observed
    )
    return {
        "schema": "uor/ci-component-receipt/1",
        "base_sha": plan["base_sha"],
        "head_sha": head,
        "mode": mode,
        "selected": parts,
        "observed": observed,
        "components_passed": passed,
        "merge_qualified": False,
    }


def modeled_execution_diagnostic(code: str, **details) -> dict:
    """Typed public CLI failure from the registered CI-02 refusal model."""
    document = tomllib.loads(
        (Path(__file__).resolve().parents[2] / "model/ci-obligation-diagnostics.toml")
        .read_text(encoding="utf-8")
    )
    rows = document.get("refusal", [])
    required = {
        "CI_COMPONENT_CONTROLS_INCOMPLETE",
        "CI_COMPONENT_REFUSED",
        "CI_COMPONENT_FAILED",
    }
    if (document.get("schema") != "uor/ci-obligation-diagnostics/1"
            or not isinstance(rows, list) or len(rows) != len(required)
            or any(not isinstance(row, dict) for row in rows)
            or {row.get("code") for row in rows} != required):
        raise ValueError("CI-02 refusal model is missing or malformed")
    row = next(row for row in rows if row["code"] == code)
    if any(not isinstance(row.get(key), str) or not row[key]
           for key in ("kind", "message")):
        raise ValueError("CI-02 refusal model has incomplete fields")
    return {"code": code, "kind": row["kind"], "message": row["message"], **details}

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--summary", type=Path)
    parser.add_argument("--execute", action="store_true",
                        help="execute verified selected checks; not merge qualification")
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()
    result = report(args.repo.resolve(), args.base, args.head)
    result["obligation_plan"] = report_obligation_plan(
        args.repo.resolve(), args.base, args.head
    )
    print(json.dumps(result, sort_keys=True))
    if args.execute:
        try:
            receipt = execute_obligation_plan(
                args.repo.resolve(), result["obligation_plan"]
            )
            if args.receipt:
                args.receipt.write_text(
                    json.dumps(receipt, sort_keys=True) + "\n", encoding="utf-8"
                )
        except (UnverifiedDiff, OSError, ValueError, KeyError, TypeError, UnicodeError) as exc:
            print(json.dumps(modeled_execution_diagnostic(
                "CI_COMPONENT_REFUSED", reason=str(exc)
            ), sort_keys=True), file=sys.stderr)
            return 2
        print(json.dumps(receipt, sort_keys=True))
        if not receipt["components_passed"]:
            print(json.dumps(modeled_execution_diagnostic(
                "CI_COMPONENT_FAILED", base_sha=receipt["base_sha"],
                head_sha=receipt["head_sha"], observed=receipt["observed"]
            ), sort_keys=True), file=sys.stderr)
            return 1
        return 0
    if args.summary:
        with args.summary.open("a", encoding="utf-8") as output:
            output.write("\n### Advisory PR impact classification (not merge acceptance)\n\n"
                         f"Exact base: `{result['base_sha']}`  \nExact head: `{result['head_sha']}`  \n"
                         f"Impact route: **{result['route']}**  \n"
                         f"Development suites: {', '.join(result['selected_development_suites'])}  \n"
                         "Full native `just vv` remains mandatory before merge; "
                         "this report does not skip, authorize, or claim execution of a job.\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
