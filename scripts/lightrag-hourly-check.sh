#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${LIGHTRAG_BASE_URL:-http://10.30.30.152:9621}"
MONITOR_DIR="${LIGHTRAG_MONITOR_DIR:-/home/josue/Documents/projects/memd/.monitor}"
LOG_FILE="${MONITOR_DIR}/lightrag-hourly.log"
STATE_FILE="${MONITOR_DIR}/lightrag-last-status.json"
TMP_FILE="${MONITOR_DIR}/.lightrag-status.tmp"
FAIL_ALERT_DELTA="${FAIL_ALERT_DELTA:-3}"
STALL_ALERT_HOURS="${STALL_ALERT_HOURS:-2}"

mkdir -p "$MONITOR_DIR"

health_json="$(curl -fsS "$BASE_URL/health")"
counts_json="$(curl -fsS "$BASE_URL/documents/status_counts")"
pipeline_json="$(curl -fsS "$BASE_URL/documents/pipeline_status")"

prev_failed=0
prev_processed=0
prev_stall_count=0
if [[ -f "$STATE_FILE" ]]; then
  prev_failed="$(jq -r '.failed // 0' "$STATE_FILE" 2>/dev/null || echo 0)"
  prev_processed="$(jq -r '.processed // 0' "$STATE_FILE" 2>/dev/null || echo 0)"
  prev_stall_count="$(jq -r '.stall_count // 0' "$STATE_FILE" 2>/dev/null || echo 0)"
fi

jq -n \
  --arg ts "$(date -Iseconds)" \
  --arg base_url "$BASE_URL" \
  --argjson health "$health_json" \
  --argjson counts "$counts_json" \
  --argjson pipeline "$pipeline_json" \
  --argjson prev_failed "$prev_failed" \
  --argjson prev_processed "$prev_processed" \
  --argjson prev_stall_count "$prev_stall_count" \
  '
  ($counts.status_counts.processed // 0) as $processed |
  ($health.pipeline_busy // false) as $busy |
  ((if $busy and ($processed <= $prev_processed) then ($prev_stall_count + 1) else 0 end)) as $stall_count |
  {
    timestamp: $ts,
    base_url: $base_url,
    failed: ($counts.status_counts.failed // 0),
    pending: ($counts.status_counts.pending // 0),
    processed: $processed,
    processing: ($counts.status_counts.processing // 0),
    all: ($counts.status_counts.all // 0),
    failed_delta: (($counts.status_counts.failed // 0) - $prev_failed),
    processed_delta: ($processed - $prev_processed),
    stall_count: $stall_count,
    pipeline_busy: ($health.pipeline_busy // false),
    llm_model: ($health.configuration.llm_model // null),
    embedding_model: ($health.configuration.embedding_model // null),
    latest_message: ($pipeline.latest_message // ""),
    cur_batch: ($pipeline.cur_batch // 0),
    docs: ($pipeline.docs // 0)
  }
  ' > "$TMP_FILE"

cat "$TMP_FILE" >> "$LOG_FILE"
printf '\n' >> "$LOG_FILE"
mv "$TMP_FILE" "$STATE_FILE"

failed_delta="$(jq -r '.failed_delta' "$STATE_FILE")"
failed_total="$(jq -r '.failed' "$STATE_FILE")"
processed_total="$(jq -r '.processed' "$STATE_FILE")"
pending_total="$(jq -r '.pending' "$STATE_FILE")"
processed_delta="$(jq -r '.processed_delta' "$STATE_FILE")"
stall_count="$(jq -r '.stall_count' "$STATE_FILE")"

if command -v notify-send >/dev/null 2>&1; then
  if (( failed_delta >= FAIL_ALERT_DELTA )); then
    notify-send "LightRAG Hourly Check" "Failures increased by ${failed_delta}. Failed=${failed_total}, Processed=${processed_total}, Pending=${pending_total}"
  elif (( stall_count >= STALL_ALERT_HOURS )); then
    notify-send "LightRAG Hourly Check" "Processing appears stalled for ${stall_count} checks. Failed=${failed_total}, Processed=${processed_total}, Pending=${pending_total}"
  elif (( failed_delta > 0 )); then
    notify-send "LightRAG Hourly Check" "Minor failure increase: +${failed_delta}. Failed=${failed_total}, Processed=${processed_total}, Pending=${pending_total}"
  else
    notify-send "LightRAG Hourly Check" "Healthy. Failed=${failed_total}, Processed=${processed_total} (${processed_delta:+$processed_delta}), Pending=${pending_total}"
  fi
fi
