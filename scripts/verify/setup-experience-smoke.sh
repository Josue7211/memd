#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MEMD_BIN="$(command -v memd || true)"
if [ -z "$MEMD_BIN" ] && [ -x "$ROOT/target/debug/memd" ]; then
  MEMD_BIN="$ROOT/target/debug/memd"
fi
if [ -z "$MEMD_BIN" ]; then
  echo "memd binary not found; run cargo build -p memd-client --bin memd" >&2
  exit 1
fi

TMP="$(mktemp -d)"
KEEP="${MEMD_SETUP_SMOKE_KEEP:-0}"
cleanup() {
  if [ "$KEEP" != "1" ]; then
    rm -rf "$TMP"
  else
    echo "setup smoke kept temp dir: $TMP"
  fi
}
trap cleanup EXIT

MEMD_OUTPUT="$TMP/.memd"
REPORT="$TMP/setup-experience-smoke.md"
RESUME_OUT="$TMP/resume.out"
export MEMD_OUTPUT
cd "$TMP"

{
  echo "# Setup Experience Smoke"
  echo
  echo "- date_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "- host: $(hostname 2>/dev/null || echo unknown)"
  echo "- os: $(uname -a)"
  echo "- memd_path: $MEMD_BIN"
  echo "- output: $MEMD_OUTPUT"
  echo
  echo "## Commands"
  echo
  echo '```text'
  echo "command -v memd"
  echo "$MEMD_BIN"
  echo
  echo "memd setup --output '$MEMD_OUTPUT' --summary --force --allow-localhost-read-only-fallback"
  "$MEMD_BIN" setup --output "$MEMD_OUTPUT" --summary --force --allow-localhost-read-only-fallback
  echo
  echo "memd setup --guided --summary"
  "$MEMD_BIN" setup --guided --summary | sed -n '1,40p'
  echo
  echo "memd setup-demo --summary"
  "$MEMD_BIN" setup-demo --summary
  echo
  echo "memd doctor --output '$MEMD_OUTPUT' --summary"
  "$MEMD_BIN" doctor --output "$MEMD_OUTPUT" --summary
  echo
  echo "memd status --output '$MEMD_OUTPUT' --summary"
  "$MEMD_BIN" status --output "$MEMD_OUTPUT" --summary
  echo
  echo "memd resume --output '$MEMD_OUTPUT' --intent current_task"
  "$MEMD_BIN" resume --output "$MEMD_OUTPUT" --intent current_task >"$RESUME_OUT"
  sed -n '1,40p' "$RESUME_OUT"
  echo '```'
  echo
  echo "## Result"
  echo
  echo "setup-experience-smoke=pass"
} | tee "$REPORT"

echo "setup smoke report: $REPORT"
