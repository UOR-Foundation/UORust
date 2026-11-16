#!/usr/bin/env bash
# The translator ratchet (UE-18) estimated before a push, without the MIR
# dump: the work tree and BASE (default origin/main) each projected by the
# surface path alone into a fresh register, and the head's register with the
# work tree's certified manifest held against the base's register with
# BASE's manifest (`uorust certify ratchet`). Each function the change adds
# or edits has a κ no manifest certifies yet. Refused, naming each function
# and its cause ("lifted ..., not yet certified" is owed to an export;
# "residue (...)" to a rewrite), when the head has more uncertified
# translator functions than the base.
#
# An estimate: it can pass where the gate (`just ratchet`, UE-18) refuses.
# - The surface path reads no #[cfg(test)] module, and the MIR register the
#   gate reads does: a translator function added or edited in one (such as
#   uorust::mir::generate) counts there and not here. It warns when the
#   change touches one.
# - The few functions only the MIR path lifts, and the manifest certifies,
#   are residue here on both sides: editing one grows the gate by one and
#   this by none.
# - Each side is projected by its own source's binary, as `just ratchet`
#   does, only when the binaries are built here (no UORUST) or BASE_UORUST
#   names the base's. With UORUST alone both sides are projected by the
#   head's binary, which holds only while the change leaves the lifting and
#   κ as they were: a change of them moves the base's count with the head's.
#   It warns when the change touches crates/uorust/src.
# usage: scripts/hooks/translator-estimate.sh [BASE] [CARGO_ARG...]
#        UORUST=<binary> [BASE_UORUST=<binary>] scripts/hooks/translator-estimate.sh [BASE]
# Arguments after BASE go to cargo (a lane's `--config` flags), as
# ratchet-staged.sh takes them; UORUST names a built binary of the work
# tree instead, BASE_UORUST one of BASE. Without UORUST the base is built
# from its own source in target/translator-estimate/base-tree into the same
# target directory, as `just ratchet` builds its base.
set -euo pipefail
root=$(git rev-parse --show-toplevel)
cd "$root"
base=${1:-origin/main}
shift $(($# > 0 ? 1 : 0))
out=$root/target/translator-estimate
target=${CARGO_TARGET_DIR:-$root/target}
[[ $target == /* ]] || target=$root/$target
if [[ -n ${UORUST:-} ]]; then
  uorust=("$UORUST")
  base_uorust=("${BASE_UORUST:-$UORUST}")
else
  uorust=(cargo run -q --locked -p uorust "$@" --)
  base_uorust=(env -C "$out/base-tree" CARGO_TARGET_DIR="$target" cargo run -q --locked -p uorust "$@" --)
fi
rm -rf "$out" && mkdir -p "$out/base-tree"
trap 'rm -rf "$out/base-tree"' EXIT
git archive "$base" | tar -x -C "$out/base-tree"
changed=$(git diff --name-only "$base" -- crates/uorust/src)
if [[ -n ${UORUST:-} && -z ${BASE_UORUST:-} && -n $changed ]]; then
  echo "translator-estimate: the base is projected by the work tree's binary; this holds only while the change leaves the lifting and κ as they were (BASE_UORUST, or no UORUST, projects it by its own)" >&2
fi
# A changed file inside an out-of-line #[cfg(test)] module of the translator.
while IFS= read -r file; do
  dir=crates/uorust/src
  rest=${file#"$dir"/}
  while [[ $rest == */* || $rest == *.rs ]]; do
    name=${rest%%/*}
    name=${name%.rs}
    [[ $name == mod ]] && break
    declarer=$dir/mod.rs
    [[ -f $declarer ]] || declarer=$dir.rs
    [[ $dir == crates/uorust/src ]] && declarer=$dir/lib.rs
    if [[ -f $declarer ]] && grep -A1 -F '#[cfg(test)]' "$declarer" | grep -Eq "^[[:space:]]*(pub(\([a-z]+\))? )?mod $name;"; then
      echo "translator-estimate: $file is in a #[cfg(test)] module, which the gate's MIR register counts and this does not" >&2
      break
    fi
    [[ $rest == */* ]] || break
    dir=$dir/$name
    rest=${rest#*/}
  done
done < <(grep -E '^crates/uorust/src/(mir|lift|lex)(/|\.rs$)' <<<"$changed" || true)
# The command line of `just register`, without the MIR inputs.
project() {
  local bin=("${@:3}")
  "${bin[@]}" project "$1" --out "$out/$2" --name uorust --last-modified 2026-11-11T00:00:00Z --register "$out/$2.toml" >/dev/null
}
project . head "${uorust[@]}"
project "$out/base-tree" base "${base_uorust[@]}"
manifest=crates/certified/manifest.toml
"${uorust[@]}" certify ratchet --register "$out/head.toml" --manifest "$manifest" \
  --base-register "$out/base.toml" --base-manifest "$out/base-tree/$manifest"
