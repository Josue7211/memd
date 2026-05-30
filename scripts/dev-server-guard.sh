#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MEMD_BIN="${MEMD_BIN:-memd}"
HOST_IO_GUARD="${HOST_IO_GUARD:-$ROOT/scripts/memd-host-io-guard.sh}"
HOST_IO_GUARD_ENABLED="${HOST_IO_GUARD_ENABLED:-1}"

if [[ "$HOST_IO_GUARD_ENABLED" == "1" || "$HOST_IO_GUARD_ENABLED" == "true" ]]; then
  if [[ -x "$HOST_IO_GUARD" ]]; then
    "$HOST_IO_GUARD"
  else
    echo "dev-server-guard: host I/O guard not executable: $HOST_IO_GUARD" >&2
    exit 127
  fi
fi

if ! command -v "$MEMD_BIN" >/dev/null 2>&1; then
  if [[ -x "$ROOT/target/debug/memd" ]]; then
    MEMD_BIN="$ROOT/target/debug/memd"
  else
    echo "dev-server-guard: memd command not found" >&2
    exit 127
  fi
fi

if [[ "${MEMD_ALLOW_CLAWCONTROL_DEV_SERVER:-0}" != "1" \
  && "${MEMD_ALLOW_CLAWCONTROL_DEV_SERVER:-false}" != "true" ]]; then
  repo_root="$ROOT"
  repo_name="$(basename "$repo_root")"
  command_text="$*"
  if [[ "$repo_name" != "clawcontrol" && "$command_text" == *"clawcontrol"* ]]; then
    {
      echo "dev-server-guard: refusing to launch ClawControl from memd."
      echo "dev-server-guard: memd and ClawControl are separate; use ClawControl's own repo/session to start it."
      echo "dev-server-guard: set MEMD_ALLOW_CLAWCONTROL_DEV_SERVER=1 only for an intentional override."
    } >&2
    exit 66
  fi
fi

exec "$MEMD_BIN" dev-server guard "$@"
