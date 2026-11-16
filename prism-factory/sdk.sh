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
set -euo pipefail
command=${1:?usage: prism-factory/sdk.sh COMMAND}
image=$(sed -n 's/.*"sdk_image":"\([^"]*\)".*/\1/p' prismpm.lock)
if [[ ! $image =~ ^[a-z0-9.-]+(:[0-9]{1,5})?/[a-z0-9./_-]+@sha256:[0-9a-f]{64}$ ]]; then
  echo 'prism-factory/sdk.sh: prismpm.lock does not select an immutable SDK image' >&2
  exit 64
fi
environment=(--env CARGO_INCREMENTAL=0 --env CARGO_PROFILE_DEV_DEBUG=0 --env CARGO_PROFILE_TEST_DEBUG=0)
if [[ -n ${FACTORY_CARGO_TARGET_DIR:-} ]]; then
  environment+=(--env "CARGO_TARGET_DIR=$FACTORY_CARGO_TARGET_DIR")
fi
if [[ -n ${FACTORY_SDK_CONTAINER:-} ]]; then
  running=$(docker inspect --format '{{.Config.Image}}' "$FACTORY_SDK_CONTAINER")
  if [[ $running != "$image" ]]; then
    printf 'prism-factory/sdk.sh: container %s runs %s, not the locked %s\n' "$FACTORY_SDK_CONTAINER" "$running" "$image" >&2
    exit 69
  fi
  exec docker exec --workdir "$PWD" "${environment[@]}" "$FACTORY_SDK_CONTAINER" bash -lc "$command"
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
  "$image" bash -c "$command"
