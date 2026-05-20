#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/memd-unblock-status.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

approved="$tmp/approved-request.json"
supermemory="$tmp/supermemory-request.json"

cat >"$approved" <<'JSON'
{
  "schema": "memd.approved-communications-request.v1",
  "status": "needs_user_or_process_approval",
  "approval": {
    "requestPath": "/tmp/approved-request.json",
    "approvedFileTemplate": "/tmp/approved-template.json"
  },
  "missing": ["messages", "email"]
}
JSON

cat >"$supermemory" <<'JSON'
{
  "schema": "memd.supermemory-replay-request.v1",
  "status": "needs_replay_artifact_or_process_credential",
  "missing_requirements": ["supermemory_same_fixture_replay_artifact"],
  "approved_routes": {
    "run_replay": "TRY_REPLAY=1 scripts/verify/25-5-supermemory-head-to-head.sh",
    "provide_artifact": "SUPERMEMORY_REPLAYS=/path/to/supermemory-replays scripts/verify/25-5-supermemory-head-to-head.sh"
  },
  "bitwarden": {
    "status": "locked"
  },
  "same_fixture_contract": {
    "memd_report": "/tmp/memd-report.json",
    "replay_path": "/tmp/supermemory-replays.json",
    "required_limit": 10
  }
}
JSON

output="$(
  MEMD_APPROVED_COMMUNICATIONS_REQUEST="$approved" \
  MEMD_SUPERMEMORY_REPLAY_REQUEST="$supermemory" \
  "$ROOT/scripts/memd-unblock-status.sh"
)"

grep -q '^UNBLOCK_STATUS=blocked$' <<<"$output"
grep -q '^UNBLOCK_APPROVED_COMMUNICATIONS_MISSING=messages,email$' <<<"$output"
grep -q 'APPROVED_COMMUNICATIONS_FILE=/tmp/approved-template.json' <<<"$output"
grep -q '^UNBLOCK_SUPERMEMORY_MISSING=supermemory_same_fixture_replay_artifact$' <<<"$output"
grep -q '^UNBLOCK_SUPERMEMORY_REQUIRED_LIMIT=10$' <<<"$output"
grep -q 'TRY_REPLAY=1 scripts/verify/25-5-supermemory-head-to-head.sh' <<<"$output"
grep -q 'credential must stay process-local' <<<"$output"

clear_output="$(
  MEMD_APPROVED_COMMUNICATIONS_REQUEST="$tmp/missing-approved.json" \
  MEMD_SUPERMEMORY_REPLAY_REQUEST="$tmp/missing-supermemory.json" \
  "$ROOT/scripts/memd-unblock-status.sh"
)"
grep -q '^UNBLOCK_STATUS=clear$' <<<"$clear_output"

echo "memd unblock status test: ok"
