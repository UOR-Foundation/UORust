#!/usr/bin/env bash
# Push this worktree's commits to main, one lane at a time:
#
#     LANE_N=<n> scripts/hooks/lane-push.sh
#
# The push lock (beside the build dir's hooks lock) is taken before the
# pre-push hook's tests and held through the push, so lanes on this machine
# no longer race each other; only CI's bot commits move main meanwhile. When
# one does ("cannot lock ref", a non-fast-forward), the commits are rebased
# onto the new main and pushed again: the hook skips the tests of a patch it
# already passed when main gained nothing that patch or its tests read
# (scripts/hooks/passes.py), and tests again otherwise.
set -euo pipefail
root=$(git rev-parse --show-toplevel)
build=${CARGO_BUILD_BUILD_DIR:-/home/user/uorust-engine/target}
mkdir -p "$build"
exec 8>"$build/.push.lock"
flock 8
log=$(mktemp)
trap 'rm -f "$log"' EXIT
for attempt in 1 2 3 4 5 6 7 8; do
  git -C "$root" fetch -q origin
  git -C "$root" rebase -q origin/main
  if git -C "$root" push origin HEAD:main 2>&1 | tee "$log"; then
    if ! grep -q "rejected\|failed to push" "$log"; then
      exit 0
    fi
  fi
  if ! grep -q "cannot lock ref\|non-fast-forward\|fetch first" "$log"; then
    echo "lane-push: the push was refused (above)" >&2
    exit 1
  fi
  echo "lane-push: main moved during attempt $attempt; rebasing and pushing again" >&2
done
echo "lane-push: main kept moving" >&2
exit 1
