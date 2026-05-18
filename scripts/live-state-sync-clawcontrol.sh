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
MAC_BRIDGE_FALLBACK="${MAC_BRIDGE_FALLBACK:-1}"
MAC_BRIDGE_CAPTURE_SCRIPT="${MAC_BRIDGE_CAPTURE_SCRIPT:-$ROOT/scripts/live-state-capture-mac-bridge.mjs}"
APPROVED_COMMUNICATIONS_FALLBACK="${APPROVED_COMMUNICATIONS_FALLBACK:-1}"
APPROVED_COMMUNICATIONS_CAPTURE_SCRIPT="${APPROVED_COMMUNICATIONS_CAPTURE_SCRIPT:-$ROOT/scripts/live-state-capture-approved-communications.mjs}"
CAPTURE_UNAVAILABLE=0
FALLBACK_CAPTURED=0
HOST_IO_GUARD="${HOST_IO_GUARD:-$ROOT/scripts/memd-host-io-guard.sh}"
HOST_IO_GUARD_ENABLED="${HOST_IO_GUARD_ENABLED:-1}"

if [[ "$HOST_IO_GUARD_ENABLED" == "1" || "$HOST_IO_GUARD_ENABLED" == "true" ]]; then
  if [[ -x "$HOST_IO_GUARD" ]]; then
    "$HOST_IO_GUARD"
  else
    echo "live-state-sync-clawcontrol: host I/O guard not executable: $HOST_IO_GUARD" >&2
    exit 127
  fi
fi

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
  AUTHORITY_MEMD_OUTPUT="$MEMD_OUTPUT"
  set +e
  MEMD_BIN="$MEMD_BIN" MEMD_OUTPUT="$CLAWCONTROL_MEMD_OUTPUT" SOURCE_STATUS_OUTPUT="$AUTHORITY_MEMD_OUTPUT" "$CAPTURE_SCRIPT"
  capture_status=$?
  set -e
  if [[ "$capture_status" -eq 2 ]]; then
    CAPTURE_UNAVAILABLE=1
    echo "live-state-sync-clawcontrol: capture unavailable; continuing with existing bundle records" >&2
    if [[ "$MAC_BRIDGE_FALLBACK" == "1" || "$MAC_BRIDGE_FALLBACK" == "true" ]]; then
      if [[ -x "$MAC_BRIDGE_CAPTURE_SCRIPT" ]]; then
        set +e
        MEMD_BIN="$MEMD_BIN" MEMD_OUTPUT="$MEMD_OUTPUT" "$MAC_BRIDGE_CAPTURE_SCRIPT"
        bridge_status=$?
        set -e
        if [[ "$bridge_status" -eq 0 ]]; then
          FALLBACK_CAPTURED=1
          echo "live-state-sync-clawcontrol: mac-bridge fallback captured visible_page/calendar/reminders/todos metadata" >&2
        elif [[ "$bridge_status" -eq 2 ]]; then
          echo "live-state-sync-clawcontrol: mac-bridge fallback unavailable" >&2
        else
          exit "$bridge_status"
        fi
      else
        echo "live-state-sync-clawcontrol: mac-bridge fallback script not executable: $MAC_BRIDGE_CAPTURE_SCRIPT" >&2
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
          echo "live-state-sync-clawcontrol: approved communications fallback captured messages/email metadata" >&2
        elif [[ "$communications_status" -eq 2 ]]; then
          echo "live-state-sync-clawcontrol: approved communications fallback unavailable" >&2
        else
          exit "$communications_status"
        fi
      else
        echo "live-state-sync-clawcontrol: approved communications fallback script not executable: $APPROVED_COMMUNICATIONS_CAPTURE_SCRIPT" >&2
      fi
    fi
  elif [[ "$capture_status" -ne 0 ]]; then
    exit "$capture_status"
  fi
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

if [[ "$CAPTURE_UNAVAILABLE" == "1" && "$FALLBACK_CAPTURED" == "1" ]]; then
  set +e
  "$MEMD_BIN" live-state status --output "$MEMD_OUTPUT" --check --due-within-secs "$DUE_WITHIN_SECS" >/dev/null
  status_check=$?
  set -e
  if [[ "$status_check" -ne 0 ]]; then
    echo "live-state-sync-clawcontrol: fallback records captured; live-state still requires sync" >&2
    "$MEMD_BIN" live-state status --output "$MEMD_OUTPUT" --tasks
    exit "$status_check"
  fi
  echo "live-state-sync-clawcontrol: fallback records satisfy live-state requirements" >&2
  "$MEMD_BIN" live-state status --output "$MEMD_OUTPUT"
  exit 0
fi

if [[ "$CAPTURE_UNAVAILABLE" == "1" ]]; then
  sync_stdout="$(mktemp)"
  sync_stderr="$(mktemp)"
  cleanup_sync_logs() {
    rm -f "$sync_stdout" "$sync_stderr"
  }
  trap cleanup_sync_logs EXIT
  set +e
  "$MEMD_BIN" "${args[@]}" >"$sync_stdout" 2>"$sync_stderr"
  sync_status=$?
  set -e
  if [[ "$sync_status" -ne 0 ]]; then
    sync_error_text="$(cat "$sync_stderr")"
    if [[ "$sync_error_text" == *"no live-state records imported"* ]]; then
      echo "live-state-sync-clawcontrol: source bundle has no fresh importable records" >&2
    else
      printf '%s\n' "$sync_error_text" >&2
    fi
    echo "live-state-sync-clawcontrol: no import completed after capture outage; live-state still requires sync" >&2
    "$MEMD_BIN" live-state status --output "$MEMD_OUTPUT" --tasks
    exit 2
  fi
  cat "$sync_stdout"
  cat "$sync_stderr" >&2
  exit 0
fi

exec "$MEMD_BIN" "${args[@]}"
