#!/usr/bin/env bash
# The tool's entry to the Prism factory, on a workstation or any runner.
#
#   factory.sh routes            each obligation's route (proved, observe, refuted, blocked) and how many are left
#   factory.sh run [NAME...]     plan and run to the fixed point (only the NAMEs, when given)
#   factory.sh start [NAME...]   the same, detached: its log and one-line status beside the receipts
#   factory.sh status            the status line of the last detached run (`stale` when its process is gone)
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
#   FACTORY_HOST      a host binary already built from prism-factory/host (the
#                     entry's own tests use it); the entry then builds nothing
#   FACTORY_SDK_CONTAINER, FACTORY_CARGO_TARGET_DIR   read by prism-factory/sdk.sh
set -euo pipefail
self=$(cd "$(dirname "$0")" && pwd)/$(basename "$0")
root=$(git rev-parse --show-toplevel)
cd "$root"
manifest=prism-factory/host/Cargo.toml
profile=${FACTORY_PROFILE:-release}
directory=$profile
[[ $profile == dev ]] && directory=debug
host=${FACTORY_HOST:-${CARGO_TARGET_DIR:-$root/prism-factory/host/target}/$directory/prism-factory}
store=${FACTORY_RECEIPTS:-$root/target/factory/receipts}
work=${FACTORY_WORK:-${TMPDIR:-/tmp}/prism-factory}
mkdir -p "$store"
store=$(cd "$store" && pwd)
status=$store/status log=$store/factory.log

build() {
  [[ -n ${FACTORY_HOST:-} ]] || cargo build --quiet --profile "$profile" --locked --offline --manifest-path "$manifest"
}

# A source of an obligation that names a tracked file the obligation does not
# declare is refused before anything is planned (`graph.py reads`): in the
# closure that read would fail, or, where the source tolerates a missing file,
# pass on other inputs than the identity covers.
reads() { python3 -I prism-factory/graph.py reads >/dev/null; }

# One run at a time per store. The lock is taken before the status is read or
# written and is held by the detached run and every process it starts, so two
# starts never both pass the check, never share closures and never write one
# log; a second one is refused while any process of the first is alive.
lock() {
  exec 9>>"$store/lock"
  if ! flock --nonblock 9; then
    echo "factory: a run is in progress ($(cat "$status" 2>/dev/null || echo 'no status yet'))" >&2
    exit 75
  fi
}

# The plan's routes, then their counts and how many are left (not proved).
routes() {
  "$host" plan --receipts "$store" | awk '
    $1 == "obligation" { print; count[$3]++; total++ }
    END {
      printf "routes proved %d observe %d refuted %d blocked %d\n", count["proved"], count["observe"], count["refuted"], count["blocked"]
      printf "left %d of %d\n", total - count["proved"], total
    }'
}

# Exit 1, naming them, when the plan holds a refuted obligation.
refuted() {
  "$host" plan --receipts "$store" | awk '
    $1 == "obligation" && $3 == "refuted" { print "factory: " $2 " is refuted on these inputs" > "/dev/stderr"; bad = 1 }
    END { exit bad }'
}

only() { local name; for name in "$@"; do printf -- '--only\n%s\n' "$name"; done; }

command=${1:-}
case "$command" in
  routes)
    build
    reads
    routes
    ;;
  run)
    shift
    build
    reads
    lock
    mapfile -t selected < <(only "$@")
    # Exit 75: an obligation's command was stopped by a signal and left no receipt.
    "$host" run --receipts "$store" --work "$work" "${selected[@]}"
    # A hermetic failure is refuted and is not run again, so a second run of a
    # refuted tree runs nothing; it still fails.
    refuted
    ;;
  start)
    shift
    build
    reads
    lock
    # The detached run inherits the lock (descriptor 9), writes `running PID`
    # first and its end last; it needs no agent and no terminal.
    nohup setsid bash "$self" detached "$host" "$store" "$work" $(only "$@") </dev/null >/dev/null 2>&1 &
    echo "factory: started; log $log, status $status"
    ;;
  detached)
    # The body of `start`, in its own session. Its end is one line:
    #   done CODE ...          the host ended by itself (CODE 1: a verdict failed or a refuted obligation remains)
    #   interrupted SIGNAL ... an obligation's command, or the host, or this run was stopped by SIGNAL;
    #                          the obligation in flight has no receipt and the next run runs it again
    # each followed by the counts. A run killed outright (SIGKILL of its session)
    # writes nothing, and `status` reports its `running` line as stale.
    host=$2 store=$3 work=$4
    shift 4
    set +e
    finish() {
      local established counts
      established=$(grep -cE "^factory: [^ ]+ (passed|failed)$" "$log")
      counts=$("$host" plan --receipts "$store" | awk '$1 == "obligation" { c[$3]++ } END { printf "proved %d refuted %d blocked %d observe %d", c["proved"], c["refuted"], c["blocked"], c["observe"] }')
      echo "$1 established $established $counts" >"$status"
    }
    trap 'finish "interrupted 1"; exit 129' HUP
    trap 'finish "interrupted 2"; exit 130' INT
    trap 'finish "interrupted 15"; exit 143' TERM
    echo "running $$ $(date -u +%Y-%m-%dT%H:%M:%SZ)" >"$status"
    "$host" run --receipts "$store" --work "$work" "$@" >"$log" 2>&1
    code=$?
    if ((code > 128)); then
      finish "interrupted $((code - 128))"
    elif ((code == 75)); then
      finish "interrupted $(sed -n 's/^factory: [^ ]* interrupted by signal \([0-9]*\);.*/\1/p' "$log" | tail -n 1)"
    else
      if [[ $code == 0 ]] && "$host" plan --receipts "$store" | grep -qE "^obligation [^ ]+ refuted "; then code=1; fi
      finish "done $code"
    fi
    ;;
  status)
    [[ -f $status ]] || { echo "factory: no detached run has started (no $status)" >&2; exit 66; }
    read -r state pid _ <"$status"
    if [[ $state == running ]] && ! kill -0 "$pid" 2>/dev/null; then
      echo "stale $(cat "$status"): that process is gone without writing its end, so the run was interrupted; the next run runs only what has no receipt"
      exit 1
    fi
    cat "$status"
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
