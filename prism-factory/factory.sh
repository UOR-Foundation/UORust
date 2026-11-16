#!/usr/bin/env bash
# The tool's entry to the Prism factory, on a workstation or any runner.
#
#   factory.sh routes            each obligation's route (proved, observe, refuted, blocked) and how many are left
#   factory.sh run [NAME...]     plan and run to the fixed point (only the NAMEs, when given)
#   factory.sh start [NAME...]   the same, detached: its log and one-line status beside the receipts
#   factory.sh status            the status line of the last detached run
#   factory.sh keep BRANCH       commit the store's receipts onto BRANCH of origin and push it
#   factory.sh plan|request|plan-request|reweigh [options]   the host CLI (see `prism-factory`)
#
# Decisions are made by the certified core in prism-factory/certified; this
# script only builds the host, moves receipts and reports what the plan says.
#
#   FACTORY_RECEIPTS  the receipt store, a directory that outlives the worktree
#                     and the target directory (default target/factory/receipts)
#   FACTORY_WORK      scratch for closures, outside any repository (default $TMPDIR/prism-factory)
#   FACTORY_PROFILE   the cargo profile the host is built with (default release)
#   FACTORY_SDK_CONTAINER, FACTORY_CARGO_TARGET_DIR   read by prism-factory/sdk.sh
set -euo pipefail
root=$(git rev-parse --show-toplevel)
cd "$root"
manifest=prism-factory/host/Cargo.toml
profile=${FACTORY_PROFILE:-release}
directory=$profile
[[ $profile == dev ]] && directory=debug
host=${CARGO_TARGET_DIR:-$root/prism-factory/host/target}/$directory/prism-factory
store=${FACTORY_RECEIPTS:-$root/target/factory/receipts}
work=${FACTORY_WORK:-${TMPDIR:-/tmp}/prism-factory}
mkdir -p "$store"
store=$(cd "$store" && pwd)

build() { cargo build --quiet --profile "$profile" --locked --offline --manifest-path "$manifest"; }

# The plan's routes, then their counts and how many are left (not proved).
routes() {
  "$host" plan --receipts "$store" | awk '
    $1 == "obligation" { print; count[$3]++; total++ }
    END {
      printf "routes proved %d observe %d refuted %d blocked %d\n", count["proved"], count["observe"], count["refuted"], count["blocked"]
      printf "left %d of %d\n", total - count["proved"], total
    }'
}

only() { local name; for name in "$@"; do printf -- '--only\n%s\n' "$name"; done; }

command=${1:-}
case "$command" in
  routes)
    build
    routes
    ;;
  run)
    shift
    build
    mapfile -t selected < <(only "$@")
    exec "$host" run --receipts "$store" --work "$work" "${selected[@]}"
    ;;
  start)
    shift
    build
    status=$store/status log=$store/factory.log
    if [[ -f $status ]] && read -r state pid _ <"$status" && [[ $state == running ]] && kill -0 "$pid" 2>/dev/null; then
      echo "factory: a run is in progress ($(cat "$status"))" >&2
      exit 75
    fi
    # The detached run writes `running PID` first and its counts last; it needs
    # no agent and no terminal, and an interrupted run loses at most the
    # obligation in flight, whose receipt is the one not yet written.
    nohup setsid bash -c '
      set -uo pipefail
      host=$1 store=$2 work=$3 status=$4 log=$5; shift 5
      echo "running $$ $(date -u +%Y-%m-%dT%H:%M:%SZ)" >"$status"
      "$host" run --receipts "$store" --work "$work" "$@" >"$log" 2>&1
      code=$?
      established=$(grep -cE "^factory: [^ ]+ (passed|failed)$" "$log" || true)
      counts=$("$host" plan --receipts "$store" | awk "\$1 == \"obligation\" { c[\$3]++ } END { printf \"proved %d refuted %d blocked %d observe %d\", c[\"proved\"], c[\"refuted\"], c[\"blocked\"], c[\"observe\"] }")
      echo "done $code established $established $counts" >"$status"
    ' factory-start "$host" "$store" "$work" "$status" "$log" $(only "$@") </dev/null >/dev/null 2>&1 &
    echo "factory: started; log $log, status $status"
    ;;
  status)
    cat "$store/status" 2>/dev/null || { echo "factory: no detached run has started (no $store/status)" >&2; exit 66; }
    ;;
  keep)
    # Upstream's receipts branch: the store's receipts added to the branch's
    # newest head as one commit, then pushed; another run may push first, so
    # retry. Pushing needs the repository owner's grant.
    branch=${2:?usage: factory.sh keep BRANCH}
    for attempt in 1 2 3 4 5; do
      index=$(mktemp); parent=()
      if git fetch --quiet --depth=1 origin "refs/heads/$branch" 2>/dev/null; then
        GIT_INDEX_FILE=$index git read-tree FETCH_HEAD; parent=(-p "$(git rev-parse FETCH_HEAD)")
      else
        rm -f "$index"
      fi
      for receipt in "$store"/*.receipt "$store"/weights; do
        [[ -e $receipt ]] || continue
        blob=$(git hash-object -w "$receipt")
        GIT_INDEX_FILE=$index git update-index --add --cacheinfo "100644,$blob,${receipt##*/}"
      done
      tree=$(GIT_INDEX_FILE=$index git write-tree); rm -f "$index"
      commit=$(git -c user.name=prism-factory -c user.email=prism-factory@users.noreply.github.com \
        commit-tree "$tree" "${parent[@]}" -m "Receipts for $(git rev-parse HEAD)")
      if git push --quiet origin "$commit:refs/heads/$branch"; then echo "factory: kept the receipts on $branch"; exit 0; fi
      sleep $((attempt * 5))
    done
    echo "factory: could not push receipts to $branch" >&2
    exit 1
    ;;
  plan|request|plan-request|reweigh)
    build
    shift
    exec "$host" "$command" --receipts "$store" "$@"
    ;;
  *)
    sed -n '2,10p' "$0" >&2
    exit 64
    ;;
esac
