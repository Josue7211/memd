#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

tmp="$(mktemp -d "${TMPDIR:-/tmp}/memd-approved-communications.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

approved_empty="$tmp/approved-empty.json"
dry_output="$tmp/dry-output.json"
cat >"$approved_empty" <<'JSON'
{
  "messages": [],
  "email": []
}
JSON

DRY_RUN=1 \
APPROVED_COMMUNICATIONS_FILE="$approved_empty" \
"$ROOT/scripts/live-state-capture-approved-communications.mjs" >"$dry_output"

jq -e '.records | length == 2' "$dry_output" >/dev/null
jq -e '.records[] | select(.module == "messages" and .summary == "messages: approved metadata loaded; conversations=0")' "$dry_output" >/dev/null
jq -e '.records[] | select(.module == "email" and .summary == "email: approved metadata loaded; inbox_items=0")' "$dry_output" >/dev/null
jq -e 'all(.records[]; .approved == true and .visibility == "private" and .privacy == "metadata")' "$dry_output" >/dev/null

approved_empty_env="$tmp/approved-empty-env.json"
DRY_RUN=1 \
APPROVED_COMMUNICATIONS_EMPTY_APPROVED=1 \
"$ROOT/scripts/live-state-capture-approved-communications.mjs" >"$approved_empty_env"

jq -e '.records | length == 2' "$approved_empty_env" >/dev/null
jq -e '.records[] | select(.module == "messages" and .summary == "messages: approved metadata loaded; conversations=0")' "$approved_empty_env" >/dev/null
jq -e '.records[] | select(.module == "email" and .summary == "email: approved metadata loaded; inbox_items=0")' "$approved_empty_env" >/dev/null

raw_body="$tmp/raw-body.json"
cat >"$raw_body" <<'JSON'
{
  "messages": [
    {
      "approved": true,
      "contact": "Example",
      "body": "raw message body must not enter memd"
    }
  ]
}
JSON

set +e
DRY_RUN=1 \
APPROVED_COMMUNICATIONS_FILE="$raw_body" \
"$ROOT/scripts/live-state-capture-approved-communications.mjs" >"$tmp/raw.out" 2>"$tmp/raw.err"
raw_status=$?
set -e

if [[ "$raw_status" -eq 0 ]]; then
  echo "approved communications capture test: raw body was accepted" >&2
  exit 1
fi
grep -q 'raw communication field is not allowed' "$tmp/raw.err"

request_output="$tmp/request-output"
mkdir -p "$request_output/state"
set +e
MEMD_OUTPUT="$request_output" \
SOURCE_STATUS_OUTPUT="$request_output" \
"$ROOT/scripts/live-state-capture-approved-communications.mjs" >"$tmp/request.out" 2>"$tmp/request.err"
request_status=$?
set -e
if [[ "$request_status" -ne 2 ]]; then
  echo "approved communications capture test: missing approval did not block" >&2
  exit 1
fi
request_json="$request_output/state/approved-communications-request.json"
status_json="$request_output/state/live-app-source-status.json"
test -f "$request_json"
request_json="$(cd "$(dirname "$request_json")" && pwd)/$(basename "$request_json")"
jq -e '.schema == "memd.approved-communications-request.v1"' "$request_json" >/dev/null
jq -e '.status == "needs_user_or_process_approval"' "$request_json" >/dev/null
jq -e '.approval.set == "APPROVED_COMMUNICATIONS_FILE"' "$request_json" >/dev/null
jq -e '.privacyContract[] | select(. == "Raw chat/mail body text, HTML, transcripts, blobs, and raw media are rejected.")' "$request_json" >/dev/null
jq -e --arg path "$request_json" '.sources[] | select(.source_app == "approved_communications" and .approval_request_path == $path)' "$status_json" >/dev/null

echo "approved communications capture test: ok"
