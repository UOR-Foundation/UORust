# Sourced by the hooks: the lane's cargo flags (the brief's LANE_CFG) in
# `cfg`, and the shared target. The lane's N is $LANE_N when set, else one
# derived from the worktree's path; every workspace member is built with it,
# since repo-model reads its own CARGO_MANIFEST_DIR and a reused rlib could
# check another lane's tree. Intermediates share one build dir; the final
# artifacts (the uorust binary the gates run) land in the lane's own target
# dir, so a gate never runs a binary another lane just rebuilt.
root=$(git rev-parse --show-toplevel)
N=${LANE_N:-$(( $(printf '%s' "$root" | cksum | cut -d' ' -f1) % 200 + 30 ))}
cfg=()
for p in uorust repo-model repo-conformance xtask; do
  cfg+=(--config "profile.dev.package.$p.codegen-units=$N" --config "profile.dev.package.$p.debug=0")
done
export CARGO_BUILD_BUILD_DIR=${CARGO_BUILD_BUILD_DIR:-/home/user/uorust-engine/target}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/home/user/uorust-engine/lanes/$N} CARGO_INCREMENTAL=0
