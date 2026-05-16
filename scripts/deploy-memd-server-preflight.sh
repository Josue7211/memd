#!/usr/bin/env bash
# Emit deploy env for memd-server and block dirty authority builds by default.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

branch="$(git rev-parse --abbrev-ref HEAD)"
commit="$(git rev-parse --short HEAD)"
dirty="clean"
if [[ -n "$(git status --porcelain)" ]]; then
  dirty="dirty"
fi

if [[ "$dirty" != "clean" && "${MEMD_ALLOW_DIRTY_DEPLOY:-0}" != "1" ]]; then
  cat >&2 <<MSG
memd-server deploy blocked: working tree is dirty.
Commit or clean changes, then rerun.
To override for an explicit emergency deploy, set MEMD_ALLOW_DIRTY_DEPLOY=1.
MSG
  exit 2
fi

status_url="${MEMD_SERVER_STATUS_URL:-}"
if [[ -z "$status_url" && -f ".memd/config.json" ]]; then
  status_url="$(
    python3 - <<'PY'
import json

try:
    with open(".memd/config.json", "r", encoding="utf-8") as handle:
        config = json.load(handle)
except Exception:
    raise SystemExit(0)

base_url = (
    config.get("authority_state", {}).get("shared_base_url")
    or config.get("base_url")
    or ""
)
if base_url:
    print(base_url.rstrip("/") + "/api/status")
PY
  )"
fi

server_status="unavailable"
server_git_commit=""
server_git_dirty=""
server_benchmark_gate=""
server_latency_p95_ms=""
server_blockers=""

if [[ -n "$status_url" ]]; then
  probe_output="$(
    python3 - "$status_url" "$commit" <<'PY'
import json
import os
import sys
import urllib.error
import urllib.request

url = sys.argv[1]
local_commit = sys.argv[2]
timeout = float(os.environ.get("MEMD_SERVER_STATUS_TIMEOUT", "3"))

try:
    with urllib.request.urlopen(url, timeout=timeout) as response:
        status_code = getattr(response, "status", 200)
        payload = json.load(response)
except (OSError, json.JSONDecodeError, urllib.error.URLError) as exc:
    print("status=unavailable")
    print(f"blockers=status probe failed: {exc}")
    raise SystemExit(0)

if status_code < 200 or status_code >= 300:
    print("status=unavailable")
    print(f"blockers=status probe returned HTTP {status_code}")
    raise SystemExit(0)

server_commit = str(payload.get("git_commit") or "")
server_dirty = str(payload.get("git_dirty") or "")
gate = str(payload.get("benchmark_gate") or "")
latency = payload.get("latency_p95_ms")
latency_text = "" if latency is None else str(latency)

blockers = []
if server_commit and server_commit != local_commit:
    blockers.append(
        f"server git_commit={server_commit} does not match local HEAD {local_commit}"
    )
if server_dirty and server_dirty != "clean":
    blockers.append(f"server_git_dirty={server_dirty}")
if gate not in ("pass", "acceptable"):
    suffix = f" latency_p95_ms={latency_text}" if latency_text else ""
    blockers.append(f"server benchmark_gate={gate or 'unknown'}{suffix}")

print("status=" + ("blocked" if blockers else "ready"))
print("git_commit=" + server_commit)
print("git_dirty=" + server_dirty)
print("benchmark_gate=" + gate)
print("latency_p95_ms=" + latency_text)
print("blockers=" + " | ".join(blockers))
PY
  )"
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

cat <<ENV
MEMD_GIT_BRANCH=$branch
MEMD_GIT_COMMIT=$commit
MEMD_GIT_DIRTY=$dirty
MEMD_SERVER_STATUS=$server_status
MEMD_SERVER_STATUS_URL=$status_url
MEMD_SERVER_GIT_COMMIT=$server_git_commit
MEMD_SERVER_GIT_DIRTY=$server_git_dirty
MEMD_SERVER_BENCHMARK_GATE=$server_benchmark_gate
MEMD_SERVER_LATENCY_P95_MS=$server_latency_p95_ms
ENV

cat >&2 <<MSG
memd-server deploy env:
  MEMD_GIT_BRANCH=$branch
  MEMD_GIT_COMMIT=$commit
  MEMD_GIT_DIRTY=$dirty
  MEMD_SERVER_STATUS=$server_status
  MEMD_SERVER_STATUS_URL=$status_url
  MEMD_SERVER_GIT_COMMIT=$server_git_commit
  MEMD_SERVER_GIT_DIRTY=$server_git_dirty
  MEMD_SERVER_BENCHMARK_GATE=$server_benchmark_gate
  MEMD_SERVER_LATENCY_P95_MS=$server_latency_p95_ms

Docker build example:
  docker build -f deploy/docker/Dockerfile.memd-server \\
    --build-arg MEMD_GIT_BRANCH=$branch \\
    --build-arg MEMD_GIT_COMMIT=$commit \\
    --build-arg MEMD_GIT_DIRTY=$dirty \\
    -t memd-server:$commit .
MSG

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

if [[ "$server_status" != "ready" && "${MEMD_REQUIRE_SERVER_READY:-0}" == "1" ]]; then
  exit 3
fi
