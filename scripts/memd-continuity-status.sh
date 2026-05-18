#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PREFLIGHT="${MEMD_CONTINUITY_PREFLIGHT:-$ROOT/scripts/deploy-memd-server-preflight.sh}"
CONFIG_PATH="${MEMD_CONTINUITY_CONFIG:-$ROOT/.memd/config.json}"
WAKE_PATH="${MEMD_CONTINUITY_WAKE:-$ROOT/.memd/wake.md}"
export MEMD_ALLOW_DIRTY_DEPLOY="${MEMD_ALLOW_DIRTY_DEPLOY:-1}"

printf 'memd continuity status\n'

if [[ -f "$WAKE_PATH" ]]; then
  wake_recovery="$(awk '/^- recovery voice=/ { print; exit }' "$WAKE_PATH" 2>/dev/null || true)"
  if [[ -n "$wake_recovery" ]]; then
    printf 'WAKE_RECOVERY=%s\n' "$wake_recovery"
  fi
fi

if [[ -f "$CONFIG_PATH" ]]; then
  json_array_values() {
    local key="$1"
    awk -v key="$key" '
      function collect(line, fields, n, i, value) {
        n = split(line, fields, "\"")
        for (i = 2; i <= n; i += 2) {
          value = fields[i]
          if (value == "") continue
          if (values != "") values = values ","
          values = values value
        }
      }
      $0 ~ "\"" key "\"[[:space:]]*:" {
        line = $0
        if (line !~ /\[/) next
        in_array = 1
        sub(/^.*\[/, "", line)
        if (line ~ /\]/) {
          sub(/\].*$/, "", line)
          collect(line)
          printed = 1
          print values
          exit
        }
        collect(line)
        next
      }
      in_array {
        line = $0
        if (line ~ /\]/) {
          sub(/\].*$/, "", line)
          collect(line)
          printed = 1
          print values
          exit
        }
        collect(line)
      }
      END { if (!printed) print values }
    ' "$CONFIG_PATH" 2>/dev/null || true
  }

  config_project="$(awk -F'"' '/"project"[[:space:]]*:/ { print $4; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
  config_namespace="$(awk -F'"' '/"namespace"[[:space:]]*:/ { print $4; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
  config_agent="$(awk -F'"' '/"agent"[[:space:]]*:/ { print $4; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
  config_session="$(awk -F'"' '/"session"[[:space:]]*:/ { print $4; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
  config_authority="$(awk -F'"' '/"authority"[[:space:]]*:/ { print $4; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
  config_hive_system="$(awk -F'"' '/"hive_system"[[:space:]]*:/ { print $4; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
  config_hive_role="$(awk -F'"' '/"hive_role"[[:space:]]*:/ { print $4; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
  config_hive_groups="$(json_array_values "hive_groups")"
  config_hive_project_enabled="$(awk -F'[:,]' '/"hive_project_enabled"[[:space:]]*:/ { gsub(/[[:space:]]/, "", $2); print $2; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
  config_authority_mode="$(awk -F'"' '/"authority_state"[[:space:]]*:/ { in_state = 1; next } in_state && /"mode"[[:space:]]*:/ { print $4; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
  config_shared_url="$(awk -F'"' '/"shared_base_url"[[:space:]]*:/ { print $4; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
  config_capabilities="$(json_array_values "capabilities")"
  config_blocked_capabilities="$(json_array_values "blocked_capabilities")"
  printf 'CONFIG_PROJECT=%s\n' "$config_project"
  printf 'CONFIG_NAMESPACE=%s\n' "$config_namespace"
  printf 'CONFIG_AGENT=%s\n' "$config_agent"
  printf 'CONFIG_SESSION=%s\n' "$config_session"
  printf 'CONFIG_AUTHORITY=%s\n' "$config_authority"
  printf 'CONFIG_HIVE_SYSTEM=%s\n' "$config_hive_system"
  printf 'CONFIG_HIVE_ROLE=%s\n' "$config_hive_role"
  printf 'CONFIG_HIVE_GROUPS=%s\n' "$config_hive_groups"
  printf 'CONFIG_HIVE_PROJECT_ENABLED=%s\n' "$config_hive_project_enabled"
  if [[ "$config_hive_project_enabled" == "true" ]]; then
    printf 'CONFIG_HIVE_PROJECT_ACTION=project_hive_enabled\n'
  else
    printf 'CONFIG_HIVE_PROJECT_ACTION=enable_project_hive_before_handoff\n'
  fi
  printf 'CONFIG_AUTHORITY_MODE=%s\n' "$config_authority_mode"
  printf 'CONFIG_SHARED_BASE_URL=%s\n' "$config_shared_url"
  printf 'CONFIG_CAPABILITIES=%s\n' "$config_capabilities"
  printf 'CONFIG_BLOCKED_CAPABILITIES=%s\n' "$config_blocked_capabilities"
fi

file_mtime_epoch() {
  local path="$1"
  stat -f '%m' "$path" 2>/dev/null || stat -c '%Y' "$path" 2>/dev/null || true
}

active_memd_binary="${MEMD_ACTIVE_MEMD_BINARY:-/Volumes/T7/node/bin/memd}"
active_memd_sources="${MEMD_ACTIVE_MEMD_SOURCE_PATHS:-crates/memd-client/src/hive/commands_runtime.rs crates/memd-client/src/render/render_summary.rs crates/memd-client/src/awareness/mod.rs crates/memd-client/src/runtime/recall/mod.rs crates/memd-client/src/hive/ops_runtime.rs}"
active_memd_binary_epoch=""
active_memd_source_epoch=0
active_memd_source_path=""
if [[ -e "$active_memd_binary" ]]; then
  active_memd_binary_epoch="$(file_mtime_epoch "$active_memd_binary")"
fi
for source_path in $active_memd_sources; do
  if [[ "$source_path" != /* ]]; then
    source_path="$ROOT/$source_path"
  fi
  if [[ -e "$source_path" ]]; then
    source_epoch="$(file_mtime_epoch "$source_path")"
    if [[ -n "$source_epoch" && "$source_epoch" -gt "$active_memd_source_epoch" ]]; then
      active_memd_source_epoch="$source_epoch"
      active_memd_source_path="$source_path"
    fi
  fi
done
active_memd_binary_state="unknown"
active_memd_binary_action="inspect_active_memd_binary"
if [[ ! -e "$active_memd_binary" ]]; then
  active_memd_binary_state="missing"
  active_memd_binary_action="rebuild_active_memd_after_host_guard_clear"
elif [[ -n "$active_memd_binary_epoch" && "$active_memd_source_epoch" -gt 0 ]]; then
  if [[ "$active_memd_binary_epoch" -lt "$active_memd_source_epoch" ]]; then
    active_memd_binary_state="stale"
    active_memd_binary_action="rebuild_active_memd_after_host_guard_clear"
  else
    active_memd_binary_state="current"
    active_memd_binary_action="none"
  fi
fi
printf 'ACTIVE_MEMD_BINARY=%s\n' "$active_memd_binary"
printf 'ACTIVE_MEMD_BINARY_STATE=%s\n' "$active_memd_binary_state"
printf 'ACTIVE_MEMD_BINARY_ACTION=%s\n' "$active_memd_binary_action"
printf 'ACTIVE_MEMD_BINARY_MTIME_EPOCH=%s\n' "$active_memd_binary_epoch"
printf 'ACTIVE_MEMD_SOURCE_NEWEST=%s\n' "$active_memd_source_path"
printf 'ACTIVE_MEMD_SOURCE_NEWEST_MTIME_EPOCH=%s\n' "$active_memd_source_epoch"

run_preflight_capture() {
  "$PREFLIGHT" 2>&1
}

set +e
preflight_output="$(run_preflight_capture)"
preflight_status=$?
set -e

preflight_action="$(printf '%s\n' "$preflight_output" | awk -F= '$1 == "MEMD_CODEBASE_LIVE_MAP_ACTION" { print $2; exit }')"
preflight_git_blockers="$(printf '%s\n' "$preflight_output" | awk -F= '$1 == "MEMD_GIT_STATUS_BLOCKERS" { print $2; exit }')"
host_guard_refresh_status=""
if [[ "${MEMD_CONTINUITY_AUTO_HOST_GUARD:-1}" != "0" \
  && "${MEMD_CONTINUITY_AUTO_HOST_GUARD:-1}" != "false" ]]; then
  if [[ "$preflight_action" == "refresh_host_guard_before_trusting_live_map" \
    || "$preflight_git_blockers" == *"project_hint=host-io-report"*"state=missing"* \
    || "$preflight_git_blockers" == *"project_hint=host-io-report"*"state=stale"* ]]; then
    host_guard="${MEMD_CONTINUITY_HOST_GUARD:-$ROOT/scripts/memd-host-io-guard.sh}"
    if [[ -x "$host_guard" ]]; then
      set +e
      host_guard_refresh_output="$("$host_guard" 2>&1)"
      host_guard_refresh_status=$?
      set -e
      set +e
      preflight_output="$(run_preflight_capture)"
      preflight_status=$?
      set -e
    else
      host_guard_refresh_status="missing"
      host_guard_refresh_output="host guard not executable: $host_guard"
    fi
  fi
fi
if [[ -n "$host_guard_refresh_status" ]]; then
  printf 'HOST_GUARD_REFRESH_EXIT=%s\n' "$host_guard_refresh_status"
fi

preflight_action="$(printf '%s\n' "$preflight_output" | awk -F= '$1 == "MEMD_CODEBASE_LIVE_MAP_ACTION" { print $2; exit }')"
host_report_for_live_map="${MEMD_HOST_IO_REPORT:-$ROOT/.memd/state/host-io-guard.txt}"
host_report_status_for_live_map=""
if [[ -f "$host_report_for_live_map" ]]; then
  host_report_status_for_live_map="$(awk -F= '$1 == "status" { print $2; exit }' "$host_report_for_live_map" 2>/dev/null || true)"
fi
live_map_refresh_status=""
if [[ "${MEMD_CONTINUITY_AUTO_LIVE_MAP:-1}" != "0" \
  && "${MEMD_CONTINUITY_AUTO_LIVE_MAP:-1}" != "false" \
  && "$preflight_action" == "refresh_host_guard_before_trusting_live_map" \
  && "$host_report_status_for_live_map" == "clear" ]]; then
  memd_for_live_map="$active_memd_binary"
  if [[ ! -x "$memd_for_live_map" ]]; then
    memd_for_live_map="$(command -v memd 2>/dev/null || true)"
  fi
  if [[ -n "$memd_for_live_map" ]]; then
    set +e
    MEMD_CODEBASE_LIVE_MAP_TTL_SECS=0 "$memd_for_live_map" awareness \
      --output "$ROOT/.memd" \
      --root "$ROOT" \
      --include-current \
      --summary >/dev/null 2>&1
    live_map_refresh_status=$?
    set -e
    set +e
    preflight_output="$(run_preflight_capture)"
    preflight_status=$?
    set -e
  else
    live_map_refresh_status="missing"
  fi
fi
if [[ -n "$live_map_refresh_status" ]]; then
  printf 'LIVE_MAP_REFRESH_EXIT=%s\n' "$live_map_refresh_status"
fi

printf 'PREFLIGHT_EXIT=%s\n' "$preflight_status"
printf '%s\n' "$preflight_output" | awk '
  /^MEMD_GIT_BRANCH=/ ||
  /^MEMD_GIT_COMMIT=/ ||
  /^MEMD_GIT_DIRTY=/ ||
  /^MEMD_AUTHORITY_STACK=/ ||
  /^MEMD_AUTHORITY_CONTAINER=/ ||
  /^MEMD_AUTHORITY_IMAGE_REPO=/ ||
  /^MEMD_AUTHORITY_NETWORK=/ ||
  /^MEMD_AUTHORITY_DATA_VOLUME=/ ||
  /^MEMD_AUTHORITY_PORT=/ ||
  /^MEMD_AUTHORITY_URL=/ ||
  /^MEMD_AUTHORITY_DEPLOY_CONTRACT=/ ||
  /^MEMD_AUTHORITY_IDENTITY_STATUS=/ ||
  /^MEMD_AUTHORITY_IDENTITY_BLOCKERS=/ ||
  /^MEMD_SERVER_STATUS=/ ||
  /^MEMD_SERVER_GIT_COMMIT=/ ||
  /^MEMD_SERVER_GIT_DIRTY=/ ||
  /^MEMD_SERVER_BENCHMARK_GATE=/ ||
  /^MEMD_SERVER_LATENCY_P95_MS=/ ||
  /^MEMD_CODEBASE_LIVE_MAP_STATE=/ ||
  /^MEMD_CODEBASE_LIVE_MAP_STATUS=/ ||
  /^MEMD_CODEBASE_LIVE_MAP_REREAD_REQUIRED=/ ||
  /^MEMD_CODEBASE_LIVE_MAP_AUTOSYNC=/ ||
  /^MEMD_CODEBASE_LIVE_MAP_UPDATED_AT=/ ||
  /^MEMD_CODEBASE_LIVE_MAP_FINGERPRINT=/ ||
  /^MEMD_CODEBASE_LIVE_MAP_AGE_SECS=/ ||
  /^MEMD_CODEBASE_LIVE_MAP_TTL_SECS=/ ||
  /^MEMD_CODEBASE_LIVE_MAP_FRESH=/ ||
  /^MEMD_CODEBASE_LIVE_MAP_ACTION=/ ||
  /^MEMD_GIT_STATUS_BLOCKERS=/ { print }
'

if [[ -n "${wake_recovery:-}" ]]; then
  wake_server_authority_blockers="$(
    printf '%s\n' "$wake_recovery" | awk '
      {
        key = "server_authority_blockers="
        start = index($0, key)
        if (start == 0) exit
        value = substr($0, start + length(key))
        pipe = index(value, " | ")
        if (pipe > 0) value = substr(value, 1, pipe - 1)
        print value
        exit
      }
    '
  )"
  wake_stale_fields=""
  current_server_status="$(printf '%s\n' "$preflight_output" | awk -F= '$1 == "MEMD_SERVER_STATUS" { print $2; exit }')"
  current_server_blockers="$(printf '%s\n' "$preflight_output" | awk -F= '$1 == "MEMD_SERVER_STATUS_BLOCKERS" { print $2; exit }')"
  if [[ -n "$wake_server_authority_blockers" \
    && "$wake_server_authority_blockers" != "none" \
    && "$current_server_status" == "ready" \
    && -z "$current_server_blockers" ]]; then
    wake_stale_fields="server_authority_blockers"
  fi
  if [[ -n "$wake_stale_fields" ]]; then
    printf 'WAKE_RECOVERY_STALE_FIELDS=%s\n' "$wake_stale_fields"
    printf 'WAKE_RECOVERY_ACTION=prefer_current_continuity_status_and_refresh_handoff\n'
  else
    printf 'WAKE_RECOVERY_STALE_FIELDS=\n'
  fi
fi

action="$(printf '%s\n' "$preflight_output" | awk -F= '$1 == "MEMD_CODEBASE_LIVE_MAP_ACTION" { print $2; exit }')"
if [[ -f "$CONFIG_PATH" ]]; then
  config_hive_project_enabled_for_action="$(awk -F'[:,]' '/"hive_project_enabled"[[:space:]]*:/ { gsub(/[[:space:]]/, "", $2); print $2; exit }' "$CONFIG_PATH" 2>/dev/null || true)"
else
  config_hive_project_enabled_for_action=""
fi
if [[ "$config_hive_project_enabled_for_action" != "true" ]]; then
  printf 'NEXT_CONTINUITY_ACTION=enable_project_hive_before_handoff\n'
elif [[ -n "$action" ]]; then
  printf 'NEXT_CONTINUITY_ACTION=%s\n' "$action"
elif printf '%s\n' "$preflight_output" | grep -q '^MEMD_GIT_STATUS_BLOCKERS='; then
  printf 'NEXT_CONTINUITY_ACTION=refresh_host_guard_before_broad_repo_work\n'
else
  printf 'NEXT_CONTINUITY_ACTION=inspect_preflight_output\n'
fi

live_map_path="$(printf '%s\n' "$preflight_output" | awk -F= '$1 == "MEMD_CODEBASE_LIVE_MAP_STATE" { print $2; exit }')"
if [[ -n "$live_map_path" && -f "$live_map_path" ]]; then
  if command -v node >/dev/null 2>&1; then
    live_map_changes="$(
      node -e '
        const fs = require("fs");
        const path = process.argv[1];
        const state = JSON.parse(fs.readFileSync(path, "utf8"));
        const diff = state.last_changes || {};
        const sample = ["added", "modified", "deleted"].flatMap((key) =>
          (diff[key] || []).slice(0, 3).map((path) => `${key}:${path}`)
        );
        process.stdout.write(
          `added:${diff.added_count || 0} modified:${diff.modified_count || 0} deleted:${diff.deleted_count || 0} baseline:${!!diff.baseline_available}`
        );
        if (sample.length) process.stdout.write(` sample=${sample.join("|")}`);
      ' "$live_map_path" 2>/dev/null || true
    )"
    printf 'LIVE_MAP_CHANGES=%s\n' "$live_map_changes"
  fi
  printf 'LIVE_MAP_BLOCKERS_SAMPLE='
  awk '
    /"blockers"[[:space:]]*:/ {
      if ($0 ~ /\[[[:space:]]*\]/) exit
      in_blockers = 1
      next
    }
    in_blockers && /\]/ { exit }
    in_blockers {
      gsub(/^[[:space:]]+"/, "")
      gsub(/",?[[:space:]]*$/, "")
      if ($0 != "") {
        if (seen > 0) printf " | "
        printf "%s", $0
        seen += 1
        if (seen >= 3) exit
      }
    }
  ' "$live_map_path"
  printf '\n'
fi

live_map_events="${MEMD_CODEBASE_LIVE_MAP_EVENTS:-$ROOT/.memd/state/codebase-live-map-events.ndjson}"
if [[ -f "$live_map_events" && -s "$live_map_events" ]]; then
  tail -3 "$live_map_events" 2>/dev/null | awk '
    NF > 0 {
      line = $0
      gsub(/[[:space:]]+/, " ", line)
      if (length(line) > 500) {
        line = substr(line, 1, 500) "..."
      }
      printf "LIVE_MAP_EVENT_%d=%s\n", NR, line
    }
  ' || true
fi

host_report="${MEMD_HOST_IO_REPORT:-$ROOT/.memd/state/host-io-guard.txt}"
if [[ -f "$host_report" ]]; then
  printf 'HOST_IO_REPORT=%s\n' "$host_report"
  awk '
    /^ts=/ || /^status=/ { print "HOST_IO_" toupper($0); next }
    /^repo=/ || /^pid=/ { next }
    NF > 0 && seen < 3 {
      printf "HOST_IO_BLOCKER_%d=%s\n", seen + 1, $0
      seen += 1
    }
  ' "$host_report"
fi

host_awareness="${MEMD_HOST_IO_AWARENESS:-$ROOT/.memd/state/host-io-awareness.txt}"
if [[ -f "$host_awareness" ]]; then
  printf 'HOST_IO_AWARENESS=%s\n' "$host_awareness"
  awk '
    /^ts=/ || /^status=/ || /^observation_count=/ || /^hard_blocker_count=/ { print "HOST_IO_AWARENESS_" toupper($0); next }
    /^repo=/ || /^pid=/ { next }
    NF > 0 && seen < 5 {
      printf "HOST_IO_OBSERVATION_%d=%s\n", seen + 1, $0
      seen += 1
    }
  ' "$host_awareness"
fi
