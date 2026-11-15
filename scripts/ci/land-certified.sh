#!/usr/bin/env bash
# Land the certified translator on main while other commits keep moving it:
#
#     scripts/ci/land-certified.sh <deadline>
#
# The export's bundles (target/uorust-certified) are certificates of the
# functions of the commit it exported, each recorded with its κ
# (kappas.toml). When main has moved under what the sync reads, push.sh
# refuses the crate: its adapters are written for that commit's call sites.
# So the sync, which is cheap, is taken again on main's new head: each
# package kept only while its function's κ is still the exported one, the
# adapters written for the new tree and compiled. Then it is pushed again.
# Nothing lands when the sync refuses, or when main keeps moving until
# another sync would outrun the job's deadline (seconds since the epoch),
# timed by the sync before it: the next run exports again.
set -euo pipefail
deadline=$1
took=0
for attempt in $(seq 1 1000000); do
  status=0
  MOVED_EXIT=3 scripts/ci/push.sh "Rebuild the certified translator" \
    "crates/certified crates/uorust/src/certified.rs" certified.yml generate.yml || status=$?
  if [[ $status != 3 ]]; then
    exit "$status"
  fi
  if (( $(date +%s) + took > deadline )); then
    echo "land: another sync would outrun the job; the next run exports again"
    exit 0
  fi
  started=$(date +%s)
  # push.sh fetched main's head into upstream/main.
  git checkout -q --detach upstream/main
  echo "land: attempt $attempt takes the sync again on $(git rev-parse HEAD)"
  if ! scripts/ci/sdk.sh 'just certified-sync'; then
    echo "land: the sync on $(git rev-parse HEAD) refused; the next run exports again"
    exit 0
  fi
  took=$(( $(date +%s) - started ))
done
