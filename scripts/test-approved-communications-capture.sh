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

echo "approved communications capture test: ok"
