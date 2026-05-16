#!/usr/bin/env bash
# Emit deploy env for memd-server and block dirty authority builds by default.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

branch="$(git rev-parse --abbrev-ref HEAD)"
commit="$(git rev-parse --short HEAD)"
dirty="clean"
if [[ -n "$(git status --porcelain)" ]]; then
  dirty="dirty"
fi

if [[ "$dirty" != "clean" && "${MEMD_ALLOW_DIRTY_DEPLOY:-0}" != "1" ]]; then
  cat >&2 <<MSG
memd-server deploy blocked: working tree is dirty.
Commit or clean changes, then rerun.
To override for an explicit emergency deploy, set MEMD_ALLOW_DIRTY_DEPLOY=1.
MSG
  exit 2
fi

cat <<ENV
MEMD_GIT_BRANCH=$branch
MEMD_GIT_COMMIT=$commit
MEMD_GIT_DIRTY=$dirty
ENV

cat >&2 <<MSG
memd-server deploy env:
  MEMD_GIT_BRANCH=$branch
  MEMD_GIT_COMMIT=$commit
  MEMD_GIT_DIRTY=$dirty

Docker build example:
  docker build -f deploy/docker/Dockerfile.memd-server \\
    --build-arg MEMD_GIT_BRANCH=$branch \\
    --build-arg MEMD_GIT_COMMIT=$commit \\
    --build-arg MEMD_GIT_DIRTY=$dirty \\
    -t memd-server:$commit .
MSG
