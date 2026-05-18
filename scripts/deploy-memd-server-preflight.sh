#!/usr/bin/env bash
# Emit deploy env for memd-server and block dirty authority builds by default.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
if [[ -f "$ROOT/scripts/lib/memd-cargo-env.sh" ]]; then
  # Reuse bounded host I/O guard helpers so deploy preflight does not add
  # another stuck Git process while the shared volume is already blocked.
  # shellcheck source=scripts/lib/memd-cargo-env.sh
  source "$ROOT/scripts/lib/memd-cargo-env.sh"
fi

git_head="$(cat .git/HEAD 2>/dev/null || true)"
branch="unknown"
commit="unknown"
if [[ "$git_head" == ref:\ refs/heads/* ]]; then
  branch="${git_head#ref: refs/heads/}"
  ref_path=".git/refs/heads/$branch"
  if [[ -f "$ref_path" ]]; then
    commit="$(cut -c1-8 "$ref_path")"
  elif [[ -f .git/packed-refs ]]; then
    commit="$(awk -v ref="refs/heads/$branch" '$2 == ref {print substr($1, 1, 8); exit}' .git/packed-refs)"
    commit="${commit:-unknown}"
  fi
elif [[ -n "$git_head" ]]; then
  commit="$(printf '%s' "$git_head" | cut -c1-8)"
fi

git_status_blockers=""
if [[ "${MEMD_SKIP_GIT_STATUS:-0}" == "1" || "${MEMD_SKIP_GIT_STATUS:-0}" == "true" ]]; then
  dirty="unknown"
elif declare -F memd_host_io_fresh_report_status >/dev/null 2>&1 \
  && [[ -n "$(memd_cargo_volume_root_for_path "$ROOT")" ]] \
  && [[ "${MEMD_DEPLOY_ALLOW_GIT_STATUS_WITHOUT_HOST_REPORT:-0}" != "1" ]]; then
  host_report_status="$(memd_host_io_fresh_report_status "$ROOT" || true)"
  if [[ "$host_report_status" == "blocked" ]]; then
    git_status_blockers="$(memd_host_io_fresh_report_blockers "$ROOT" || true)"
    dirty="unknown"
  elif [[ "$host_report_status" == "clear" ]]; then
    dirty="$(
      python3 - <<'PY'
import os
import subprocess
import sys

timeout = float(os.environ.get("MEMD_GIT_STATUS_TIMEOUT", "5"))
try:
    result = subprocess.run(
        ["git", "status", "--porcelain"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        timeout=timeout,
    )
except subprocess.TimeoutExpired:
    print("unknown")
    sys.exit(0)
except OSError:
    print("unknown")
    sys.exit(0)

if result.returncode != 0:
    print("unknown")
elif result.stdout.strip():
    print("dirty")
else:
    print("clean")
PY
    )"
  else
    report_path="$(memd_host_io_report_path "$ROOT")"
    report_state="missing"
    report_timing=""
    if [[ -f "$report_path" ]]; then
      report_state="stale"
      report_ts="$(awk -F= '$1 == "ts" { print $2; exit }' "$report_path" 2>/dev/null || true)"
      if [[ -n "$report_ts" ]]; then
        report_epoch="$(memd_host_io_report_epoch "$report_ts" 2>/dev/null || true)"
        if [[ -n "$report_epoch" ]]; then
          now_epoch="$(date -u '+%s')"
          report_age=$((now_epoch - report_epoch))
          if [[ "$report_age" -lt 0 ]]; then
            report_age=0
          fi
          report_ttl="$(memd_host_io_report_ttl_secs)"
          report_timing=" age_s=$report_age ttl_s=$report_ttl"
        fi
      fi
    fi
    git_status_blockers="repo project_hint=host-io-report pid=$$ state=$report_state command=$report_path${report_timing} action=run scripts/memd-host-io-guard.sh before deploy preflight"
    dirty="unknown"
  fi
else
  dirty="$(
    python3 - <<'PY'
import os
import subprocess
import sys

timeout = float(os.environ.get("MEMD_GIT_STATUS_TIMEOUT", "5"))
try:
    result = subprocess.run(
        ["git", "status", "--porcelain"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        timeout=timeout,
    )
except subprocess.TimeoutExpired:
    print("unknown")
    sys.exit(0)
except OSError:
    print("unknown")
    sys.exit(0)

if result.returncode != 0:
    print("unknown")
elif result.stdout.strip():
    print("dirty")
else:
    print("clean")
PY
  )"
fi

if [[ "$dirty" != "clean" && "${MEMD_ALLOW_DIRTY_DEPLOY:-0}" != "1" ]]; then
  cat >&2 <<MSG
memd-server deploy blocked: working tree is $dirty.
Commit or clean changes, then rerun.
To override for an explicit emergency deploy, set MEMD_ALLOW_DIRTY_DEPLOY=1.
MSG
  if [[ -n "$git_status_blockers" ]]; then
    cat >&2 <<MSG

host I/O blockers prevented git status:
$git_status_blockers
MSG
  fi
  exit 2
fi

status_url="${MEMD_SERVER_STATUS_URL:-}"
authority_container="${MEMD_AUTHORITY_CONTAINER:-memd-authority}"
authority_image_repo="${MEMD_AUTHORITY_IMAGE_REPO:-memd-authority}"
authority_port="${MEMD_AUTHORITY_PORT:-${MEMD_AUTHORITY_MIGRATION_PORT:-8788}}"
authority_public_host="${MEMD_AUTHORITY_PUBLIC_HOST:-100.104.154.24}"
authority_url="http://$authority_public_host:$authority_port"
authority_deploy_contract="$ROOT/docs/contracts/memd-authority-deploy.md"
authority_identity_status="ready"
authority_identity_blockers=""
if [[ "$authority_container" == clawcontrol-* ]]; then
  authority_identity_status="blocked"
  authority_identity_blockers="MEMD_AUTHORITY_CONTAINER=$authority_container is ClawControl-owned; use memd-authority"
fi
if [[ "$authority_image_repo" == clawcontrol-* || "$authority_image_repo" == portainer-clawcontrol-* ]]; then
  if [[ -n "$authority_identity_blockers" ]]; then
    authority_identity_blockers+=" | "
  fi
  authority_identity_status="blocked"
  authority_identity_blockers+="MEMD_AUTHORITY_IMAGE_REPO=$authority_image_repo is ClawControl-owned; use memd-authority"
fi
if [[ "${MEMD_SKIP_SERVER_STATUS:-0}" == "1" || "${MEMD_SKIP_SERVER_STATUS:-0}" == "true" ]]; then
  status_url=""
elif [[ -z "$status_url" && -f ".memd/config.json" ]]; then
  status_url="$(
    awk -F'"' '
      /"shared_base_url"[[:space:]]*:/ && $4 != "" { print $4 "/api/status"; found = 1; exit }
      /"base_url"[[:space:]]*:/ && base == "" && $4 != "" { base = $4 }
      END { if (!found && base != "") print base "/api/status" }
    ' .memd/config.json 2>/dev/null
  )"
fi

server_status="unavailable"
server_git_commit=""
server_git_dirty=""
server_benchmark_gate=""
server_latency_p95_ms=""
server_blockers=""

if [[ "${MEMD_SKIP_SERVER_STATUS:-0}" == "1" || "${MEMD_SKIP_SERVER_STATUS:-0}" == "true" ]]; then
  server_status="skipped"
  server_blockers="server status probe skipped by MEMD_SKIP_SERVER_STATUS"
elif [[ -n "$status_url" ]]; then
  status_timeout="${MEMD_SERVER_STATUS_TIMEOUT:-3}"
  status_payload="$(curl -fsS --max-time "$status_timeout" "$status_url" 2>/tmp/memd-server-status-probe.err || true)"
  if [[ -z "$status_payload" ]]; then
    status_error="$(cat /tmp/memd-server-status-probe.err 2>/dev/null || true)"
    probe_output="$(
      printf 'status=unavailable\n'
      printf 'blockers=status probe failed or timed out after %ss%s\n' "$status_timeout" "${status_error:+: $status_error}"
    )"
  else
    server_commit="$(printf '%s' "$status_payload" | tr -d '\n' | sed -n 's/.*"git_commit"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
    server_dirty="$(printf '%s' "$status_payload" | tr -d '\n' | sed -n 's/.*"git_dirty"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
    gate="$(printf '%s' "$status_payload" | tr -d '\n' | sed -n 's/.*"benchmark_gate"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
    latency_text="$(printf '%s' "$status_payload" | tr -d '\n' | sed -n 's/.*"latency_p95_ms"[[:space:]]*:[[:space:]]*\([^,}]*\).*/\1/p')"
    blockers=()
    if [[ -n "$server_commit" && "$server_commit" != "$commit" ]]; then
      blockers+=("server git_commit=$server_commit does not match local HEAD $commit")
    fi
    if [[ -n "$server_dirty" && "$server_dirty" != "clean" ]]; then
      blockers+=("server_git_dirty=$server_dirty")
    fi
    if [[ "$gate" != "pass" && "$gate" != "acceptable" ]]; then
      suffix=""
      if [[ -n "$latency_text" ]]; then
        suffix=" latency_p95_ms=$latency_text"
      fi
      blockers+=("server benchmark_gate=${gate:-unknown}$suffix")
    fi
    blocker_text=""
    if ((${#blockers[@]} > 0)); then
      for blocker in "${blockers[@]}"; do
        if [[ -n "$blocker_text" ]]; then
          blocker_text+=" | "
        fi
        blocker_text+="$blocker"
      done
    fi
    status="ready"
    if [[ -n "$blocker_text" ]]; then
      status="blocked"
    fi
    probe_output="$(
      printf 'status=%s\n' "$status"
      printf 'git_commit=%s\n' "$server_commit"
      printf 'git_dirty=%s\n' "$server_dirty"
      printf 'benchmark_gate=%s\n' "$gate"
      printf 'latency_p95_ms=%s\n' "$latency_text"
      printf 'blockers=%s\n' "$blocker_text"
    )"
  fi
  while IFS='=' read -r key value; do
    case "$key" in
      status) server_status="$value" ;;
      git_commit) server_git_commit="$value" ;;
      git_dirty) server_git_dirty="$value" ;;
      benchmark_gate) server_benchmark_gate="$value" ;;
      latency_p95_ms) server_latency_p95_ms="$value" ;;
      blockers) server_blockers="$value" ;;
    esac
  done <<<"$probe_output"
fi

codebase_live_map_path="${MEMD_CODEBASE_LIVE_MAP_STATE:-$ROOT/.memd/state/codebase-live-map.json}"
codebase_live_map_status=""
codebase_live_map_reread_required=""
codebase_live_map_autosync=""
codebase_live_map_updated_at=""
codebase_live_map_fingerprint=""
codebase_live_map_age_s=""
codebase_live_map_ttl_s="${MEMD_CODEBASE_LIVE_MAP_TTL_SECS:-15}"
case "$codebase_live_map_ttl_s" in
  ''|*[!0-9]*) codebase_live_map_ttl_s=15 ;;
esac
codebase_live_map_fresh=""
codebase_live_map_action=""
host_report_status_for_live_map=""
if declare -F memd_host_io_fresh_report_status >/dev/null 2>&1 \
  && [[ -n "$(memd_cargo_volume_root_for_path "$ROOT")" ]]; then
  host_report_status_for_live_map="$(memd_host_io_fresh_report_status "$ROOT" || true)"
fi
if [[ -f "$codebase_live_map_path" ]]; then
  codebase_live_map_json="$(tr -d '\n' < "$codebase_live_map_path" 2>/dev/null || true)"
  codebase_live_map_status="$(printf '%s' "$codebase_live_map_json" | sed -n 's/.*"status"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
  codebase_live_map_autosync="$(printf '%s' "$codebase_live_map_json" | sed -n 's/.*"autosync"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
  codebase_live_map_updated_at="$(printf '%s' "$codebase_live_map_json" | sed -n 's/.*"updated_at"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
  codebase_live_map_fingerprint="$(printf '%s' "$codebase_live_map_json" | sed -n 's/.*"fingerprint"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
  if printf '%s' "$codebase_live_map_json" | grep -q '"needs_reread"[[:space:]]*:[[:space:]]*true'; then
    codebase_live_map_reread_required="true"
  elif printf '%s' "$codebase_live_map_json" | grep -q '"needs_reread"[[:space:]]*:[[:space:]]*false'; then
    codebase_live_map_reread_required="false"
  fi
  if [[ -n "$codebase_live_map_updated_at" ]] && declare -F memd_host_io_report_epoch >/dev/null 2>&1; then
    codebase_live_map_epoch="$(memd_host_io_report_epoch "$codebase_live_map_updated_at" 2>/dev/null || true)"
    if [[ -n "$codebase_live_map_epoch" ]]; then
      now_epoch="$(date -u '+%s')"
      codebase_live_map_age_s=$((now_epoch - codebase_live_map_epoch))
      if [[ "$codebase_live_map_age_s" -lt 0 ]]; then
        codebase_live_map_age_s=0
      fi
      if [[ "$codebase_live_map_age_s" -le "$codebase_live_map_ttl_s" ]]; then
        codebase_live_map_fresh=true
      else
        codebase_live_map_fresh=false
      fi
    fi
  fi
fi
if [[ "$host_report_status_for_live_map" == "blocked" ]] \
  && { [[ "$codebase_live_map_status" == "blocked" ]] || [[ "$codebase_live_map_reread_required" == "true" ]]; }; then
  codebase_live_map_action="wait_or_coordinate_before_broad_repo_work"
elif [[ "$codebase_live_map_fresh" == "false" ]]; then
  codebase_live_map_action="refresh_host_guard_before_trusting_live_map"
elif [[ "$codebase_live_map_status" == "blocked" ]]; then
  codebase_live_map_action="wait_or_coordinate_before_broad_repo_work"
elif [[ "$codebase_live_map_reread_required" == "true" ]]; then
  codebase_live_map_action="inspect_codebase_live_map_diff_before_broad_repo_work"
elif [[ -z "$codebase_live_map_status" ]]; then
  codebase_live_map_action="missing_live_map_run_host_guard_or_awareness"
else
  codebase_live_map_action="live_map_current"
fi

cat <<ENV
MEMD_GIT_BRANCH=$branch
MEMD_GIT_COMMIT=$commit
MEMD_GIT_DIRTY=$dirty
MEMD_AUTHORITY_CONTAINER=$authority_container
MEMD_AUTHORITY_IMAGE_REPO=$authority_image_repo
MEMD_AUTHORITY_PORT=$authority_port
MEMD_AUTHORITY_URL=$authority_url
MEMD_AUTHORITY_DEPLOY_CONTRACT=$authority_deploy_contract
MEMD_AUTHORITY_IDENTITY_STATUS=$authority_identity_status
MEMD_AUTHORITY_IDENTITY_BLOCKERS=$authority_identity_blockers
MEMD_SERVER_STATUS=$server_status
MEMD_SERVER_STATUS_URL=$status_url
MEMD_SERVER_GIT_COMMIT=$server_git_commit
MEMD_SERVER_GIT_DIRTY=$server_git_dirty
MEMD_SERVER_BENCHMARK_GATE=$server_benchmark_gate
MEMD_SERVER_LATENCY_P95_MS=$server_latency_p95_ms
MEMD_CODEBASE_LIVE_MAP_STATE=$codebase_live_map_path
MEMD_CODEBASE_LIVE_MAP_STATUS=$codebase_live_map_status
MEMD_CODEBASE_LIVE_MAP_REREAD_REQUIRED=$codebase_live_map_reread_required
MEMD_CODEBASE_LIVE_MAP_AUTOSYNC=$codebase_live_map_autosync
MEMD_CODEBASE_LIVE_MAP_UPDATED_AT=$codebase_live_map_updated_at
MEMD_CODEBASE_LIVE_MAP_FINGERPRINT=$codebase_live_map_fingerprint
MEMD_CODEBASE_LIVE_MAP_AGE_SECS=$codebase_live_map_age_s
MEMD_CODEBASE_LIVE_MAP_TTL_SECS=$codebase_live_map_ttl_s
MEMD_CODEBASE_LIVE_MAP_FRESH=$codebase_live_map_fresh
MEMD_CODEBASE_LIVE_MAP_ACTION=$codebase_live_map_action
ENV
if [[ -n "$git_status_blockers" ]]; then
  printf 'MEMD_GIT_STATUS_BLOCKERS=%s\n' "$(printf '%s' "$git_status_blockers" | tr '\n' '|')"
fi

cat >&2 <<MSG
memd-server deploy env:
  MEMD_GIT_BRANCH=$branch
  MEMD_GIT_COMMIT=$commit
  MEMD_GIT_DIRTY=$dirty
  MEMD_AUTHORITY_CONTAINER=$authority_container
  MEMD_AUTHORITY_IMAGE_REPO=$authority_image_repo
  MEMD_AUTHORITY_PORT=$authority_port
  MEMD_AUTHORITY_URL=$authority_url
  MEMD_AUTHORITY_DEPLOY_CONTRACT=$authority_deploy_contract
  MEMD_AUTHORITY_IDENTITY_STATUS=$authority_identity_status
  MEMD_AUTHORITY_IDENTITY_BLOCKERS=$authority_identity_blockers
  MEMD_SERVER_STATUS=$server_status
  MEMD_SERVER_STATUS_URL=$status_url
  MEMD_SERVER_GIT_COMMIT=$server_git_commit
  MEMD_SERVER_GIT_DIRTY=$server_git_dirty
  MEMD_SERVER_BENCHMARK_GATE=$server_benchmark_gate
  MEMD_SERVER_LATENCY_P95_MS=$server_latency_p95_ms
  MEMD_CODEBASE_LIVE_MAP_STATE=$codebase_live_map_path
  MEMD_CODEBASE_LIVE_MAP_STATUS=$codebase_live_map_status
  MEMD_CODEBASE_LIVE_MAP_REREAD_REQUIRED=$codebase_live_map_reread_required
  MEMD_CODEBASE_LIVE_MAP_AUTOSYNC=$codebase_live_map_autosync
  MEMD_CODEBASE_LIVE_MAP_UPDATED_AT=$codebase_live_map_updated_at
  MEMD_CODEBASE_LIVE_MAP_FINGERPRINT=$codebase_live_map_fingerprint
  MEMD_CODEBASE_LIVE_MAP_AGE_SECS=$codebase_live_map_age_s
  MEMD_CODEBASE_LIVE_MAP_TTL_SECS=$codebase_live_map_ttl_s
  MEMD_CODEBASE_LIVE_MAP_FRESH=$codebase_live_map_fresh
  MEMD_CODEBASE_LIVE_MAP_ACTION=$codebase_live_map_action
  MEMD_GIT_STATUS_BLOCKERS=$(printf '%s' "$git_status_blockers" | tr '\n' '|')

Docker build example:
  docker build -f deploy/docker/Dockerfile.memd-server \\
    --build-arg MEMD_GIT_BRANCH=$branch \\
    --build-arg MEMD_GIT_COMMIT=$commit \\
    --build-arg MEMD_GIT_DIRTY=$dirty \\
    -t $authority_image_repo:$commit .
MSG

if [[ "$authority_identity_status" == "blocked" ]]; then
  cat >&2 <<MSG

memd authority identity blocked:
  $authority_identity_blockers
  contract: $authority_deploy_contract
MSG
fi

if [[ "$server_status" == "blocked" ]]; then
  cat >&2 <<MSG

memd-server live status blocked:
  $server_blockers
MSG
elif [[ "$server_status" == "unavailable" ]]; then
  cat >&2 <<MSG

memd-server live status unavailable:
  ${server_blockers:-no status URL configured}
MSG
fi

if [[ "$authority_identity_status" != "ready" && "${MEMD_REQUIRE_AUTHORITY_IDENTITY_READY:-1}" == "1" ]]; then
  exit 4
fi

if [[ "$server_status" != "ready" && "${MEMD_REQUIRE_SERVER_READY:-0}" == "1" ]]; then
  exit 3
fi
