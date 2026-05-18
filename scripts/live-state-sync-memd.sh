#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MEMD_BIN="${MEMD_BIN:-memd}"
MEMD_OUTPUT="${MEMD_OUTPUT:-$ROOT/.memd}"
DUE_WITHIN_SECS="${DUE_WITHIN_SECS:-300}"
MAC_BRIDGE_FALLBACK="${MAC_BRIDGE_FALLBACK:-1}"
MAC_BRIDGE_CAPTURE_SCRIPT="${MAC_BRIDGE_CAPTURE_SCRIPT:-$ROOT/scripts/live-state-capture-mac-bridge.mjs}"
MAC_BRIDGE_GUARD_SCRIPT="${MAC_BRIDGE_GUARD_SCRIPT:-$ROOT/scripts/memd-mac-bridge-guard.sh}"
MAC_BRIDGE_GUARD_ENABLED="${MAC_BRIDGE_GUARD_ENABLED:-1}"
APPROVED_COMMUNICATIONS_FALLBACK="${APPROVED_COMMUNICATIONS_FALLBACK:-1}"
APPROVED_COMMUNICATIONS_CAPTURE_SCRIPT="${APPROVED_COMMUNICATIONS_CAPTURE_SCRIPT:-$ROOT/scripts/live-state-capture-approved-communications.mjs}"
HOST_IO_GUARD="${HOST_IO_GUARD:-$ROOT/scripts/memd-host-io-guard.sh}"
HOST_IO_GUARD_ENABLED="${HOST_IO_GUARD_ENABLED:-1}"
DAEMON_MODE="${MEMD_LIVE_STATE_SYNC_DAEMON:-0}"
FALLBACK_CAPTURED=0

say() {
  printf 'live-state-sync-memd: %s\n' "$*" >&2
}

if [[ "$HOST_IO_GUARD_ENABLED" == "1" || "$HOST_IO_GUARD_ENABLED" == "true" ]]; then
  if [[ -x "$HOST_IO_GUARD" ]]; then
    "$HOST_IO_GUARD"
  else
    say "host I/O guard not executable: $HOST_IO_GUARD"
    exit 127
  fi
fi

if ! command -v "$MEMD_BIN" >/dev/null 2>&1; then
  if [[ -x "$ROOT/target/debug/memd" ]]; then
    MEMD_BIN="$ROOT/target/debug/memd"
  else
    say "memd command not found"
    exit 127
  fi
fi

if [[ "$MAC_BRIDGE_FALLBACK" == "1" || "$MAC_BRIDGE_FALLBACK" == "true" ]]; then
  if [[ -x "$MAC_BRIDGE_CAPTURE_SCRIPT" ]]; then
    if [[ "$MAC_BRIDGE_GUARD_ENABLED" == "1" || "$MAC_BRIDGE_GUARD_ENABLED" == "true" ]]; then
      if [[ -x "$MAC_BRIDGE_GUARD_SCRIPT" ]]; then
        "$MAC_BRIDGE_GUARD_SCRIPT" || true
      else
        say "mac-bridge guard not executable: $MAC_BRIDGE_GUARD_SCRIPT"
      fi
    fi
    set +e
    MEMD_BIN="$MEMD_BIN" MEMD_OUTPUT="$MEMD_OUTPUT" "$MAC_BRIDGE_CAPTURE_SCRIPT"
    bridge_status=$?
    set -e
    if [[ "$bridge_status" -eq 0 ]]; then
      FALLBACK_CAPTURED=1
      say "mac-bridge producer ran"
    elif [[ "$bridge_status" -eq 2 ]]; then
      say "mac-bridge producer unavailable"
    else
      exit "$bridge_status"
    fi
  else
    say "mac-bridge producer not executable: $MAC_BRIDGE_CAPTURE_SCRIPT"
  fi
fi

if [[ "$APPROVED_COMMUNICATIONS_FALLBACK" == "1" || "$APPROVED_COMMUNICATIONS_FALLBACK" == "true" ]]; then
  if [[ -x "$APPROVED_COMMUNICATIONS_CAPTURE_SCRIPT" ]]; then
    set +e
    MEMD_BIN="$MEMD_BIN" MEMD_OUTPUT="$MEMD_OUTPUT" "$APPROVED_COMMUNICATIONS_CAPTURE_SCRIPT"
    communications_status=$?
    set -e
    if [[ "$communications_status" -eq 0 ]]; then
      FALLBACK_CAPTURED=1
      say "approved communications producer captured messages/email metadata"
    elif [[ "$communications_status" -eq 2 ]]; then
      say "approved communications producer unavailable"
    else
      exit "$communications_status"
    fi
  else
    say "approved communications producer not executable: $APPROVED_COMMUNICATIONS_CAPTURE_SCRIPT"
  fi
fi

set +e
"$MEMD_BIN" live-state status --output "$MEMD_OUTPUT" --check --due-within-secs "$DUE_WITHIN_SECS" >/dev/null
status_check=$?
set -e

if [[ "$status_check" -eq 0 ]]; then
  if [[ "$FALLBACK_CAPTURED" == "1" ]]; then
    say "fallback records satisfy live-state requirements"
  fi
  "$MEMD_BIN" live-state status --output "$MEMD_OUTPUT"
  exit 0
fi

if [[ "$FALLBACK_CAPTURED" == "1" ]]; then
  say "fallback records captured; live-state still requires sync"
else
  say "memd-owned producers unavailable; live-state still requires approved producers"
fi
set +e
"$MEMD_BIN" live-state status --output "$MEMD_OUTPUT" --tasks
tasks_status=$?
set -e
if [[ "$tasks_status" -ne 0 ]]; then
  say "live-state task report exited $tasks_status"
fi
if [[ "$DAEMON_MODE" == "1" || "$DAEMON_MODE" == "true" ]]; then
  say "daemon mode: recorded live-state blockers; exiting cleanly so launchd keeps polling"
  exit 0
fi
exit "$status_check"
