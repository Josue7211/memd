#!/usr/bin/env bash

# Source this in memd scripts before running Cargo. Cargo's package cache is
# process-global by default, so separate repos can block each other even when
# their targets and worktrees are unrelated. memd scripts use a memd-owned
# Cargo home/target unless the caller deliberately overrides them.

MEMD_CARGO_HOME="${MEMD_CARGO_HOME:-${TMPDIR:-/tmp}/memd-cargo-home}"
MEMD_CARGO_TARGET_DIR="${MEMD_CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/memd-cargo-target}"
mkdir -p "$MEMD_CARGO_HOME" "$MEMD_CARGO_TARGET_DIR"

memd_cargo_volume_root_for_path() {
  local path="${1:-}"
  case "$path" in
    /Volumes/*)
      local rest="${path#/Volumes/}"
      printf '/Volumes/%s\n' "${rest%%/*}"
      ;;
  esac
}

memd_host_io_ps_timeout_secs() {
  local timeout="${MEMD_HOST_IO_PS_TIMEOUT_SECS:-2}"
  case "$timeout" in
    ''|*[!0-9]*) timeout=2 ;;
  esac
  if [[ "$timeout" -lt 1 ]]; then
    timeout=1
  fi
  printf '%s\n' "$timeout"
}

memd_host_io_report_ttl_secs() {
  local ttl="${MEMD_HOST_IO_REPORT_TTL_SECS:-120}"
  case "$ttl" in
    ''|*[!0-9]*) ttl=120 ;;
  esac
  printf '%s\n' "$ttl"
}

memd_host_io_report_path() {
  local repo_root="${1:-}"
  printf '%s\n' "${MEMD_HOST_IO_REPORT:-$repo_root/.memd/state/host-io-guard.txt}"
}

memd_host_io_awareness_path() {
  local repo_root="${1:-}"
  printf '%s\n' "${MEMD_HOST_IO_AWARENESS:-$repo_root/.memd/state/host-io-awareness.txt}"
}

memd_host_io_report_epoch() {
  local ts="${1:-}"
  if [[ -z "$ts" ]]; then
    return 1
  fi
  if [[ "$ts" == *.* ]]; then
    ts="${ts%%.*}Z"
  fi
  date -j -u -f '%Y-%m-%dT%H:%M:%SZ' "$ts" '+%s' 2>/dev/null \
    || date -u -d "$ts" '+%s' 2>/dev/null
}

memd_host_io_fresh_report_status() {
  local repo_root="${1:-}"
  local report
  report="$(memd_host_io_report_path "$repo_root")"
  [[ -f "$report" ]] || return 1

  local status ts report_epoch now_epoch age ttl
  status="$(awk -F= '$1 == "status" { print $2; exit }' "$report" 2>/dev/null)"
  [[ -n "$status" ]] || return 1
  ts="$(awk -F= '$1 == "ts" { print $2; exit }' "$report" 2>/dev/null)"
  report_epoch="$(memd_host_io_report_epoch "$ts")" || return 1
  now_epoch="$(date -u '+%s')"
  age=$((now_epoch - report_epoch))
  if [[ "$age" -lt 0 ]]; then
    age=0
  fi
  ttl="$(memd_host_io_report_ttl_secs)"
  if [[ "$age" -gt "$ttl" ]]; then
    return 1
  fi
  printf '%s\n' "$status"
}

memd_host_io_fresh_report_blockers() {
  local repo_root="${1:-}"
  local report
  report="$(memd_host_io_report_path "$repo_root")"
  [[ -f "$report" ]] || return 1

  local status ts
  status="$(awk -F= '$1 == "status" { print $2; exit }' "$report" 2>/dev/null)"
  [[ "$status" == "blocked" ]] || return 1
  ts="$(awk -F= '$1 == "ts" { print $2; exit }' "$report" 2>/dev/null)"
  local report_epoch now_epoch age ttl
  report_epoch="$(memd_host_io_report_epoch "$ts")" || return 1
  now_epoch="$(date -u '+%s')"
  age=$((now_epoch - report_epoch))
  if [[ "$age" -lt 0 ]]; then
    age=0
  fi
  ttl="$(memd_host_io_report_ttl_secs)"
  if [[ "$age" -gt "$ttl" ]]; then
    return 1
  fi

  printf 'repo project_hint=host-io-report pid=%s state=cached command=%s age_s=%s ttl_s=%s\n' "$$" "$report" "$age" "$ttl"
  awk '
    /^ts=/ || /^repo=/ || /^pid=/ || /^status=/ { next }
    /project_hint=host-io-report/ { next }
    NF > 0 { print }
  ' "$report"
}

memd_host_io_ps_snapshot() {
  if [[ -n "${MEMD_HOST_IO_PS_FILE:-}" ]]; then
    cat "$MEMD_HOST_IO_PS_FILE"
    return
  fi

  local repo_root="${1:-}"
  local volume_root="${2:-}"
  local timeout_s
  timeout_s="$(memd_host_io_ps_timeout_secs)"
  local tmp ps_pid deadline status
  tmp="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-ps.XXXXXX")" || return 1
  ps -axo pid,ppid,state,command >"$tmp" 2>/dev/null &
  ps_pid=$!
  deadline=$((SECONDS + timeout_s))
  while kill -0 "$ps_pid" 2>/dev/null; do
    if [[ "$SECONDS" -ge "$deadline" ]]; then
      kill "$ps_pid" 2>/dev/null || true
      kill -9 "$ps_pid" 2>/dev/null || true
      rm -f "$tmp"
      local scope="unknown"
      if [[ -n "$repo_root" ]]; then
        scope="repo"
      elif [[ -n "$volume_root" ]]; then
        scope="volume:$volume_root"
      fi
      printf '%s project_hint=host-process-scan pid=%s state=timeout command=ps -axo pid,ppid,state,command timeout_s=%s\n' "$scope" "$ps_pid" "$timeout_s"
      return 75
    fi
    sleep 0.05
  done
  wait "$ps_pid" 2>/dev/null
  status=$?
  if [[ "$status" -ne 0 ]]; then
    rm -f "$tmp"
    return "$status"
  fi
  cat "$tmp"
  rm -f "$tmp"
}

memd_cargo_host_blockers() {
  local repo_root="${MEMD_CARGO_REPO_ROOT:-}"
  if [[ -z "$repo_root" ]]; then
    repo_root="$(pwd -P 2>/dev/null || pwd)"
  fi
  local volume_root="${MEMD_CARGO_VOLUME_ROOT:-}"
  if [[ -z "$volume_root" ]]; then
    volume_root="$(memd_cargo_volume_root_for_path "$repo_root")"
  fi

  if [[ -z "${MEMD_HOST_IO_PS_FILE:-}" ]]; then
    local report_blockers
    report_blockers="$(memd_host_io_fresh_report_blockers "$repo_root" || true)"
    if [[ -n "$report_blockers" ]]; then
      printf '%s\n' "$report_blockers"
      return 0
    fi
  fi

  { memd_host_io_ps_snapshot "$repo_root" "$volume_root" 2>/dev/null || true; } | awk -v repo="$repo_root" -v volume="$volume_root" '
    /project_hint=host-process-scan/ { print; next }
    NR == 1 && $1 == "PID" { next }
    {
      pid = $1
      state = $3
      $1 = $2 = $3 = ""
      sub(/^[[:space:]]+/, "", $0)
      command = $0
      active_runtime = command ~ /(cargo[[:space:]]+tauri[[:space:]]+dev|tauri[[:space:]]+dev|npm[[:space:]]+run[[:space:]]+dev|node .*vite|\/vite(\.js)?([[:space:]]|$)|agent-shell-adapter\.js|clawctrl([[:space:]]|$))/
      filesystem = command ~ /(UVFSService|mds_stores|\/mds)/
      interesting = filesystem || command ~ /(^|[[:space:]\/])(git|cargo|rustc|rustfmt|clang|clang\+\+|cc|c\+\+)([[:space:]]|$)/ || command ~ /(vitest|tsc)/
      if (state !~ /U/ && !active_runtime) {
        next
      }
      if (active_runtime) {
        interesting = 1
      }
      if (!interesting) {
        next
      }
      scope = "unknown"
      project = "unknown"
      if (repo != "" && index(command, repo) > 0) {
        scope = "repo"
      } else if (volume != "" && index(command, volume) > 0) {
        scope = "volume:" volume
      } else if (filesystem && volume != "") {
        scope = "volume:" volume
      }
      if (scope == "unknown" && active_runtime) {
        scope = "unknown"
        project = "active-runtime"
      } else if (scope == "unknown") {
        next
      }
      project_marker = "/projects/"
      project_start = index(command, project_marker)
      if (project_start > 0) {
        project_tail = substr(command, project_start + length(project_marker))
        split(project_tail, project_parts, /[\/[:space:]\"\047]/)
        if (project_parts[1] != "") {
          project = project_parts[1]
        }
      } else if (filesystem) {
        project = "filesystem"
      } else if (command ~ /\/Xcode[^[:space:]]*\.app\// && command ~ /\/git([[:space:]]|$)/) {
        project = "app-git"
      } else if (command ~ /\/(cargo|rustc|rustfmt)([[:space:]]|$)/) {
        project = "cargo-tooling"
      } else if (command ~ /(^|[[:space:]\/])(clang|clang\+\+|cc|c\+\+)([[:space:]]|$)/) {
        project = "native-tooling"
      } else if (command ~ /(^|[[:space:]\/])(vitest|tsc)([[:space:]]|$)/) {
        project = "node-tooling"
      }
      if (length(command) > 240) {
        command = substr(command, 1, 240) "..."
      }
      reason = active_runtime && state !~ /U/ ? " reason=separate-existing-runtime" : ""
      printf "%s project_hint=%s pid=%s state=%s command=%s%s\n", scope, project, pid, state, command, reason
    }
  '
}

memd_host_io_hard_blockers() {
  local repo_root="${MEMD_CARGO_REPO_ROOT:-}"
  if [[ -z "$repo_root" ]]; then
    repo_root="$(pwd -P 2>/dev/null || pwd)"
  fi
  local repo_name
  repo_name="$(basename "$repo_root")"
  local block_scope="${MEMD_HOST_IO_BLOCK_SCOPE:-repo}"
  awk -v repo_name="$repo_name" -v block_scope="$block_scope" '
    NF == 0 { next }
    block_scope == "volume" { print; next }
    /project_hint=host-io-report/ { print; next }
    /project_hint=host-process-scan/ { print; next }
    /^repo[[:space:]]/ { print; next }
    /project_hint=filesystem/ { print; next }
    repo_name != "" && index($0, "project_hint=" repo_name) > 0 { print; next }
    /project_hint=app-git/ { print; next }
    /project_hint=cargo-tooling/ { print; next }
    /project_hint=native-tooling/ { print; next }
    /project_hint=node-tooling/ { print; next }
  '
}

memd_cargo_refuse_on_host_blockers() {
  if [[ "${MEMD_HOST_IO_GUARD:-1}" == "0" || "${MEMD_CARGO_BLOCK_ON_HOST_IO:-1}" == "0" ]]; then
    return 0
  fi
  local observations blockers
  observations="$(memd_cargo_host_blockers || true)"
  blockers="$(printf '%s\n' "$observations" | memd_host_io_hard_blockers)"
  memd_host_io_write_awareness "$observations" "$blockers"
  if ! grep -q 'project_hint=host-io-report' <<<"$observations"; then
    memd_host_io_write_report "$blockers"
  fi
  if [[ -z "$blockers" ]]; then
    if [[ -n "$observations" && "${MEMD_HOST_IO_SHOW_SIBLING_AWARENESS:-0}" == "1" ]]; then
      local label="${MEMD_HOST_IO_GUARD_LABEL:-memd cargo guard}"
      {
        echo "$label: separate existing app activity observed; continuing because it is not memd work."
        echo "$label: sibling activity is awareness only, not a memd test/build dependency."
        printf '%s\n' "$observations"
      } >&2
    fi
    return 0
  fi
  local label="${MEMD_HOST_IO_GUARD_LABEL:-memd cargo guard}"
  {
    echo "$label: host I/O blockers visible; refusing work."
    echo "$label: set MEMD_HOST_IO_GUARD=0 only for an intentional override."
    printf '%s\n' "$blockers"
  } >&2
  return 75
}

memd_host_io_write_live_map_event() {
  local repo_root="${1:-}"
  local report_path="${2:-}"
  local blockers="${3:-}"
  if [[ -n "${MEMD_HOST_IO_PS_FILE:-}" && -z "${MEMD_HOST_IO_LIVE_MAP_EVENTS:-}" ]]; then
    return 0
  fi
  if [[ -z "$repo_root" || -z "$report_path" ]]; then
    return 0
  fi

  local event_path="${MEMD_HOST_IO_LIVE_MAP_EVENTS:-$repo_root/.memd/state/codebase-live-map-events.ndjson}"
  local event_dir
  event_dir="$(dirname "$event_path")"
  mkdir -p "$event_dir" 2>/dev/null || return 0

  local status="clear"
  if [[ -n "$blockers" ]]; then
    status="blocked"
  fi

  MEMD_EVENT_TS="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
  MEMD_EVENT_SOURCE="host-io-guard:$status" \
  MEMD_EVENT_STATUS="$status" \
  MEMD_EVENT_PATH="$report_path" \
    awk '
      function json_quote(value) {
        gsub(/\\/, "\\\\", value)
        gsub(/"/, "\\\"", value)
        return "\"" value "\""
      }
      BEGIN {
        count = 0
      }
      NF > 0 {
        blockers[count] = $0
        count += 1
      }
      END {
        printf "{\"ts\":%s,\"source\":%s,\"status\":%s,\"paths\":[%s],\"blocker_count\":%d,\"blockers\":[",
          json_quote(ENVIRON["MEMD_EVENT_TS"]),
          json_quote(ENVIRON["MEMD_EVENT_SOURCE"]),
          json_quote(ENVIRON["MEMD_EVENT_STATUS"]),
          json_quote(ENVIRON["MEMD_EVENT_PATH"]),
          count
        for (i = 0; i < count && i < 5; i += 1) {
          if (i > 0) {
            printf ","
          }
          printf "%s", json_quote(blockers[i])
        }
        printf "]}\n"
      }
    ' >> "$event_path" 2>/dev/null <<< "$blockers" || true
}

memd_host_io_write_awareness_event() {
  local repo_root="${1:-}"
  local awareness_path="${2:-}"
  local status="${3:-}"
  local observations="${4:-}"
  if [[ -n "${MEMD_HOST_IO_PS_FILE:-}" && -z "${MEMD_HOST_IO_LIVE_MAP_EVENTS:-}" ]]; then
    return 0
  fi
  if [[ -z "$repo_root" || -z "$awareness_path" || -z "$observations" ]]; then
    return 0
  fi
  local event_dir event_path
  event_dir="${MEMD_HOST_IO_LIVE_MAP_EVENTS:-$repo_root/.memd/state/codebase-live-map-events.ndjson}"
  event_path="$event_dir"
  event_dir="$(dirname "$event_path")"
  mkdir -p "$event_dir" 2>/dev/null || return 0

  MEMD_EVENT_TS="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
  MEMD_EVENT_SOURCE="host-io-awareness:$status" \
  MEMD_EVENT_STATUS="$status" \
  MEMD_EVENT_PATH="$awareness_path" \
    awk '
      function json_quote(value) {
        gsub(/\\/, "\\\\", value)
        gsub(/"/, "\\\"", value)
        return "\"" value "\""
      }
      BEGIN {
        count = 0
      }
      NF > 0 {
        observations[count] = $0
        count += 1
      }
      END {
        printf "{\"ts\":%s,\"source\":%s,\"status\":%s,\"paths\":[%s],\"observation_count\":%d,\"observations\":[",
          json_quote(ENVIRON["MEMD_EVENT_TS"]),
          json_quote(ENVIRON["MEMD_EVENT_SOURCE"]),
          json_quote(ENVIRON["MEMD_EVENT_STATUS"]),
          json_quote(ENVIRON["MEMD_EVENT_PATH"]),
          count
        for (i = 0; i < count && i < 5; i += 1) {
          if (i > 0) {
            printf ","
          }
          printf "%s", json_quote(observations[i])
        }
        printf "]}\n"
      }
    ' >> "$event_path" 2>/dev/null <<< "$observations" || true
}

memd_host_io_seed_live_map_state() {
  local repo_root="${1:-}"
  local blockers="${2:-}"
  if [[ -z "$repo_root" ]]; then
    return 0
  fi
  if [[ -n "${MEMD_HOST_IO_PS_FILE:-}" && -z "${MEMD_CODEBASE_LIVE_MAP_STATE:-}" ]]; then
    return 0
  fi

  local state_path="${MEMD_CODEBASE_LIVE_MAP_STATE:-$repo_root/.memd/state/codebase-live-map.json}"
  if [[ -f "$state_path" ]]; then
    if ! grep -q '"fingerprint"[[:space:]]*:[[:space:]]*"host-io-\(blocked\|clear\)-no-scan"' "$state_path" 2>/dev/null; then
      return 0
    fi
  fi
  local state_dir tmp
  state_dir="$(dirname "$state_path")"
  mkdir -p "$state_dir" 2>/dev/null || return 0
  tmp="$(mktemp "$state_dir/.codebase-live-map.XXXXXX" 2>/dev/null)" || return 0

  local fingerprint="host-io-clear-no-scan"
  local status="out_of_sync"
  local autosync="host_io_clear_rescan_required"
  if [[ -n "$blockers" ]]; then
    fingerprint="host-io-blocked-no-scan"
    status="blocked"
    autosync="blocked_no_scan"
  fi

  MEMD_LIVE_MAP_TS="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
  MEMD_LIVE_MAP_REPO="$repo_root" \
  MEMD_LIVE_MAP_FINGERPRINT="$fingerprint" \
  MEMD_LIVE_MAP_STATUS="$status" \
  MEMD_LIVE_MAP_AUTOSYNC="$autosync" \
    awk '
      function json_quote(value) {
        gsub(/\\/, "\\\\", value)
        gsub(/"/, "\\\"", value)
        return "\"" value "\""
      }
      BEGIN {
        printf "{\n"
        printf "  \"repo_root\": %s,\n", json_quote(ENVIRON["MEMD_LIVE_MAP_REPO"])
        printf "  \"fingerprint\": %s,\n", json_quote(ENVIRON["MEMD_LIVE_MAP_FINGERPRINT"])
        printf "  \"file_count\": 0,\n"
        printf "  \"newest_mtime_unix\": 0,\n"
        printf "  \"updated_at\": %s,\n", json_quote(ENVIRON["MEMD_LIVE_MAP_TS"])
        printf "  \"status\": %s,\n", json_quote(ENVIRON["MEMD_LIVE_MAP_STATUS"])
        printf "  \"needs_reread\": true,\n"
        printf "  \"autosync\": %s,\n", json_quote(ENVIRON["MEMD_LIVE_MAP_AUTOSYNC"])
        printf "  \"blockers\": [\n"
      }
      NF > 0 {
        if (seen > 0) {
          printf ",\n"
        }
        printf "    %s", json_quote($0)
        seen += 1
      }
      END {
        printf "\n  ],\n"
        printf "  \"files\": {},\n"
        printf "  \"last_changes\": {\"added_count\":0,\"modified_count\":0,\"deleted_count\":0,\"added\":[],\"modified\":[],\"deleted\":[],\"truncated\":false}\n"
        printf "}\n"
      }
    ' > "$tmp" 2>/dev/null <<< "$blockers" \
    && mv "$tmp" "$state_path" 2>/dev/null || rm -f "$tmp"
}

memd_host_io_write_report() {
  local blockers="${1:-}"
  if [[ -n "${MEMD_HOST_IO_PS_FILE:-}" && -z "${MEMD_HOST_IO_REPORT:-}" ]]; then
    return 0
  fi
  local repo_root="${MEMD_CARGO_REPO_ROOT:-}"
  if [[ -z "$repo_root" ]]; then
    repo_root="$(pwd -P 2>/dev/null || pwd)"
  fi
  local output="${MEMD_HOST_IO_REPORT:-$repo_root/.memd/state/host-io-guard.txt}"
  local output_dir tmp
  output_dir="$(dirname "$output")"
  mkdir -p "$output_dir" 2>/dev/null || return 0
  tmp="$(mktemp "$output_dir/.host-io-guard.XXXXXX" 2>/dev/null)" || return 0
  {
    printf 'ts=%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    printf 'repo=%s\n' "$repo_root"
    printf 'pid=%s\n' "$$"
    if [[ -z "$blockers" ]]; then
      printf 'status=clear\n'
    else
      printf 'status=blocked\n'
      printf '%s\n' "$blockers"
    fi
  } > "$tmp" 2>/dev/null && mv "$tmp" "$output" 2>/dev/null && memd_host_io_write_live_map_event "$repo_root" "$output" "$blockers" && memd_host_io_seed_live_map_state "$repo_root" "$blockers" || rm -f "$tmp"
}

memd_host_io_write_awareness() {
  local observations="${1:-}"
  local blockers="${2:-}"
  if [[ -n "${MEMD_HOST_IO_PS_FILE:-}" && -z "${MEMD_HOST_IO_AWARENESS:-}" ]]; then
    return 0
  fi
  local repo_root="${MEMD_CARGO_REPO_ROOT:-}"
  if [[ -z "$repo_root" ]]; then
    repo_root="$(pwd -P 2>/dev/null || pwd)"
  fi
  local output
  output="$(memd_host_io_awareness_path "$repo_root")"
  local output_dir tmp status observation_count hard_blocker_count
  output_dir="$(dirname "$output")"
  mkdir -p "$output_dir" 2>/dev/null || return 0
  tmp="$(mktemp "$output_dir/.host-io-awareness.XXXXXX" 2>/dev/null)" || return 0
  if [[ -n "$blockers" ]]; then
    status="blocked"
  elif [[ -n "$observations" ]]; then
    status="observed"
  else
    status="clear"
  fi
  observation_count="$(printf '%s\n' "$observations" | awk 'NF > 0 { count += 1 } END { print count + 0 }')"
  hard_blocker_count="$(printf '%s\n' "$blockers" | awk 'NF > 0 { count += 1 } END { print count + 0 }')"
  {
    printf 'ts=%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    printf 'repo=%s\n' "$repo_root"
    printf 'pid=%s\n' "$$"
    printf 'status=%s\n' "$status"
    printf 'observation_count=%s\n' "$observation_count"
    printf 'hard_blocker_count=%s\n' "$hard_blocker_count"
    printf '%s\n' "$observations" | awk 'NF > 0 { print }'
  } > "$tmp" 2>/dev/null && mv "$tmp" "$output" 2>/dev/null && memd_host_io_write_awareness_event "$repo_root" "$output" "$status" "$observations" || rm -f "$tmp"
}

cargo() {
  memd_cargo_refuse_on_host_blockers || return $?
  command env \
    CARGO_HOME="$MEMD_CARGO_HOME" \
    CARGO_TARGET_DIR="$MEMD_CARGO_TARGET_DIR" \
    CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}" \
    CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}" \
    cargo "$@"
}
