#!/usr/bin/env bash
# Fetch the newest certified-manifest artifact (the last export's manifest)
# into target/certified-prior, so the export refuses before any build the
# roots it already refused, and plans its shards by its seconds per root.
# The artifact carries what a dropped push to main would have lost.
set -euo pipefail
: "${GH_TOKEN:?GH_TOKEN is required}"
mkdir -p target/certified-prior
# main's export reads main's manifests only; a pull request's branch reads
# its own newest, else main's (its land keeps one too).
newest() {
  { gh api "repos/$GITHUB_REPOSITORY/actions/artifacts?name=certified-manifest&per_page=20" || echo '{"artifacts":[]}'; } |
    jq -r --arg branch "$1" '[.artifacts[] | select((.expired | not) and .workflow_run.head_branch == $branch)] | sort_by(.created_at) | last | .id // empty' || true
}
branch=${GITHUB_REF_NAME:-main}
id=$(newest "$branch")
if [[ -z $id && $branch != main ]]; then
  id=$(newest main)
fi
if [[ -n $id ]]; then
  gh api "repos/$GITHUB_REPOSITORY/actions/artifacts/$id/zip" >target/certified-prior.zip
  unzip -q -o target/certified-prior.zip -d target/certified-prior
  rm target/certified-prior.zip
fi
ls target/certified-prior
