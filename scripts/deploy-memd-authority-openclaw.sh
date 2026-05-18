#!/usr/bin/env bash
# Build/deploy the shared memd authority without touching ClawControl-owned
# containers. Activation is intentionally conservative: if port 8787 is owned
# by a legacy clawcontrol-* container, this script stops before mutation.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

REMOTE="${MEMD_AUTHORITY_REMOTE:-openclaw-vm}"
CONTAINER="${MEMD_AUTHORITY_CONTAINER:-memd-authority}"
IMAGE_REPO="${MEMD_AUTHORITY_IMAGE_REPO:-memd-authority}"
PORT="${MEMD_AUTHORITY_PORT:-8787}"
NETWORK="${MEMD_AUTHORITY_NETWORK:-portainer_default}"
DATA_VOLUME="${MEMD_AUTHORITY_DATA_VOLUME:-memd_authority_data}"
MODE="${1:-build-only}"

case "$MODE" in
  build-only|activate) ;;
  *)
    cat >&2 <<MSG
usage: $0 [build-only|activate]

build-only  build the remote memd authority image, no container mutation
activate    start/replace only the $CONTAINER container
MSG
    exit 64
    ;;
esac

if [[ "$CONTAINER" == clawcontrol-* ]]; then
  cat >&2 <<MSG
refusing memd authority deploy: MEMD_AUTHORITY_CONTAINER=$CONTAINER is ClawControl-owned.
Use a memd-owned name such as memd-authority.
MSG
  exit 65
fi

if [[ "$IMAGE_REPO" == clawcontrol-* || "$IMAGE_REPO" == portainer-clawcontrol-* ]]; then
  cat >&2 <<MSG
refusing memd authority deploy: MEMD_AUTHORITY_IMAGE_REPO=$IMAGE_REPO is ClawControl-owned.
Use a memd-owned image repo such as memd-authority.
MSG
  exit 65
fi

preflight_output="$(MEMD_REQUIRE_SERVER_READY=0 scripts/deploy-memd-server-preflight.sh)"
branch="$(printf '%s\n' "$preflight_output" | awk -F= '$1 == "MEMD_GIT_BRANCH" { print $2; exit }')"
commit="$(printf '%s\n' "$preflight_output" | awk -F= '$1 == "MEMD_GIT_COMMIT" { print $2; exit }')"
dirty="$(printf '%s\n' "$preflight_output" | awk -F= '$1 == "MEMD_GIT_DIRTY" { print $2; exit }')"

if [[ -z "$commit" || "$commit" == "unknown" ]]; then
  echo "refusing memd authority deploy: unknown git commit" >&2
  exit 66
fi

image_tag="$IMAGE_REPO:$commit"

echo "building memd authority image on $REMOTE: $image_tag"
git archive --format=tar HEAD | ssh "$REMOTE" \
  "docker build -f deploy/docker/Dockerfile.memd-server \
    --build-arg MEMD_GIT_BRANCH='$branch' \
    --build-arg MEMD_GIT_COMMIT='$commit' \
    --build-arg MEMD_GIT_DIRTY='$dirty' \
    -t '$image_tag' -"

if [[ "$MODE" == "build-only" ]]; then
  cat <<MSG
MEMD_AUTHORITY_REMOTE=$REMOTE
MEMD_AUTHORITY_CONTAINER=$CONTAINER
MEMD_AUTHORITY_IMAGE=$image_tag
MEMD_AUTHORITY_ACTION=build_complete_no_runtime_change
MSG
  exit 0
fi

existing="$(
  ssh "$REMOTE" "docker ps -a --format '{{.Names}} {{.Ports}}' | awk -v port=':$PORT->' '\$0 ~ port { print } \$1 == \"$CONTAINER\" { print }'" || true
)"

if printf '%s\n' "$existing" | awk '{ print $1 }' | grep -q '^clawcontrol-'; then
  cat >&2 <<MSG
refusing memd authority activation: port/container is still owned by a ClawControl-named service.
$existing

Do the migration as an explicit infra step: create a memd-owned authority service
($CONTAINER), then point ClawControl at it as a consumer.
MSG
  exit 67
fi

if printf '%s\n' "$existing" | awk '{ print $1 }' | grep -v -x "$CONTAINER" | grep -q .; then
  cat >&2 <<MSG
refusing memd authority activation: port $PORT is owned by a non-memd container.
$existing
MSG
  exit 68
fi

echo "activating memd authority container on $REMOTE: $CONTAINER"
ssh "$REMOTE" "
  set -euo pipefail
  docker volume create '$DATA_VOLUME' >/dev/null
  docker rm -f '$CONTAINER' >/dev/null 2>&1 || true
  docker run -d \
    --name '$CONTAINER' \
    --restart unless-stopped \
    --network '$NETWORK' \
    -p '$PORT:8787' \
    -v '$DATA_VOLUME:/data' \
    -e MEMD_DB_PATH=/data/memd.db \
    -e MEMD_BIND_ADDR=0.0.0.0:8787 \
    -e MEMD_AUTHORITY_SEARCH=\"\${MEMD_AUTHORITY_SEARCH:-1}\" \
    -e MEMD_AUTHORITY_TOKEN=\"\${MEMD_AUTHORITY_TOKEN:-}\" \
    -e MEMD_GIT_BRANCH='$branch' \
    -e MEMD_GIT_COMMIT='$commit' \
    -e MEMD_GIT_DIRTY='$dirty' \
    '$image_tag'
"

cat <<MSG
MEMD_AUTHORITY_REMOTE=$REMOTE
MEMD_AUTHORITY_CONTAINER=$CONTAINER
MEMD_AUTHORITY_IMAGE=$image_tag
MEMD_AUTHORITY_ACTION=activated
MSG
