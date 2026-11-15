#!/usr/bin/env bash
# Land a library's merged register on main while other commits keep moving
# it (UE-19):
#
#     scripts/ci/land-register.sh <crate> <deadline>
#
# The merge's register is a census of the commit its shards checked out,
# plus the verified columns (the bridge's stamps) measured there. When main
# has moved under one of the writers' inputs, push.sh refuses it: the
# census of a newer translator can differ, and a stale census on main is
# not this workspace's register. So the census, which is cheap, is taken
# again on main's new head, whole: translated, residue, embedded, blocked.
# The verified columns, which take the shards their hour, are carried onto
# it (`uorust carry`), each row's only while its κ, embedded term and
# translation's digest are those they were measured on, and marked with the
# commit they were measured at; every other translated row is pending. Then
# it is pushed again. Nothing lands when the census refuses (a translated
# row now residue), when the MIR's key moved (the dump is not the head's),
# or when main keeps moving until another census would outrun the job's
# deadline (seconds since the epoch), timed by the census before it: the
# next run regenerates.
set -euo pipefail
crate=$1
deadline=$2
register="crates/uorust/$crate"
measured=$(git rev-parse HEAD)
key=$(python3 -I scripts/mir/ledger.py mir-key)
rm -rf target/measured && mkdir -p target/measured && cp "$register"/*.toml target/measured/
push() {
  MOVED_EXIT=3 scripts/ci/push.sh "Regenerate the $crate register" "$register crates/uorust/ledger" generate.yml
}
took=0
for attempt in $(seq 1 1000000); do
  status=0
  push || status=$?
  if [[ $status != 3 ]]; then
    exit "$status"
  fi
  if (( $(date +%s) + took > deadline )); then
    echo "land: another census would outrun the job; the next run regenerates"
    exit 0
  fi
  started=$(date +%s)
  # push.sh fetched main's head into upstream/main.
  git checkout -q --detach upstream/main
  if [[ $(python3 -I scripts/mir/ledger.py mir-key) != "$key" ]]; then
    echo "land: the MIR's key moved at $(git rev-parse HEAD); its own run regenerates"
    exit 0
  fi
  echo "land: attempt $attempt takes the census again on $(git rev-parse HEAD)"
  channel=$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml)
  library="target/rust-src-$channel/library"
  # The census against main's own register, which it refuses to regress;
  # the embedding's census stamps each row; the verified columns are
  # carried; an empty merge rewrites the summary over them.
  rm -rf target/embed-head target/no-shards && mkdir -p target/no-shards
  if ! scripts/ci/sdk.sh "cargo build --locked -p uorust && scripts/mir/dump-lib.sh $crate --modules >/dev/null && target/debug/uorust $crate --mir target/mir --library $library --register $register && target/debug/uorust embed --crate $crate --mir target/mir --library $library --out target/embed-head --register $register && target/debug/uorust carry --from target/measured --register $register --measured $measured && target/debug/uorust merge --crate $crate --shards target/no-shards --register $register"; then
    echo "land: the census on $(git rev-parse HEAD) refused; its own run regenerates"
    exit 0
  fi
  python3 -I scripts/mir/ledger.py record "$register" target/shards
  took=$(( $(date +%s) - started ))
done
