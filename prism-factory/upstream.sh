#!/usr/bin/env bash
# The factory-upstream obligation: fetch the upstream revision upstream.toml
# names and hold every installed file of prism-factory/ to upstream's there
# (byte for byte, derived, reduced or relocked, as upstream.toml says). It
# reads the network, so it is a fetch obligation.
set -euo pipefail
repository=$(sed -n 's/^repository = "\(.*\)"$/\1/p' prism-factory/upstream.toml)
revision=$(sed -n 's/^revision = "\([0-9a-f]\{40\}\)"$/\1/p' prism-factory/upstream.toml)
[[ -n $repository && -n $revision ]] || { echo 'upstream.sh: upstream.toml names no repository and revision' >&2; exit 65; }
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
git -C "$work" init --quiet repository
git -C "$work/repository" fetch --quiet --depth=1 "$repository" "$revision"
git -C "$work/repository" archive FETCH_HEAD prism-factory | tar -x -C "$work"
python3 -I prism-factory/graph.py upstream "$work/prism-factory"
