#!/usr/bin/env bash
set -euo pipefail

STATE_DIR="${MEMD_HOOK_STATE_DIR:-$HOME/.memd/hook_state}"
mkdir -p "$STATE_DIR"

INPUT="$(cat)"
SESSION_ID="$(
  printf '%s' "$INPUT" | python3 -c 'import json, sys; print(json.load(sys.stdin).get("session_id", "unknown"))' 2>/dev/null || printf 'unknown'
)"

echo "[$(date '+%H:%M:%S')] PRE-COMPACT memd save triggered for session $SESSION_ID" >> "$STATE_DIR/hook.log"

cat <<'HOOKJSON'
{
  "decision": "block",
  "reason": "COMPACTION IMMINENT. Persist everything important to memd before context is compressed. 1. checkpoint current task state, 2. write durable decisions/corrections/preferences/facts, 3. run memd hook spill --output .memd --stdin --apply for any compaction packet or turn-state delta, 4. then allow compaction."
}
HOOKJSON
