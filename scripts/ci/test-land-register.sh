#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
SOURCE="$ROOT/scripts/ci/land-register.sh"
# Fail when the real script loses exact child-status propagation.
grep -Fq 'exit "$census_status"' "$SOURCE" || { echo "missing nonzero census status propagation" >&2; exit 1; }
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/bin" "$WORK/scripts/ci" "$WORK/scripts/mir" "$WORK/crates/uorust/demo"
cat >"$WORK/bin/git" <<'SH'
#!/usr/bin/env bash
if [[ ${1:-} == rev-parse && ${2:-} == HEAD ]]; then echo aaaaaaaa; exit 0; fi
if [[ ${1:-} == checkout ]]; then exit 0; fi
printf 'unexpected git %s\n' "$*" >&2; exit 61
SH
cat >"$WORK/bin/python3" <<'SH'
#!/usr/bin/env bash
if [[ "${3:-}" == mir-key ]]; then
  counter=0; [[ ! -e .key-counter ]] || read -r counter < .key-counter
  counter=$((counter+1)); echo "$counter" > .key-counter
  if [[ ${KEY_MOVED:-0} == 1 && $counter -gt 1 ]]; then echo NEW; else echo OLD; fi
  exit 0
fi
if [[ "${3:-}" == record ]]; then echo "${4:-}" >> .recorded; exit 0; fi
echo "unexpected python3 $*" >&2; exit 62
SH
cat >"$WORK/scripts/ci/push.sh" <<'SH'
#!/usr/bin/env bash
counter=0; [[ ! -e .push-counter ]] || read -r counter < .push-counter
counter=$((counter+1)); echo "$counter" >.push-counter
if [[ $counter == 1 ]]; then exit 3; else exit 0; fi
SH
cat >"$WORK/scripts/ci/sdk.sh" <<'SH'
#!/usr/bin/env bash
printf 'sdk invoked\n' >> .sdk-log
exit "${SDK_EXIT:-0}"
SH
chmod +x "$WORK/bin"/* "$WORK/scripts/ci/"*.sh
printf 'channel = "1.97.1"\n' >"$WORK/rust-toolchain.toml"
printf 'sample = 1\n' >"$WORK/crates/uorust/demo/a.toml"
pass=0
case_test(){
  local name=$1 script=$2 sdk_exit=$3 key_moved=$4 expected=$5 deadline=$6
  rm -rf "$WORK/target"; rm -f "$WORK/.key-counter" "$WORK/.push-counter" "$WORK/.recorded" "$WORK/.sdk-log"
  cp "$SOURCE" "$WORK/scripts/ci/land-register.sh"
  if [[ $script == original ]]; then
    sed -i 's/exit "\$census_status"/exit 0/' "$WORK/scripts/ci/land-register.sh"
  fi
  chmod +x "$WORK/scripts/ci/land-register.sh"
  local status=0 output=""
  output=$(cd "$WORK" && PATH="$WORK/bin:$PATH" SDK_EXIT="$sdk_exit" KEY_MOVED="$key_moved" \
    bash scripts/ci/land-register.sh demo "$deadline" 2>&1) || status=$?
  if [[ $status != "$expected" ]]; then
    printf 'FAIL %-34s expected exit %s; got %s; output=%s\n' "$name" "$expected" "$status" "$output"
    exit 1
  fi
  if [[ $expected == 42 ]]; then
    [[ ! -e "$WORK/.recorded" ]] || { echo 'FAIL recorded a refused census'; exit 1; }
    [[ $output == *'failed with exit 42'* ]] || { echo 'FAIL missing intended error'; exit 1; }
  elif [[ $key_moved == 1 || $deadline == 0 ]]; then
    [[ ! -e "$WORK/.sdk-log" ]] || { echo 'FAIL ran census after deferral'; exit 1; }
  elif [[ $sdk_exit == 0 ]]; then
    [[ -e "$WORK/.recorded" ]] || { echo 'FAIL did not retain a valid census'; exit 1; }
  fi
  pass=$((pass+1))
  printf 'PASS %-34s exit %s\n' "$name" "$status"
}
case_test 'negative control: old failure swallowed' original 42 0 0 9999999999
case_test 'new failure propagates exact status' repaired 42 0 42 9999999999
case_test 'new positive census records' repaired 0 0 0 9999999999
case_test 'new moved key remains benign deferral' repaired 0 1 0 9999999999
case_test 'new deadline remains benign deferral' repaired 0 0 0 0
printf '%s isolated shell scenarios passed\n' "$pass"
