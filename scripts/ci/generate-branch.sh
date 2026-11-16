#!/usr/bin/env bash
# generate.yml's step for a pull request's branch: every writer run on the
# branch merged onto main, the result pushed to the branch as that merge,
# and the gate dispatched on it. The certified export is dispatched on the
# branch when the merged tree's translator has more uncertified functions
# than main's (the ratchet, UE-18, on the registers the writers just
# generated): its land pushes to the branch the same way and dispatches
# this again, which then gates what it landed.
# usage: BRANCH=<b> EVENT=<github.event_name> scripts/ci/generate-branch.sh SUBJECT "PATHS"
#
# Only a person's push (EVENT pull_request) dispatches the export, so the
# chain push -> generate -> certified -> generate -> bootstrap ends: a run
# certified dispatched (EVENT workflow_dispatch) dispatches the gate only.
# main's brand is not re-pinned here; the branch takes main's.
set -euo pipefail
subject=$1
paths=$2
branch=${BRANCH:?BRANCH is required}
event=${EVENT:?EVENT is required}
: "${GH_TOKEN:?GH_TOKEN is required}"
root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"
status=0
scripts/ci/merge-main.sh || status=$?
if [[ $status == 2 ]]; then
  exit 0
elif [[ $status != 0 ]]; then
  exit "$status"
fi
read -r main _head <"$(git rev-parse --git-dir)/branch-merge"
scripts/ci/sdk.sh 'just generate'
# The ratchet against main, on the register just generated for the merged
# tree and the manifest it holds (`just generate` built the binary). On a
# person's push a grown count dispatches the export, and the gate waits for
# what it lands rather than running red now.
grown=0
if [[ $event == pull_request ]]; then
  base=target/branch-base
  rm -rf "$base" && mkdir -p "$base"
  git show "$main:frontier.toml" >"$base/frontier.toml"
  git show "$main:crates/certified/manifest.toml" >"$base/manifest.toml"
  scripts/ci/sdk.sh "target/debug/uorust certify ratchet --register frontier.toml --manifest crates/certified/manifest.toml --base-register $base/frontier.toml --base-manifest $base/manifest.toml" || grown=1
fi
gate=(bootstrap.yml)
if ((grown)); then
  gate=()
fi
out=$(scripts/ci/push-branch.sh "$subject" "$paths" "${gate[@]}")
printf '%s\n' "$out"
if grep -q ' moved ' <<<"$out"; then
  exit 0
fi
if grep -q '^push: nothing changed' <<<"$out" && [[ $event == workflow_dispatch ]]; then
  # The head is a commit the workflow token pushed (certified's land), so
  # nothing has gated it yet.
  gh workflow run bootstrap.yml --repo "$GITHUB_REPOSITORY" --ref "$branch"
fi
if ((grown)); then
  gh workflow run certified.yml --repo "$GITHUB_REPOSITORY" --ref "$branch" -f land_branch=true
  echo "generate: more translator functions are uncertified than on main; dispatched the certified export on $branch" | tee -a "${GITHUB_STEP_SUMMARY:-/dev/null}"
fi
