#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

tmp="$(mktemp -d "${TMPDIR:-/tmp}/memd-supermemory-head-to-head.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

memd_report="$tmp/memd-external-public-scale-50.json"
artifact_dir="$tmp/supermemory-replays"
out_dir="$tmp/out"
mkdir -p "$out_dir"

cat >"$memd_report" <<'JSON'
{
  "status": "pass",
  "limit": 50,
  "rows": [
    {"dataset": "longmemeval", "accuracy": 0.94},
    {"dataset": "locomo", "accuracy": 0.91},
    {"dataset": "membench", "accuracy": 1.0},
    {"dataset": "convomem", "accuracy": 0.98}
  ]
}
JSON

for dataset in longmemeval locomo membench convomem; do
  mkdir -p "$artifact_dir/$dataset/latest"
  cat >"$artifact_dir/$dataset/latest/summary.json" <<JSON
{
  "status": "replayed",
  "accuracy": 0.50,
  "limit": 50,
  "limit_scope": "items",
  "command": "SUPERMEMORY_API_KEY=... scripts/bench-supermemory.py --benchmark $dataset --limit 50",
  "artifact_path": "$artifact_dir/$dataset/latest/"
}
JSON
done

access_route_json='{"status":"working","routes":[{"provider":"bitwarden","status":"installed","scope":"supermemory-api-key","secret_values_stored":false,"guidance":"Bitwarden is installed; unlock before use."}]}'

OUT_DIR="$out_dir" \
RUN_DATE=2099-01-01 \
MEMD_REPORT="$memd_report" \
SUPERMEMORY_REPLAYS="$artifact_dir" \
MEMD_ACCESS_ROUTE_JSON="$access_route_json" \
"$ROOT/scripts/verify/25-5-supermemory-head-to-head.sh" >/tmp/memd-supermemory-dir-pass.out

jq -e '.status == "pass"' "$out_dir/2099-01-01-supermemory-head-to-head.json" >/dev/null
jq -e '.rows | length == 4' "$out_dir/2099-01-01-supermemory-head-to-head.json" >/dev/null
jq -e 'all(.rows[]; .competitor_limit == 50 and .competitor_limit_scope == "items")' "$out_dir/2099-01-01-supermemory-head-to-head.json" >/dev/null

set +e
OUT_DIR="$out_dir" \
RUN_DATE=2099-01-02 \
MEMD_REPORT="$memd_report" \
SUPERMEMORY_REPLAYS="$tmp/missing-replays" \
SUPERMEMORY_REQUEST="$tmp/supermemory-request.json" \
MEMD_ACCESS_ROUTE_JSON="$access_route_json" \
"$ROOT/scripts/verify/25-5-supermemory-head-to-head.sh" >/tmp/memd-supermemory-missing.out
blocked_status=$?
set -e
if [[ "$blocked_status" -ne 2 ]]; then
  echo "supermemory head-to-head test: missing artifact did not block" >&2
  exit 1
fi
jq -e '.status == "blocked"' "$out_dir/2099-01-02-supermemory-head-to-head.json" >/dev/null
jq -e '.missing_requirements == ["supermemory_same_fixture_replay_artifact"]' "$out_dir/2099-01-02-supermemory-head-to-head.json" >/dev/null
request_path="$(cd "$(dirname "$tmp/supermemory-request.json")" && pwd)/supermemory-request.json"
jq -e '.supermemory_request_path == "'"$request_path"'"' "$out_dir/2099-01-02-supermemory-head-to-head.json" >/dev/null
jq -e '.schema == "memd.supermemory-replay-request.v1"' "$tmp/supermemory-request.json" >/dev/null
jq -e '.status == "needs_replay_artifact_or_process_credential"' "$tmp/supermemory-request.json" >/dev/null
jq -e '.approved_routes.process_env == "SUPERMEMORY_API_KEY"' "$tmp/supermemory-request.json" >/dev/null
jq -e '.same_fixture_contract.required_limit == 50' "$tmp/supermemory-request.json" >/dev/null
jq -e '.privacy_contract[] | select(. == "Do not store SUPERMEMORY_API_KEY in memd artifacts.")' "$tmp/supermemory-request.json" >/dev/null

echo "supermemory head-to-head test: ok"
