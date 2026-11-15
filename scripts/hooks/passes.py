"""The pre-push hook's record of tested patches.

A push whose tests passed records its patch (``git patch-id --stable`` of
what it adds to main) with the main it was tested on. When main moves before
the push lands, ``lane-push.sh`` rebases and pushes again; the hook then skips
the tests of a rebased patch with a pass on record when every commit main
gained since touches no file the patch touches and none of the paths the
patch's tests read (``inputs``: the touched packages' directories, and for
uorust's tests the workflows, scripts, app, model and manifests). Otherwise it
tests again. CI's bootstrap stays the gate on the combined tree.

    passes.py record <dir> <base> <head>
    passes.py check <dir> <base> <head> <input>...   (exit 0: skip the tests)
    passes.py self-test
"""
import os
import subprocess
import sys


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def patch_id(base, head):
    diff = git("diff", base, head)
    out = subprocess.run(["git", "patch-id", "--stable"], input=diff, check=True,
                         capture_output=True, text=True).stdout.split()
    return out[0] if out else ""


def changed(a, b):
    return [p for p in git("diff", "--name-only", a, b).splitlines() if p]


def reads(path, inputs):
    """Whether a test that reads ``inputs`` (files or directories) reads ``path``."""
    return any(path == i or path.startswith(i.rstrip("/") + "/") for i in inputs)


def disjoint(newer, touched, inputs):
    """Whether no file main gained touches the patch or what its tests read."""
    return not any(p in set(touched) or reads(p, inputs) for p in newer)


def record(where, base, head):
    os.makedirs(where, exist_ok=True)
    with open(os.path.join(where, patch_id(base, head)), "w") as f:
        f.write(f"{head} {base}\n")


def check(where, base, head, inputs):
    key = patch_id(base, head)
    path = os.path.join(where, key)
    if not key or not os.path.isfile(path):
        return False
    tested, tested_base = open(path).read().split()
    try:
        newer = changed(tested_base, base)
    except subprocess.CalledProcessError:
        return False
    if not disjoint(newer, changed(base, head), inputs):
        return False
    print(f"pre-push: this patch passed its tests as {tested[:12]}; main gained only "
          f"{len(newer)} files neither it nor its tests read", file=sys.stderr)
    return True


def self_test():
    inputs = ["crates/uorust", ".github", "Cargo.lock"]
    touched = ["crates/uorust/src/certify.rs", "Justfile"]
    cases = [
        ([], True),
        (["crates/std-lane/src/x.rs"], True),
        (["README.md", "crates/uorustx/a.rs"], True),
        (["Justfile"], False),
        (["crates/uorust/src/embed.rs"], False),
        ([".github/workflows/core.yml"], False),
        (["Cargo.lock"], False),
    ]
    for newer, want in cases:
        assert disjoint(newer, touched, inputs) == want, (newer, want)
    print("passes: the disjointness rule holds")


if __name__ == "__main__":
    cmd, rest = sys.argv[1], sys.argv[2:]
    if cmd == "record":
        record(*rest[:3])
    elif cmd == "check":
        sys.exit(0 if check(rest[0], rest[1], rest[2], rest[3:]) else 1)
    elif cmd == "self-test":
        self_test()
    else:
        sys.exit(f"passes.py: no command {cmd}")
