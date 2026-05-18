#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST_IO_GUARD="${HOST_IO_GUARD:-$ROOT/scripts/memd-host-io-guard.sh}"
if [ "${HOST_IO_GUARD_ENABLED:-1}" != "0" ] && [ "${HOST_IO_GUARD_ENABLED:-1}" != "false" ]; then
  "$HOST_IO_GUARD"
fi

bash scripts/sync-integration-hooks.sh

if ! git diff --quiet -- integrations/hooks/; then
  echo "integrations/hooks/ is out of sync with .memd/hooks/." >&2
  echo "Run scripts/sync-integration-hooks.sh and commit the result." >&2
  git diff --stat -- integrations/hooks/ >&2
  exit 1
fi

echo "hooks-lint: integrations/hooks/ in sync with .memd/hooks/"
