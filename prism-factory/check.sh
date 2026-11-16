#!/usr/bin/env bash
# The factory-model obligation: the factory checks itself (upstream's
# check.sh, with the SDK's LexLean in place of a vendored one).
#  1. The graph's model text and table are what graph.py writes from
#     graph.toml and the Justfile, the graph is complete, and every installed
#     file is what upstream.toml records at the upstream revision.
#  2. The certified bundle replays without the source, Lean or the network,
#     and the request pins this compiler's rust-std descriptor.
#  3. The model is verified through the kernel and re-exported, and the export
#     must equal the committed bundle.
#  4. The host's tests pass: the certified core's identities against Python's
#     hashlib, and the routes of this graph.
# It runs in a directory that holds only this obligation's declared inputs.
set -euo pipefail
python3 -I prism-factory/graph.py check
lexlean production verify-artifact prism-factory/certified
targets=$(mktemp)
trap 'rm -f "$targets"' EXIT
lexlean production targets --diagnostic-format json >"$targets"
python3 -I prism-factory/graph.py request-check "$targets"
(cd prism-factory/std && lexlean lock --check)
(cd prism-factory/model && lexlean lock --check && lexlean verify --all)
result=$(cd prism-factory/model && lexlean production build --request ../production-request.json | sed -n 's/.*; result \([0-9a-f]\{64\}\);.*/\1/p')
# The attestation names the compiler binary by its hash, and a binary built
# in another directory has other bytes. Everything else must be identical.
diff -r --exclude=attestation.json --exclude=result.json "prism-factory/model/.lexlean/production/$result" prism-factory/certified
python3 -I - "prism-factory/model/.lexlean/production/$result" prism-factory/certified <<'PY'
import json, sys
def load(root, name):
    value = json.load(open(f"{root}/{name}"))
    def strip(item):
        if isinstance(item, dict):
            return {key: strip(entry) for key, entry in item.items()
                    if key not in ("executable_sha256", "attestation_id", "result_id")
                    and not (key == "sha256" and item.get("path", "").endswith("attestation.json"))}
        return [strip(entry) for entry in item] if isinstance(item, list) else item
    return strip(value)
fresh, committed = sys.argv[1], sys.argv[2]
for name in ("evidence/attestation.json", "result.json"):
    assert load(fresh, name) == load(committed, name), f"{name} differs beyond the compiler binary"
PY
cargo test --quiet --locked --offline --manifest-path prism-factory/host/Cargo.toml
