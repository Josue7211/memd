#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MEMD_BIN="${MEMD_BIN:-memd}"
if ! command -v "$MEMD_BIN" >/dev/null 2>&1; then
  if [[ -x "$ROOT/target/debug/memd" ]]; then
    MEMD_BIN="$ROOT/target/debug/memd"
  else
    echo "dev-server-guard: memd command not found" >&2
    exit 127
  fi
fi

exec "$MEMD_BIN" dev-server guard "$@"
