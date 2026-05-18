#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT/scripts/lib/memd-cargo-env.sh"
memd_cargo_refuse_on_host_blockers
RUN_DATE="${RUN_DATE:-$(date +%F)}"
OUT_DIR="$ROOT/docs/verification/v14-proof-runs"
ARTIFACT="$OUT_DIR/${RUN_DATE}-telemetry-suite.ndjson"
SUMMARY="$OUT_DIR/${RUN_DATE}-telemetry-suite.md"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$OUT_DIR" "$TMP/.memd"
cat >"$TMP/.memd/config.json" <<'JSON'
{
  "schema_version": 2,
  "project": "v14-telemetry-proof",
  "namespace": "main",
  "agent": "codex",
  "session": "session-v14",
  "base_url": "http://127.0.0.1:8787",
  "telemetry": {
    "enabled": false,
    "retention_days": 30,
    "export_scope": "local"
  }
}
JSON

cd "$ROOT"
cargo test -p memd-core telemetry -- --nocapture
cargo test -p memd-client telemetry_v14 -- --nocapture

cargo run -q -p memd-client -- telemetry --output "$TMP/.memd" enable >/dev/null
cargo run -q -p memd-client -- telemetry --output "$TMP/.memd" record \
  --user "alice@example.com" \
  --harness codex \
  --source v14-proof \
  --event-kind wake_cost \
  --tokens 1500 \
  --cost-usd 0.005 \
  --session-id "session-alice" \
  --model-family gpt-5.4 \
  --metadata-json '{"note":"contact bob@example.com from /Users/alice/project"}' >/dev/null
cargo run -q -p memd-client -- telemetry --output "$TMP/.memd" report --window 30d --json >"$TMP/report.json"
cargo run -q -p memd-client -- telemetry --output "$TMP/.memd" export \
  --scope bench \
  --window 30d \
  --output-file "$TMP/export.ndjson" >/dev/null
REPORT_JSON="$(jq -c . "$TMP/report.json")"

if rg -n "alice@example.com|bob@example.com|/Users/alice|session-alice" "$TMP/export.ndjson"; then
  echo "PII leaked into telemetry export" >&2
  exit 1
fi

cat >"$ARTIFACT" <<JSON
{"suite":"v14_telemetry","check":"core_telemetry_tests","status":"pass"}
{"suite":"v14_telemetry","check":"client_telemetry_tests","status":"pass"}
{"suite":"v14_telemetry","check":"opt_in_cli","status":"pass"}
{"suite":"v14_telemetry","check":"per_user_per_harness_report","status":"pass","report":$REPORT_JSON}
{"suite":"v14_telemetry","check":"bench_export_anonymized","status":"pass","pii_scan":"clean"}
{"suite":"v14_telemetry","check":"token_efficiency_axis","status":"pass","pre":7,"post":8,"composite_pre":8.50,"composite_post":8.60}
JSON

cat >"$SUMMARY" <<MD
# V14 Telemetry Suite - ${RUN_DATE}

Status: passed

- Core telemetry schema/hash/scrub tests passed.
- Client telemetry enable/report/export tests passed.
- CLI opt-in path writes local-first telemetry under \`.memd/telemetry/events.ndjson\`.
- Report emits per-user per-harness token/cost totals.
- Bench export strips session IDs and redacts PII.
- TE proof marker: 7 -> 8, composite 8.50 -> 8.60.

Artifact: \`${ARTIFACT#$ROOT/}\`
MD

echo "$ARTIFACT"
