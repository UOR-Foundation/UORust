#!/usr/bin/env bash
# A writer's obligation: run the writer and pass only when it changed none of
# the files it writes, so that a pass means the committed outputs are what the
# writer writes now.
#
#     written.sh PATH... -- COMMAND...
#
# Each PATH is a file or a directory the writer owns; a file it adds, removes
# or changes under one fails the obligation, naming the file.
set -euo pipefail
paths=()
while [[ $# -gt 0 && $1 != -- ]]; do paths+=("$1"); shift; done
[[ $# -gt 1 && ${#paths[@]} -gt 0 ]] || { echo 'usage: written.sh PATH... -- COMMAND...' >&2; exit 64; }
shift
digest() { find "${paths[@]}" -type f -print0 2>/dev/null | LC_ALL=C sort -z | xargs -0 -r sha256sum; }
before=$(mktemp); after=$(mktemp)
trap 'rm -f "$before" "$after"' EXIT
digest >"$before"
"$@"
digest >"$after"
if ! cmp -s "$before" "$after"; then
  echo "written.sh: the writer changed what it writes; the committed outputs are stale:" >&2
  diff "$before" "$after" | sed -n 's/^[<>] [0-9a-f]*  /  /p' | LC_ALL=C sort -u >&2
  exit 1
fi
echo "written.sh: the writer left ${paths[*]} as committed"
