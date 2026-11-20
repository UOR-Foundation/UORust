#!/usr/bin/env bash
# Reproduce the complete compiler-library graph using the pinned compiler.
# Run inside the exact SDK, before the final production build. The explicit
# producer supplies the established checksum-verified source acquisition.
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
producer=${1:?an absolute preserved UORust producer is required}
case "$producer" in /*) ;; *) echo 'producer must be an absolute path' >&2; exit 1;; esac
work=${2:-"$repo/target/compiler-library-graph"}
channel=$(sed -n 's/^channel = "\(.*\)"/\1/p' "$repo/rust-toolchain.toml")
mkdir -p "$work/raw"
"$producer" library-mir core --modules --source-cache "$work/source" > "$work/modules.json"
RUSTC_BOOTSTRAP=1 rustc +"$channel" --crate-name core --edition=2024 \
  --crate-type=lib --emit=metadata --target=x86_64-unknown-linux-gnu \
  -Coverflow-checks=on -Cdebug-assertions=off -Zmir-include-spans=on \
  -Zdump-mir=ElaborateBoxDerefs -Zdump-mir-dir="$work/raw" \
  --out-dir="$work" "$work/source/library/core/src/lib.rs"
cd "$repo"
UORUST_CORE_SOURCE_ROOT="$work/source" \
UORUST_CORE_SOURCE_ARCHIVE="$work/source/rust-src.tar.xz" \
UORUST_COMPILER_LIBRARY_MIR="$work/raw" UORUST_WRITE_SPEC=1 \
  cargo test --config profile.test.package.uorust.opt-level=0 --locked -p uorust \
    --lib mir::link::compiler_library_tests::compiler_library_graph_reproduces_pinned_bodies_ue_104 \
    -- --exact --nocapture
