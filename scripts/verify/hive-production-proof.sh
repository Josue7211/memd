#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MEMD="${MEMD_BIN:-$ROOT/target/debug/memd}"
SERVER="${MEMD_SERVER_BIN:-$ROOT/target/debug/memd-server}"
RUN_TAILSCALE_CANARY=0

usage() {
  cat <<'USAGE'
Usage: scripts/verify/hive-production-proof.sh [--tailscale-canary]

Runs the destructive hive proof against an isolated local memd-server/SQLite DB.
With --tailscale-canary, also runs a tiny shared-backend canary under a unique
hive-canary-<uuid> namespace and retires/releases its canary state.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tailscale-canary)
      RUN_TAILSCALE_CANARY=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

need curl
need jq
need python3
need git

MEMD_CARGO_TARGET_DIR="${MEMD_CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/memd-cargo-target-hive-proof}"
source "$ROOT/scripts/lib/memd-cargo-env.sh"
memd_cargo_refuse_on_host_blockers

(cd "$ROOT" && cargo build -p memd-client -p memd-server)

MEMD="${MEMD_BIN:-$MEMD_CARGO_TARGET_DIR/debug/memd}"
SERVER="${MEMD_SERVER_BIN:-$MEMD_CARGO_TARGET_DIR/debug/memd-server}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/memd-hive-proof.XXXXXX")"
server_pid=""
dev_guard_pid=""

cleanup() {
  if [[ -n "$dev_guard_pid" ]] && kill -0 "$dev_guard_pid" >/dev/null 2>&1; then
    kill "$dev_guard_pid" >/dev/null 2>&1 || true
    wait "$dev_guard_pid" >/dev/null 2>&1 || true
  fi
  if [[ -n "$server_pid" ]] && kill -0 "$server_pid" >/dev/null 2>&1; then
    kill "$server_pid" >/dev/null 2>&1 || true
    wait "$server_pid" >/dev/null 2>&1 || true
  fi
  if [[ "${MEMD_HIVE_PROOF_KEEP_TMP:-0}" == "1" ]]; then
    printf 'kept temp proof dir: %s\n' "$tmp_root" >&2
  else
    rm -rf "$tmp_root"
  fi
}
trap cleanup EXIT

log() {
  printf '==> %s\n' "$*"
}

json_get() {
  local url="$1"
  shift
  curl -fsS -G "$url" "$@"
}

json_post() {
  local url="$1"
  local body="$2"
  curl -fsS -X POST "$url" -H 'content-type: application/json' --data "$body"
}

assert_jq() {
  local label="$1"
  local expr="$2"
  local file="$3"
  if ! jq -e "$expr" "$file" >/dev/null; then
    echo "assert failed: $label" >&2
    jq . "$file" >&2 || cat "$file" >&2
    exit 1
  fi
}

run_memd() {
  MEMD_BASE_URL="$BASE_URL" "$MEMD" --base-url "$BASE_URL" "$@"
}

write_min_bundle() {
  local bundle="$1"
  local project="$2"
  local namespace="$3"
  local agent="$4"
  local session="$5"
  local role="$6"
  local base_url="$7"

  mkdir -p "$bundle/state"
  cat > "$bundle/config.json" <<JSON
{
  "project": "$project",
  "namespace": "$namespace",
  "agent": "$agent",
  "session": "$session",
  "tab_id": "tab-$session",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "$base_url",
  "route": "auto",
  "intent": "current_task",
  "hive_system": "memd",
  "hive_role": "$role",
  "capabilities": ["coordination", "memory"],
  "hive_groups": ["project:$project"],
  "hive_group_goal": "prove hive production coordination",
  "authority": "participant",
  "authority_policy": {
    "shared_primary": true,
    "localhost_fallback_policy": "deny",
    "shared_required_for": ["shared_claim_mutations", "shared_task_mutations", "shared_message_mutations"]
  },
  "authority_state": {
    "mode": "shared",
    "degraded": false,
    "shared_base_url": "$base_url",
    "fallback_base_url": null,
    "activated_at": null,
    "activated_by": "hive-production-proof",
    "reason": "shared authority available",
    "warning_acknowledged_at": null,
    "expires_at": null,
    "blocked_capabilities": []
  }
}
JSON
  cat > "$bundle/env" <<ENV
MEMD_BASE_URL='$base_url'
MEMD_PROJECT='$project'
MEMD_NAMESPACE='$namespace'
MEMD_AGENT='$agent'
MEMD_SESSION='$session'
MEMD_WORKSPACE='shared'
MEMD_VISIBILITY='workspace'
MEMD_VOICE_MODE='normal'
MEMD_AUTHORITY_MODE='shared'
MEMD_LOCALHOST_FALLBACK_POLICY='deny'
MEMD_AUTHORITY_DEGRADED='false'
MEMD_SHARED_BASE_URL='$base_url'
ENV
  : > "$bundle/backend.env"
  : > "$bundle/env.ps1"
  : > "$bundle/backend.env.ps1"
}

start_local_server() {
  local port
  port="$(python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
)"
  BASE_URL="http://127.0.0.1:$port"
  MEMD_DB_PATH="$tmp_root/local/memd.db" MEMD_BIND_ADDR="127.0.0.1:$port" "$SERVER" \
    >"$tmp_root/local-server.log" 2>&1 &
  server_pid="$!"
  for _ in {1..100}; do
    if curl -fsS "$BASE_URL/healthz" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.05
  done
  echo "local memd-server did not become healthy" >&2
  cat "$tmp_root/local-server.log" >&2 || true
  exit 1
}

run_local_proof() {
  log "local isolated server"
  mkdir -p "$tmp_root/local"
  start_local_server

  local project="hive-proof"
  local namespace="hive-proof-$(uuidgen | tr 'A-Z' 'a-z')"
  local workspace="$tmp_root/local/workspace"
  local queen="$workspace/queen/.memd"
  local worker_a="$workspace/worker-a/.memd"
  local worker_b="$workspace/worker-b/.memd"
  local task_id="task-proof-$(uuidgen | tr 'A-Z' 'a-z')"
  local help_task_id="task-help-$(uuidgen | tr 'A-Z' 'a-z')"
  local review_task_id="task-review-$(uuidgen | tr 'A-Z' 'a-z')"
  local claim_scope="scope:proof:$(uuidgen | tr 'A-Z' 'a-z')"
  mkdir -p "$workspace"

  log "create hive and join scripted agents"
  run_memd hive --output "$queen" --project-root "$workspace" --project "$project" --namespace "$namespace" \
    --agent codex --session queen --tab-id tab-queen --hive-system memd --hive-role coordinator \
    --capability coordination --hive-group "project:$project" --workspace shared --visibility workspace \
    --base-url "$BASE_URL" >/dev/null
  run_memd hive --output "$worker_a" --project-root "$workspace" --project "$project" --namespace "$namespace" \
    --agent codex --session worker-a --tab-id tab-worker-a --hive-system memd --hive-role agent \
    --capability memory --hive-group "project:$project" --workspace shared --visibility workspace \
    --base-url "$BASE_URL" >/dev/null
  run_memd hive --output "$worker_b" --project-root "$workspace" --project "$project" --namespace "$namespace" \
    --agent claude-code --session worker-b --tab-id tab-worker-b --hive-system memd --hive-role agent \
    --capability review --hive-group "project:$project" --workspace shared --visibility workspace \
    --base-url "$BASE_URL" >/dev/null
  run_memd hive-join --output "$worker_a" --base-url "$BASE_URL" --summary >/dev/null
  run_memd hive-join --output "$worker_b" --base-url "$BASE_URL" --summary >/dev/null
  run_memd heartbeat --output "$queen" --probe-base-url >/dev/null
  run_memd heartbeat --output "$worker_a" --probe-base-url >/dev/null
  run_memd heartbeat --output "$worker_b" --probe-base-url >/dev/null

  json_get "$BASE_URL/coordination/sessions" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "active_only=true" \
    --data-urlencode "limit=16" >"$tmp_root/sessions.json"
  assert_jq "three active hive sessions" '.sessions | map(.session) | contains(["queen", "worker-a", "worker-b"])' "$tmp_root/sessions.json"
  run_memd hive roster --output "$queen" --json >"$tmp_root/roster.json"
  assert_jq "roster sees workers" '.bees | map(.session) | contains(["worker-a", "worker-b"])' "$tmp_root/roster.json"
  run_memd hive follow --output "$queen" --session worker-a --json >"$tmp_root/follow-worker-a.json"
  assert_jq "follow worker-a" '.target.session == "worker-a"' "$tmp_root/follow-worker-a.json"

  log "message, inbox, ack"
  run_memd messages --output "$queen" --send --target-session worker-a --kind note \
    --content "proof ping from queen" --summary >/dev/null
  json_get "$BASE_URL/coordination/messages/inbox" \
    --data-urlencode "session=worker-a" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "include_acknowledged=false" \
    --data-urlencode "limit=16" >"$tmp_root/inbox-worker-a.json"
  assert_jq "worker-a receives note" '.messages | any(.kind == "note" and .content == "proof ping from queen")' "$tmp_root/inbox-worker-a.json"
  local message_id
  message_id="$(jq -r '.messages[] | select(.kind == "note" and .content == "proof ping from queen") | .id' "$tmp_root/inbox-worker-a.json" | head -n1)"
  run_memd messages --output "$worker_a" --ack "$message_id" --summary >/dev/null
  json_get "$BASE_URL/coordination/messages/inbox" \
    --data-urlencode "session=worker-a" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "include_acknowledged=false" \
    --data-urlencode "limit=16" >"$tmp_root/inbox-worker-a-after-ack.json"
  assert_jq "worker-a note acked out of inbox" '.messages | all(.id != "'"$message_id"'")' "$tmp_root/inbox-worker-a-after-ack.json"

  log "tasks assign help review"
  run_memd tasks --output "$queen" --upsert --task-id "$task_id" --title "Proof task" \
    --description "exercise hive production task flow" --mode exclusive_write --scope "$claim_scope" --summary >/dev/null
  run_memd tasks --output "$queen" --assign-to-session worker-a --task-id "$task_id" --summary >/dev/null
  json_get "$BASE_URL/coordination/inbox" \
    --data-urlencode "session=worker-a" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "limit=32" >"$tmp_root/coord-inbox-worker-a-owned.json"
  assert_jq "worker-a owns assigned task" '.owned_tasks | any(.task_id == "'"$task_id"'")' "$tmp_root/coord-inbox-worker-a-owned.json"
  run_memd tasks --output "$queen" --request-help --target-session worker-a --task-id "$help_task_id" \
    --scope "$claim_scope" --summary >/dev/null
  run_memd tasks --output "$queen" --request-review --target-session worker-b --task-id "$review_task_id" \
    --scope "$claim_scope" --summary >/dev/null
  json_get "$BASE_URL/coordination/inbox" \
    --data-urlencode "session=worker-a" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "limit=32" >"$tmp_root/coord-inbox-worker-a.json"
  assert_jq "worker-a help task" '.help_tasks | any(.task_id == "'"$help_task_id"'")' "$tmp_root/coord-inbox-worker-a.json"
  json_get "$BASE_URL/coordination/inbox" \
    --data-urlencode "session=worker-b" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "limit=32" >"$tmp_root/coord-inbox-worker-b.json"
  assert_jq "worker-b review task" '.review_tasks | any(.task_id == "'"$review_task_id"'")' "$tmp_root/coord-inbox-worker-b.json"

  log "handoff message and receipt"
  run_memd hive handoff --output "$worker_a" --to-session worker-b --task-id "$task_id" \
    --scope "$claim_scope" --next-action "review proof task" --note "handoff from proof" --summary >/dev/null
  json_get "$BASE_URL/coordination/messages/inbox" \
    --data-urlencode "session=worker-b" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "include_acknowledged=false" \
    --data-urlencode "limit=32" >"$tmp_root/inbox-worker-b.json"
  assert_jq "worker-b receives handoff" '.messages | any(.kind == "handoff" and (.content | contains("review proof task")))' "$tmp_root/inbox-worker-b.json"
  json_get "$BASE_URL/coordination/receipts" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "limit=64" >"$tmp_root/receipts.json"
  assert_jq "handoff receipt exists" '.receipts | any(.kind == "queen_handoff" and .target_session == "worker-b")' "$tmp_root/receipts.json"

  log "claim acquire transfer release"
  local transfer_scope="scope:transfer:$(uuidgen | tr 'A-Z' 'a-z')"
  run_memd claims --output "$queen" --acquire --scope "$transfer_scope" --ttl-secs 60 --summary >/dev/null
  run_memd claims --output "$queen" --transfer-to-session worker-a --scope "$transfer_scope" --summary >/dev/null
  json_get "$BASE_URL/coordination/claims" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "active_only=true" \
    --data-urlencode "limit=64" >"$tmp_root/claims-after-transfer.json"
  assert_jq "claim transferred to worker-a" '.claims | any(.scope == "'"$transfer_scope"'" and .session == "worker-a")' "$tmp_root/claims-after-transfer.json"
  run_memd claims --output "$worker_a" --release --scope "$transfer_scope" --summary >/dev/null
  json_get "$BASE_URL/coordination/claims" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "active_only=true" \
    --data-urlencode "limit=64" >"$tmp_root/claims-after-release.json"
  assert_jq "claim released" '.claims | all(.scope != "'"$transfer_scope"'")' "$tmp_root/claims-after-release.json"

  log "dev-server lease race hard-block and board visibility"
  local dev_port
  dev_port="$(python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
)"
  run_memd dev-server guard --output "$worker_a" --port "$dev_port" --ttl-secs 20 --stale-after-secs 2 --summary -- \
    sleep 10 >"$tmp_root/dev-guard-a.out" 2>"$tmp_root/dev-guard-a.err" &
  dev_guard_pid="$!"
  local lease_seen=0
  for _ in {1..100}; do
    run_memd dev-server list --output "$worker_a" --summary >"$tmp_root/dev-server-list-active.out"
    if grep -q "127.0.0.1:$dev_port" "$tmp_root/dev-server-list-active.out"; then
      lease_seen=1
      break
    fi
    sleep 0.05
  done
  if [[ "$lease_seen" -ne 1 ]]; then
    echo "expected worker-a dev-server lease to become visible" >&2
    cat "$tmp_root/dev-guard-a.out" >&2 || true
    cat "$tmp_root/dev-guard-a.err" >&2 || true
    cat "$tmp_root/dev-server-list-active.out" >&2 || true
    exit 1
  fi
  if run_memd dev-server guard --output "$worker_b" --port "$dev_port" --ttl-secs 20 --stale-after-secs 2 --summary -- \
    sleep 1 >"$tmp_root/dev-guard-b.out" 2>"$tmp_root/dev-guard-b.err"; then
    echo "expected competing dev-server guard to hard-block" >&2
    cat "$tmp_root/dev-guard-b.out" >&2
    exit 1
  fi
  grep -Eq "409 Conflict|dev_server_conflict|already leased" "$tmp_root/dev-guard-b.err"
  json_get "$BASE_URL/coordination/receipts" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "limit=128" >"$tmp_root/dev-server-receipts.json"
  assert_jq "dev server acquire receipt exists" '.receipts | any(.kind == "dev_server_acquire")' "$tmp_root/dev-server-receipts.json"
  assert_jq "dev server conflict receipt exists" '.receipts | any(.kind == "dev_server_conflict")' "$tmp_root/dev-server-receipts.json"
  run_memd hive --output "$queen" --summary >"$tmp_root/hive-board-dev-server.out"
  grep -q "dev-server http://127.0.0.1:$dev_port" "$tmp_root/hive-board-dev-server.out"
  grep -q "reuse http://127.0.0.1:$dev_port" "$tmp_root/hive-board-dev-server.out"
  wait "$dev_guard_pid"
  dev_guard_pid=""
  run_memd dev-server list --output "$worker_a" --summary >"$tmp_root/dev-server-list-released.out"
  if grep -q "127.0.0.1:$dev_port" "$tmp_root/dev-server-list-released.out"; then
    echo "expected dev-server lease to release after guarded command exits" >&2
    cat "$tmp_root/dev-server-list-released.out" >&2
    exit 1
  fi

  log "lane collision rejection and hive-join reroute"
  local lane_root="$tmp_root/lane"
  local current_project="$lane_root/current"
  local target_project="$lane_root/target"
  local current_bundle="$current_project/.memd"
  local target_bundle="$target_project/.memd"
  mkdir -p "$current_bundle" "$target_bundle"
  printf '# current\n' >"$current_project/README.md"
  printf '# target\n' >"$target_project/NOTES.md"
  git -C "$lane_root" init >/dev/null
  git -C "$lane_root" config user.email proof@example.invalid
  git -C "$lane_root" config user.name "Hive Proof"
  git -C "$lane_root" add .
  git -C "$lane_root" commit -m init >/dev/null
  git -C "$lane_root" checkout -b feature/hive-shared >/dev/null
  write_min_bundle "$current_bundle" "$project" "$namespace" codex lane-current agent "$BASE_URL"
  write_min_bundle "$target_bundle" "$project" "$namespace" claude-code lane-target agent "$BASE_URL"
  run_memd heartbeat --output "$current_bundle" --probe-base-url >/dev/null
  run_memd heartbeat --output "$target_bundle" --probe-base-url >/dev/null
  if run_memd tasks --output "$current_bundle" --assign-to-session lane-target --task-id lane-task --summary \
    >"$tmp_root/lane-assign.out" 2>"$tmp_root/lane-assign.err"; then
    echo "expected lane assignment collision to fail" >&2
    cat "$tmp_root/lane-assign.out" >&2
    exit 1
  fi
  grep -q "unsafe hive cowork target collision" "$tmp_root/lane-assign.err"
  run_memd hive-join --output "$current_bundle" --base-url "$BASE_URL" --summary >"$tmp_root/lane-reroute.out"
  grep -q "lane_rerouted=yes" "$tmp_root/lane-reroute.out"

  log "local proof ok namespace=$namespace"
}

run_tailscale_canary() {
  local canary_base="${MEMD_HIVE_TAILSCALE_BASE_URL:-${MEMD_BASE_URL:-http://100.104.154.24:8788}}"
  BASE_URL="$canary_base"
  log "tailscale canary base=$BASE_URL"
  curl -fsS "$BASE_URL/healthz" >/dev/null

  local project="${MEMD_HIVE_TAILSCALE_PROJECT:-memd}"
  local namespace="hive-canary-$(uuidgen | tr 'A-Z' 'a-z')"
  local workspace="$tmp_root/tailscale-canary/workspace"
  local queen="$workspace/queen/.memd"
  local worker="$workspace/worker/.memd"
  local claim_scope="scope:$namespace"
  mkdir -p "$workspace"

  run_memd hive --output "$queen" --project-root "$workspace" --project "$project" --namespace "$namespace" \
    --agent codex --session canary-queen --tab-id tab-canary-queen --hive-system memd --hive-role coordinator \
    --capability coordination --hive-group "hive-canary" --workspace shared --visibility workspace \
    --base-url "$BASE_URL" >/dev/null
  run_memd hive --output "$worker" --project-root "$workspace" --project "$project" --namespace "$namespace" \
    --agent codex --session canary-worker --tab-id tab-canary-worker --hive-system memd --hive-role agent \
    --capability memory --hive-group "hive-canary" --workspace shared --visibility workspace \
    --base-url "$BASE_URL" >/dev/null
  run_memd heartbeat --output "$queen" --probe-base-url >/dev/null
  run_memd heartbeat --output "$worker" --probe-base-url >/dev/null
  run_memd messages --output "$queen" --send --target-session canary-worker --kind canary \
    --content "hive canary $namespace" --summary >/dev/null
  run_memd claims --output "$queen" --acquire --scope "$claim_scope" --ttl-secs 60 --summary >/dev/null

  json_get "$BASE_URL/coordination/messages/inbox" \
    --data-urlencode "session=canary-worker" \
    --data-urlencode "project=$project" \
    --data-urlencode "namespace=$namespace" \
    --data-urlencode "workspace=shared" \
    --data-urlencode "include_acknowledged=false" \
    --data-urlencode "limit=8" >"$tmp_root/canary-inbox.json"
  assert_jq "canary message arrived" '.messages | any(.kind == "canary")' "$tmp_root/canary-inbox.json"
  run_memd claims --output "$queen" --release --scope "$claim_scope" --summary >/dev/null

  json_post "$BASE_URL/coordination/sessions/retire" \
    '{"session":"canary-queen","project":"'"$project"'","namespace":"'"$namespace"'","workspace":"shared","reason":"hive canary cleanup"}' >/dev/null
  json_post "$BASE_URL/coordination/sessions/retire" \
    '{"session":"canary-worker","project":"'"$project"'","namespace":"'"$namespace"'","workspace":"shared","reason":"hive canary cleanup"}' >/dev/null
  log "tailscale canary ok namespace=$namespace"
}

run_local_proof
if [[ "$RUN_TAILSCALE_CANARY" -eq 1 ]]; then
  run_tailscale_canary
else
  log "tailscale canary skipped (pass --tailscale-canary to run shared canary)"
fi
