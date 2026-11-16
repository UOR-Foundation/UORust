#!/usr/bin/env bash
# Runs one command of an obligation inside the SDK image prismpm.lock selects,
# in the obligation's closure. The image is read from prismpm.lock, which every
# obligation run through here declares, so the image's digest is part of its
# identity. Where it runs is a place, not a decision:
#   - FACTORY_SDK_CONTAINER names a running container of exactly that image
#     (a workstation that keeps the SDK running, its cargo cache warm): the
#     command runs there by `docker exec`; a container of any other image is
#     refused, never used.
#   - otherwise the image runs fresh (`docker run`), as scripts/ci/sdk.sh does
#     on a runner.
# FACTORY_CARGO_TARGET_DIR, when set, is the cargo target directory the command
# builds in: a cache cargo keys by content, never an input.
#
# Inside the SDK the command runs with a cargo home of its own: each entry of
# the container's cargo home (the registry and git caches, their locks, the
# advisory databases) is linked into a fresh directory, except its
# configuration and credentials (config.toml, config, credentials.toml,
# credentials). Cargo reads that configuration in every build, no obligation
# declares it, and a container's cargo home is not part of its image, so it is
# left out rather than trusted. The obligation's names (FACTORY_OBLIGATION,
# FACTORY_KAPPA, FACTORY_REVISION) are passed in, so a check that must decide
# in a closure (UE-18) can tell that it runs in one.
set -euo pipefail
if [[ ${1:-} == --inside ]]; then
  cargo_home=${CARGO_HOME:-$HOME/.cargo}
  own=$(mktemp -d)
  trap 'rm -rf "$own"' EXIT
  shopt -s dotglob nullglob
  for entry in "$cargo_home"/*; do
    case ${entry##*/} in config | config.toml | credentials | credentials.toml) continue ;; esac
    ln -s "$entry" "$own/${entry##*/}"
  done
  shopt -u dotglob nullglob
  export CARGO_HOME=$own
  status=0
  bash -c "$FACTORY_SDK_COMMAND" || status=$?
  exit "$status"
fi
command=${1:?usage: prism-factory/sdk.sh COMMAND}
image=$(sed -n 's/.*"sdk_image":"\([^"]*\)".*/\1/p' prismpm.lock)
if [[ ! $image =~ ^[a-z0-9.-]+(:[0-9]{1,5})?/[a-z0-9./_-]+@sha256:[0-9a-f]{64}$ ]]; then
  echo 'prism-factory/sdk.sh: prismpm.lock does not select an immutable SDK image' >&2
  exit 64
fi
environment=(--env CARGO_INCREMENTAL=0 --env CARGO_PROFILE_DEV_DEBUG=0 --env CARGO_PROFILE_TEST_DEBUG=0 --env "FACTORY_SDK_COMMAND=$command")
if [[ -n ${FACTORY_CARGO_TARGET_DIR:-} ]]; then
  environment+=(--env "CARGO_TARGET_DIR=$FACTORY_CARGO_TARGET_DIR")
fi
for name in FACTORY_OBLIGATION FACTORY_KAPPA FACTORY_REVISION; do
  if [[ -n ${!name:-} ]]; then
    environment+=(--env "$name=${!name}")
  fi
done
if [[ -n ${FACTORY_SDK_CONTAINER:-} ]]; then
  running=$(docker inspect --format '{{.Config.Image}}' "$FACTORY_SDK_CONTAINER")
  if [[ $running != "$image" ]]; then
    printf 'prism-factory/sdk.sh: container %s runs %s, not the locked %s\n' "$FACTORY_SDK_CONTAINER" "$running" "$image" >&2
    exit 69
  fi
  exec docker exec --workdir "$PWD" "${environment[@]}" "$FACTORY_SDK_CONTAINER" bash -lc 'exec bash prism-factory/sdk.sh --inside'
fi
docker_config=$(mktemp -d)
trap 'rm -rf "$docker_config"' EXIT
printf '{}\n' >"$docker_config/config.json"
docker run --rm \
  --user "$(id -u):$(id -g)" \
  --tmpfs /tmp:rw,exec,nosuid,size=2g \
  --volume "$PWD:$PWD" \
  --volume "$docker_config:/tmp/prismpm-home/.docker:ro" \
  --workdir "$PWD" \
  --env HOME=/tmp/prismpm-home \
  --env PRISMPM_EPHEMERAL_HOME=1 \
  "${environment[@]}" \
  "$image" bash -c 'exec bash prism-factory/sdk.sh --inside'
