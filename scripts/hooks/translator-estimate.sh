#!/usr/bin/env bash
# The translator ratchet (UE-18) estimated before a push, without the MIR
# dump: the work tree and BASE (default origin/main) each projected by the
# surface path alone into a fresh register, and the head's register with the
# work tree's certified manifest held against the base's register with
# BASE's manifest (`uorust certify ratchet`). Both sides are read by the same
# path and the same binary, so what the MIR path would add lifts on both
# alike and the difference is the change's own: each function the change
# adds or edits has a κ no manifest certifies yet. Refused, naming each
# function and its cause ("lifted ..., not yet certified" is owed to an
# export; "residue (...)" to a rewrite), when the head has more uncertified
# translator functions than the base.
# usage: scripts/hooks/translator-estimate.sh [BASE] [CARGO_ARG...]
#        UORUST=<binary> scripts/hooks/translator-estimate.sh [BASE]
# Arguments after BASE go to cargo (a lane's `--config` flags), as
# ratchet-staged.sh takes them; UORUST names a built binary instead.
set -euo pipefail
root=$(git rev-parse --show-toplevel)
cd "$root"
base=${1:-origin/main}
shift $(($# > 0 ? 1 : 0))
if [[ -n ${UORUST:-} ]]; then
  uorust=("$UORUST")
else
  uorust=(cargo run -q --locked -p uorust "$@" --)
fi
out=target/translator-estimate
rm -rf "$out" && mkdir -p "$out/base-tree"
git archive "$base" | tar -x -C "$out/base-tree"
# The command line of `just register`, without the MIR inputs.
project() {
  "${uorust[@]}" project "$1" --out "$out/$2" --name uorust --last-modified 2026-11-11T00:00:00Z --register "$out/$2.toml" >/dev/null
}
project . head
project "$out/base-tree" base
manifest=crates/certified/manifest.toml
"${uorust[@]}" certify ratchet --register "$out/head.toml" --manifest "$manifest" \
  --base-register "$out/base.toml" --base-manifest "$out/base-tree/$manifest"
