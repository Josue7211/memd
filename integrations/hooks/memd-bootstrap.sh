#!/usr/bin/env bash
# memd bootstrap hook — universal pre-turn enforcement.
# Runs memd wake and injects output as additional context.
# Skips if last wake was within MEMD_WAKE_TTL seconds (default 120).
# Degrades gracefully: serves stale bundle if backend unreachable.
set -euo pipefail

MEMD_WAKE_TTL="${MEMD_WAKE_TTL:-120}"

# Read hook input from stdin (harness sends JSON with session_id, cwd, etc.)
INPUT="$(cat)"
CWD="$(echo "$INPUT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('cwd',''))" 2>/dev/null || echo "")"
if [ -z "$CWD" ]; then
  CWD="$(pwd)"
fi

# Walk up to find .memd bundle
find_bundle() {
  local dir="$1"
  while [ "$dir" != "/" ]; do
    if [ -f "$dir/.memd/config.json" ]; then
      echo "$dir/.memd"
      return
    fi
    dir="$(dirname "$dir")"
  done
  echo ""
}

BUNDLE_ROOT="$(find_bundle "$CWD")"
if [ -z "$BUNDLE_ROOT" ]; then
  exit 0
fi

escape_json() {
  python3 -c "import sys,json; print(json.dumps(sys.stdin.read())[1:-1])"
}

# Staleness check
MARKER_FILE="$BUNDLE_ROOT/.last-wake"
if [ -f "$MARKER_FILE" ]; then
  LAST_WAKE="$(cat "$MARKER_FILE" 2>/dev/null || echo "0")"
  NOW="$(date +%s)"
  AGE=$(( NOW - LAST_WAKE ))
  if [ "$AGE" -lt "$MEMD_WAKE_TTL" ]; then
    if [ -f "$BUNDLE_ROOT/wake.md" ]; then
      CACHED="$(cat "$BUNDLE_ROOT/wake.md")"
      ESCAPED="$(echo "$CACHED" | escape_json)"
      printf '%s' "{\"additionalContext\":\"memd bootstrap (cached ${AGE}s ago):\\n${ESCAPED}\"}"
    fi
    exit 0
  fi
fi

# Run memd wake
WAKE_OUTPUT="$(memd wake --output "$BUNDLE_ROOT" --write 2>&1 || true)"

if [ -n "$WAKE_OUTPUT" ]; then
  date +%s > "$MARKER_FILE"
  ESCAPED="$(echo "$WAKE_OUTPUT" | escape_json)"
  printf '%s' "{\"additionalContext\":\"memd bootstrap (live):\\n${ESCAPED}\"}"
elif [ -f "$BUNDLE_ROOT/wake.md" ]; then
  CACHED="$(cat "$BUNDLE_ROOT/wake.md")"
  ESCAPED="$(echo "$CACHED" | escape_json)"
  printf '%s' "{\"additionalContext\":\"memd bootstrap (stale fallback — backend unreachable):\\n${ESCAPED}\"}"
fi
