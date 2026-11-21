#!/usr/bin/env python3
"""Proof of no change: a package's optimized MIR before and after a rewrite.

`uorust figures rewrite` replaces numeric literals by macros that expand to
the same tokens, so the compiler's optimized MIR of every function must be
the same before and after. This script builds the library of one package in
two source trees (before, after) with `--emit=mir` at an optimizing level,
once per configuration (a feature set) and per overflow-check setting, and
compares the MIR item by item: functions, constants, promoted constants and
allocations.

What it normalises, and why each is unstable without being a change:
- the three comment lines rustc writes at the top of the file;
- the numbers of allocations (`allocN`, `ALLOCN`), which rustc assigns in
  the order it interns them: they are renumbered in order of appearance;
- the file name's crate hash, which the target directory decides (the
  file is found by the library's crate name, not by its hash);
- the span rustc names an impl block or a closure by (`<impl at
  file:line:col: line:col>`, `{closure@file:line:col: line:col}`), which a
  rewrite moves when the formatter breaks a line above it: such an item is
  named by its file, and two of one file by their order.
Spans are not in the text rustc emits for `--emit=mir`, so a literal
rewritten inside a line moves nothing the comparison reads. Every value
made of a caller's location is therefore outside this proof: a panic's
`file:line:col`, and what `line!()`, `column!()`, `Location::caller()` and a
`#[track_caller]` function return: the caller's location a tracked call
passes is not in the MIR text, so two trees whose calls stand at other
columns compare equal. The rewrite itself refuses a site on a line that
reads its own column (`column!()`, `Location::caller()`, a call of a
function marked `#[track_caller]` that reads its caller, read from the
source of the workspace and of every package its build resolves, under any
name a `use ... as` gives it, written by a `macro_rules!`, or called by an
operator's token; a macro the figures do not read, a metavariable's among
them; and, where the source cannot show every tracked function, any call),
unless the rule file declares those columns not behaviour (`[locations]`
with `columns = "not-behaviour"` and the declarer's `reason`), when it
places the site and records the columns it moves as the register's
`[[moved]]` rows; and it refuses, whatever the rules declare, a layout that
moves a line reading its own line or column; what is left
outside both is the location a panic reports (`unwrap`, indexing,
arithmetic overflow, a tracked function that passes its caller's location
only to a panic), which only `cargo test` compares.

Usage:
  mir-equal.py --before DIR --after DIR --package NAME --scratch DIR
               [--config default|no-default|FEATURES|no-default:FEATURES]...
               [--overflow on|off]... [--opt-level N] [--keep]

Exit 0 when every configuration's MIR is equal, 1 when an item differs,
2 when a build fails. It prints, per configuration, the items compared and
those that differ, and the seconds and bytes each build took.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time

ALLOC = re.compile(r"\b(alloc|ALLOC)(\d+)\b")
SPAN = re.compile(r"(<impl at |\{[a-z ]+@)([^:\s]+):\d+:\d+: \d+:\d+")


def lib_name(tree, package):
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--offline"],
        cwd=tree, capture_output=True, text=True, check=True,
    ).stdout
    for p in json.loads(out)["packages"]:
        if p["name"] == package:
            for t in p["targets"]:
                if "lib" in t["kind"] or "rlib" in t["kind"]:
                    return t["name"].replace("-", "_")
    raise SystemExit(f"{package} has no library target in {tree}")


def config_args(config):
    if config == "default":
        return []
    if config == "no-default":
        return ["--no-default-features"]
    if config.startswith("no-default:"):
        return ["--no-default-features", "--features", config.split(":", 1)[1]]
    return ["--features", config]


def du(path):
    total = 0
    for root, _, files in os.walk(path):
        for f in files:
            try:
                total += os.lstat(os.path.join(root, f)).st_size
            except OSError:
                pass
    return total


def build(tree, package, lib, target, config, overflow, opt):
    if os.path.isdir(target):
        shutil.rmtree(target)
    cmd = ["cargo", "rustc", "--offline", "-q", "-p", package, "--lib"]
    cmd += config_args(config)
    cmd += ["--", "-C", f"opt-level={opt}", "-C", "debuginfo=0",
            "-C", f"overflow-checks={overflow}", "--emit=mir"]
    env = dict(os.environ, CARGO_TARGET_DIR=target, CARGO_INCREMENTAL="0")
    start = time.monotonic()
    done = subprocess.run(cmd, cwd=tree, env=env, capture_output=True, text=True)
    seconds = time.monotonic() - start
    if done.returncode != 0:
        sys.stderr.write(done.stderr)
        return None, seconds, du(target)
    deps = os.path.join(target, "debug", "deps")
    mirs = [f for f in os.listdir(deps) if f.startswith(lib + "-") and f.endswith(".mir")]
    if len(mirs) != 1:
        sys.stderr.write(f"expected one {lib}-*.mir in {deps}, found {mirs}\n")
        return None, seconds, du(target)
    with open(os.path.join(deps, mirs[0]), encoding="utf-8") as f:
        return f.read(), seconds, du(target)


def items(text):
    """The MIR's items by their header line, each normalised."""
    lines = [l for l in text.splitlines() if not l.startswith("// ")]
    numbers = {}

    def renumber(m):
        key = m.group(2)
        numbers.setdefault(key, str(len(numbers)))
        return m.group(1) + numbers[key]

    lines = [SPAN.sub(r"\1\2", ALLOC.sub(renumber, l)) for l in lines]
    out, current, body = {}, None, []

    def close():
        if current is not None:
            key, n = current, 0
            while key in out:
                n += 1
                key = f"{current} #{n}"
            out[key] = "\n".join(body).rstrip()

    for line in lines:
        if line and not line[0].isspace() and line != "}":
            close()
            current, body = line.split(" {")[0].split(" = ")[0], [line]
        elif current is not None:
            body.append(line)
    close()
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--before", required=True)
    ap.add_argument("--after", required=True)
    ap.add_argument("--package", required=True)
    ap.add_argument("--scratch", required=True)
    ap.add_argument("--config", action="append")
    ap.add_argument("--overflow", action="append", choices=["on", "off"])
    ap.add_argument("--opt-level", default="2")
    ap.add_argument("--keep", action="store_true")
    a = ap.parse_args()
    configs = a.config or ["default"]
    overflows = a.overflow or ["on", "off"]
    lib = lib_name(a.before, a.package)
    a.scratch = os.path.abspath(a.scratch)
    os.makedirs(a.scratch, exist_ok=True)
    differ = False
    for config in configs:
        for overflow in overflows:
            label = f"{config}, overflow-checks={overflow}"
            mir = {}
            for side, tree in (("before", a.before), ("after", a.after)):
                target = os.path.join(a.scratch, f"target-{side}")
                text, seconds, size = build(tree, a.package, lib, target, config, overflow, a.opt_level)
                print(f"{label}: {side} built in {seconds:.1f} s, {size} bytes"
                      + ("" if text is None else f", {len(text)} bytes of MIR"))
                if not a.keep:
                    shutil.rmtree(target, ignore_errors=True)
                if text is None:
                    print(f"{label}: the {side} build failed")
                    return 2
                mir[side] = items(text)
            before, after = mir["before"], mir["after"]
            names = sorted(set(before) | set(after))
            bad = [n for n in names if before.get(n) != after.get(n)]
            print(f"{label}: {len(names)} items, {len(names) - len(bad)} equal, {len(bad)} differ")
            for n in bad:
                differ = True
                print(f"  differs: {n}")
                b = (before.get(n) or "(absent)").splitlines()
                c = (after.get(n) or "(absent)").splitlines()
                shown = 0
                for x, y in zip(b + [""] * (len(c) - len(b)), c + [""] * (len(b) - len(c))):
                    if x != y and shown < 3:
                        print(f"    - {x.strip()}\n    + {y.strip()}")
                        shown += 1
    return 1 if differ else 0


if __name__ == "__main__":
    sys.exit(main())
