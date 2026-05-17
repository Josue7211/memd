#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MEMD_BIN="${MEMD_BIN:-memd}"
MEMD_OUTPUT="${MEMD_OUTPUT:-$ROOT/.memd}"
CLAWCONTROL_MEMD_OUTPUT="${CLAWCONTROL_MEMD_OUTPUT:-$ROOT/../clawcontrol/.memd}"
SOURCE_APP="${SOURCE_APP:-clawcontrol}"
DUE_WITHIN_SECS="${DUE_WITHIN_SECS:-300}"
ALLOW_STALE="${ALLOW_STALE:-0}"
CAPTURE_HTTP="${CAPTURE_HTTP:-1}"
CAPTURE_SCRIPT="${CAPTURE_SCRIPT:-$ROOT/scripts/live-state-capture-clawcontrol-http.mjs}"

if ! command -v "$MEMD_BIN" >/dev/null 2>&1; then
  if [[ -x "$ROOT/target/debug/memd" ]]; then
    MEMD_BIN="$ROOT/target/debug/memd"
  else
    echo "live-state-sync-clawcontrol: memd command not found" >&2
    exit 127
  fi
fi

if [[ "$CAPTURE_HTTP" == "1" || "$CAPTURE_HTTP" == "true" ]]; then
  if [[ ! -x "$CAPTURE_SCRIPT" ]]; then
    echo "live-state-sync-clawcontrol: capture script not executable: $CAPTURE_SCRIPT" >&2
    exit 127
  fi
  MEMD_BIN="$MEMD_BIN" MEMD_OUTPUT="$CLAWCONTROL_MEMD_OUTPUT" "$CAPTURE_SCRIPT"
fi

args=(
  live-state
  sync
  --output "$MEMD_OUTPUT"
  --from-output "$CLAWCONTROL_MEMD_OUTPUT"
  --source "$SOURCE_APP"
  --due-within-secs "$DUE_WITHIN_SECS"
)

if [[ "$ALLOW_STALE" == "1" || "$ALLOW_STALE" == "true" ]]; then
  args+=(--allow-stale)
fi

exec "$MEMD_BIN" "${args[@]}"
