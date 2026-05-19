#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/scripts/lib/memd-cargo-env.sh"

wrapper_output="$(MEMD_AR_WRAPPER_DEBUG=1 "$ROOT/scripts/memd-ar-wrapper.sh" cqD -rD lib.a obj.o)"
grep -q '^cq$' <<<"$wrapper_output"
grep -q '^-r$' <<<"$wrapper_output"
if grep -q 'D' <<<"$wrapper_output"; then
  echo "memd host I/O guard test: ar wrapper did not strip deterministic D flag" >&2
  exit 1
fi

fixture="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-guard.XXXXXX")"
trap 'rm -f "$fixture"' EXIT

printf '%s\n' \
  'PID PPID STAT COMMAND' \
  '10 1 Us /System/Library/PrivateFrameworks/UVFSXPCService.framework/Versions/A/XPCServices/UVFSService.xpc/Contents/MacOS/UVFSService' \
  '11 1 U /Volumes/T7/Xcodes/Xcode.app/Contents/Developer/usr/bin/git -C /Volumes/T7/projects/clawcontrol status --short' \
  '12 1 U /Volumes/T7/Xcodes/Xcode.app/Contents/Developer/usr/bin/git -C /Volumes/T7/projects/memd status --short' \
  '13 1 S /Volumes/T7/Xcodes/Xcode.app/Contents/Developer/usr/bin/git -C /Volumes/T7/projects/memd status --short' \
  '14 1 U /Volumes/T7/Xcodes/Xcode-26.4.1.app/Contents/Developer/usr/bin/git -c core.fsmonitor=false status --porcelain=v1 -z' \
  '15 1 U /Volumes/T7/Xcodes/Xcode-26.4.1.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang -c build/native.o' \
  > "$fixture"

output="$(
  MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
  MEMD_HOST_IO_PS_FILE="$fixture" \
  memd_cargo_host_blockers
)"

grep -q 'volume:/Volumes/T7 project_hint=filesystem pid=10 state=Us' <<<"$output"
grep -q 'volume:/Volumes/T7 project_hint=clawcontrol pid=11 state=U' <<<"$output"
grep -q 'repo project_hint=memd pid=12 state=U' <<<"$output"
grep -q 'volume:/Volumes/T7 project_hint=app-git pid=14 state=U' <<<"$output"
grep -q 'volume:/Volumes/T7 project_hint=native-tooling pid=15 state=U' <<<"$output"
if grep -q 'pid=13' <<<"$output"; then
  echo "memd host I/O guard test: non-blocked process leaked" >&2
  exit 1
fi

if ! MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
  MEMD_HOST_IO_PS_FILE="$fixture" \
  MEMD_HOST_IO_GUARD=0 \
  memd_cargo_refuse_on_host_blockers; then
  echo "memd host I/O guard test: generic override failed" >&2
  exit 1
fi

fixture_report_repo="$(mktemp -d "${TMPDIR:-/tmp}/memd-host-io-fixture-report-repo.XXXXXX")"
trap 'rm -f "$fixture"; rm -rf "$fixture_report_repo"' EXIT
set +e
MEMD_CARGO_REPO_ROOT="$fixture_report_repo" \
MEMD_CARGO_VOLUME_ROOT=/Volumes/T7 \
MEMD_HOST_IO_PS_FILE="$fixture" \
memd_cargo_refuse_on_host_blockers >/tmp/memd-host-io-fixture-report-test.out 2>&1
fixture_report_status=$?
set -e
if [[ "$fixture_report_status" -ne 75 ]]; then
  echo "memd host I/O guard test: fixture report guard did not return 75" >&2
  exit 1
fi
if [[ -e "$fixture_report_repo/.memd/state/host-io-guard.txt" ]]; then
  echo "memd host I/O guard test: fixture ps wrote default repo report" >&2
  exit 1
fi

clear_fixture="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-guard-clear.XXXXXX")"
trap 'rm -f "$fixture" "$clear_fixture"; rm -rf "$fixture_report_repo"' EXIT
printf '%s\n' \
  'PID PPID STAT COMMAND' \
  '20 1 S /Volumes/T7/Xcodes/Xcode.app/Contents/Developer/usr/bin/git -C /Volumes/T7/projects/memd status --short' \
  > "$clear_fixture"

clear_output="$(
  MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
  MEMD_HOST_IO_PS_FILE="$clear_fixture" \
  memd_cargo_host_blockers
)"
if [[ -n "$clear_output" ]]; then
  echo "memd host I/O guard test: clear fixture reported blockers" >&2
  printf '%s\n' "$clear_output" >&2
  exit 1
fi

sibling_fixture="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-guard-sibling.XXXXXX")"
trap 'rm -f "$fixture" "$clear_fixture" "$sibling_fixture"; rm -rf "$fixture_report_repo"' EXIT
printf '%s\n' \
  'PID PPID STAT COMMAND' \
  '30 1 U /Volumes/T7/Xcodes/Xcode.app/Contents/Developer/usr/bin/git -C /Volumes/T7/projects/clawcontrol status --short' \
  > "$sibling_fixture"

sibling_output="$(
  MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
  MEMD_HOST_IO_PS_FILE="$sibling_fixture" \
  memd_cargo_host_blockers
)"
grep -q 'project_hint=clawcontrol' <<<"$sibling_output"
active_runtime_fixture="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-active-runtime.XXXXXX")"
trap 'rm -f "$fixture" "$clear_fixture" "$sibling_fixture" "$active_runtime_fixture"; rm -rf "$fixture_report_repo"' EXIT
printf '%s\n' \
  'PID PPID STAT COMMAND' \
  '31 1 S /Volumes/T7/node-v24.14.0-darwin-arm64/bin/node /Volumes/T7/projects/clawcontrol/deploy/agentshell/agent-shell-adapter.js' \
  > "$active_runtime_fixture"
active_runtime_output="$(
  MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
  MEMD_HOST_IO_PS_FILE="$active_runtime_fixture" \
  memd_cargo_host_blockers
)"
grep -q 'project_hint=clawcontrol pid=31 state=S' <<<"$active_runtime_output"
grep -q 'reason=separate-existing-runtime' <<<"$active_runtime_output"
unknown_runtime_fixture="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-unknown-runtime.XXXXXX")"
trap 'rm -f "$fixture" "$clear_fixture" "$sibling_fixture" "$active_runtime_fixture" "$unknown_runtime_fixture"; rm -rf "$fixture_report_repo"' EXIT
printf '%s\n' \
  'PID PPID STAT COMMAND' \
  '32 1 R npm run dev' \
  '33 32 R node ./node_modules/vite/bin/vite.js' \
  > "$unknown_runtime_fixture"
unknown_runtime_output="$(
  MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
  MEMD_HOST_IO_PS_FILE="$unknown_runtime_fixture" \
  memd_cargo_host_blockers
)"
grep -q 'unknown project_hint=active-runtime pid=32 state=R' <<<"$unknown_runtime_output"
grep -q 'unknown project_hint=active-runtime pid=33 state=R' <<<"$unknown_runtime_output"

tooling_fixture="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-tooling.XXXXXX")"
trap 'rm -f "$fixture" "$clear_fixture" "$sibling_fixture" "$active_runtime_fixture" "$unknown_runtime_fixture" "$tooling_fixture"; rm -rf "$fixture_report_repo"' EXIT
printf '%s\n' \
  'PID PPID STAT COMMAND' \
  '34 1 U /Volumes/T7/.cargo/bin/cargo build --manifest-path /tmp/app/Cargo.toml' \
  '35 1 U /Volumes/T7/node-v24.14.0-darwin-arm64/bin/tsc --build /tmp/app/tsconfig.json' \
  > "$tooling_fixture"
tooling_output="$(
  MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
  MEMD_CARGO_VOLUME_ROOT=/Volumes/T7 \
  MEMD_HOST_IO_PS_FILE="$tooling_fixture" \
  memd_cargo_host_blockers
)"
grep -q 'volume:/Volumes/T7 project_hint=cargo-tooling pid=34 state=U' <<<"$tooling_output"
grep -q 'volume:/Volumes/T7 project_hint=node-tooling pid=35 state=U' <<<"$tooling_output"
set +e
tooling_hard_output="$(printf '%s\n' "$tooling_output" | MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd memd_host_io_hard_blockers)"
tooling_hard_status=$?
set -e
if [[ "$tooling_hard_status" -ne 0 ]]; then
  echo "memd host I/O guard test: tooling hard blocker filter failed" >&2
  exit 1
fi
grep -q 'project_hint=cargo-tooling' <<<"$tooling_hard_output"
grep -q 'project_hint=node-tooling' <<<"$tooling_hard_output"
sibling_report_repo="$(mktemp -d "${TMPDIR:-/tmp}/memd-host-io-sibling-report.XXXXXX")"
awareness_report="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-awareness.XXXXXX")"
trap 'rm -f "$fixture" "$clear_fixture" "$sibling_fixture" "$active_runtime_fixture" "$unknown_runtime_fixture" "$tooling_fixture" "$awareness_report"; rm -rf "$fixture_report_repo" "$sibling_report_repo"' EXIT
MEMD_CARGO_REPO_ROOT="$sibling_report_repo" \
MEMD_CARGO_VOLUME_ROOT=/Volumes/T7 \
MEMD_HOST_IO_PS_FILE="$sibling_fixture" \
MEMD_HOST_IO_AWARENESS="$awareness_report" \
memd_cargo_refuse_on_host_blockers >/tmp/memd-host-io-sibling-test.out 2>&1
if [[ -s /tmp/memd-host-io-sibling-test.out ]]; then
  echo "memd host I/O guard test: sibling separate app leaked into default stderr" >&2
  cat /tmp/memd-host-io-sibling-test.out >&2
  exit 1
fi
grep -q 'status=observed' "$awareness_report"
grep -q 'observation_count=1' "$awareness_report"
grep -q 'hard_blocker_count=0' "$awareness_report"
grep -q 'project_hint=clawcontrol' "$awareness_report"
MEMD_CARGO_REPO_ROOT="$sibling_report_repo" \
MEMD_CARGO_VOLUME_ROOT=/Volumes/T7 \
MEMD_HOST_IO_PS_FILE="$active_runtime_fixture" \
MEMD_HOST_IO_AWARENESS="$awareness_report" \
memd_cargo_refuse_on_host_blockers >/tmp/memd-host-active-runtime-test.out 2>&1
if [[ -s /tmp/memd-host-active-runtime-test.out ]]; then
  echo "memd host I/O guard test: separate active runtime leaked into default stderr" >&2
  cat /tmp/memd-host-active-runtime-test.out >&2
  exit 1
fi
grep -q 'status=observed' "$awareness_report"
grep -q 'reason=separate-existing-runtime' "$awareness_report"
MEMD_CARGO_REPO_ROOT="$sibling_report_repo" \
MEMD_CARGO_VOLUME_ROOT=/Volumes/T7 \
MEMD_HOST_IO_PS_FILE="$active_runtime_fixture" \
MEMD_HOST_IO_AWARENESS="$awareness_report" \
MEMD_HOST_IO_SHOW_SIBLING_AWARENESS=1 \
memd_cargo_refuse_on_host_blockers >/tmp/memd-host-active-runtime-explicit-test.out 2>&1
grep -q 'separate existing app activity observed' /tmp/memd-host-active-runtime-explicit-test.out
grep -q 'awareness only, not a memd test/build dependency' /tmp/memd-host-active-runtime-explicit-test.out

set +e
HOST_IO_GUARD_ENABLED=0 \
MEMD_BIN=true \
"$ROOT/scripts/dev-server-guard.sh" --port 59999 -- bash -lc 'cd /Volumes/T7/projects/clawcontrol && cargo tauri dev' \
  >/tmp/memd-dev-server-clawcontrol-refusal-test.out 2>&1
clawcontrol_refusal_status=$?
set -e
if [[ "$clawcontrol_refusal_status" -ne 66 ]]; then
  echo "memd host I/O guard test: dev-server guard did not refuse cross-project ClawControl launch" >&2
  cat /tmp/memd-dev-server-clawcontrol-refusal-test.out >&2
  exit 1
fi
grep -q 'refusing to launch ClawControl from memd' /tmp/memd-dev-server-clawcontrol-refusal-test.out

no_clawcontrol_dir="$(mktemp -d "${TMPDIR:-/tmp}/memd-no-clawcontrol-sync.XXXXXX")"
cat > "$no_clawcontrol_dir/memd" <<'SH'
#!/usr/bin/env bash
if [[ "$1" == "live-state" && "$2" == "status" ]]; then
  if [[ "$*" == *"--check"* ]]; then
    exit 2
  fi
  echo "task source=memd module=messages scope=approved status=missing"
  exit 2
fi
echo "unexpected memd invocation: $*" >&2
exit 64
SH
cat > "$no_clawcontrol_dir/clawcontrol-http" <<'SH'
#!/usr/bin/env bash
echo "clawcontrol http capture should not run by default" >&2
exit 97
SH
chmod +x "$no_clawcontrol_dir/memd" "$no_clawcontrol_dir/clawcontrol-http"
set +e
HOST_IO_GUARD_ENABLED=0 \
MEMD_BIN="$no_clawcontrol_dir/memd" \
CAPTURE_SCRIPT="$no_clawcontrol_dir/clawcontrol-http" \
MAC_BRIDGE_FALLBACK=0 \
APPROVED_COMMUNICATIONS_FALLBACK=0 \
"$ROOT/scripts/live-state-sync-memd.sh" >/tmp/memd-live-state-no-clawcontrol-default.out 2>&1
no_clawcontrol_status=$?
set -e
if [[ "$no_clawcontrol_status" -ne 2 ]]; then
  echo "memd host I/O guard test: memd live-state sync should stop at memd-owned producers" >&2
  sed -n '1,40p' /tmp/memd-live-state-no-clawcontrol-default.out >&2
  exit 1
fi
grep -q 'memd-owned producers unavailable' /tmp/memd-live-state-no-clawcontrol-default.out
if grep -q 'clawcontrol http capture should not run' /tmp/memd-live-state-no-clawcontrol-default.out; then
  echo "memd host I/O guard test: memd live-state sync ran ClawControl HTTP capture" >&2
  exit 1
fi
if grep -q 'live-state-sync-clawcontrol' "$ROOT/scripts/live-state-sync-memd.sh"; then
  echo "memd host I/O guard test: memd sync delegates to ClawControl sync" >&2
  exit 1
fi
if grep -q 'IMPORT_CLAWCONTROL_BUNDLE' "$ROOT/scripts/live-state-sync-memd.sh"; then
  echo "memd host I/O guard test: memd sync exposes ClawControl bundle import" >&2
  exit 1
fi
set +e
HOST_IO_GUARD_ENABLED=0 \
MEMD_BIN="$no_clawcontrol_dir/memd" \
CAPTURE_SCRIPT="$no_clawcontrol_dir/clawcontrol-http" \
MAC_BRIDGE_FALLBACK=0 \
APPROVED_COMMUNICATIONS_FALLBACK=0 \
MEMD_LIVE_STATE_SYNC_DAEMON=1 \
"$ROOT/scripts/live-state-sync-memd.sh" >/tmp/memd-live-state-daemon-blocked.out 2>&1
daemon_blocked_status=$?
set -e
if [[ "$daemon_blocked_status" -ne 0 ]]; then
  echo "memd host I/O guard test: daemon sync should record blockers without launchd failure" >&2
  sed -n '1,40p' /tmp/memd-live-state-daemon-blocked.out >&2
  exit 1
fi
grep -q 'daemon mode: recorded live-state blockers' /tmp/memd-live-state-daemon-blocked.out
if ! "$ROOT/scripts/install-live-state-sync-launchd.sh" --print | grep -q '<key>MEMD_LIVE_STATE_SYNC_DAEMON</key>'; then
  echo "memd host I/O guard test: launchd plist missing daemon mode env" >&2
  exit 1
fi
set +e
HOST_IO_GUARD_ENABLED=0 \
MEMD_BIN=true \
"$ROOT/scripts/live-state-sync-clawcontrol.sh" >/tmp/memd-live-state-clawcontrol-refusal.out 2>&1
clawcontrol_sync_refusal_status=$?
set -e
if [[ "$clawcontrol_sync_refusal_status" -ne 66 ]]; then
  echo "memd host I/O guard test: ClawControl sync did not require explicit opt-in" >&2
  sed -n '1,40p' /tmp/memd-live-state-clawcontrol-refusal.out >&2
  exit 1
fi
grep -q 'refusing by default' /tmp/memd-live-state-clawcontrol-refusal.out
grep -q 'must not launch ClawControl' /tmp/memd-live-state-clawcontrol-refusal.out

set +e
MEMD_AUTHORITY_NETWORK=portainer_default \
"$ROOT/scripts/deploy-memd-authority.sh" build-only >/tmp/memd-authority-network-refusal.out 2>&1
authority_network_refusal_status=$?
set -e
if [[ "$authority_network_refusal_status" -ne 65 ]]; then
  echo "memd host I/O guard test: memd authority deploy accepted shared Portainer network" >&2
  sed -n '1,40p' /tmp/memd-authority-network-refusal.out >&2
  exit 1
fi
grep -q 'MEMD_AUTHORITY_NETWORK=portainer_default is not memd-owned' /tmp/memd-authority-network-refusal.out
grep -q 'memd-authority-network' /tmp/memd-authority-network-refusal.out

fake_ps_dir="$(mktemp -d "${TMPDIR:-/tmp}/memd-host-io-fake-ps.XXXXXX")"
trap 'rm -f "$fixture" "$clear_fixture" "$sibling_fixture" "$active_runtime_fixture"; rm -rf "$fixture_report_repo" "$sibling_report_repo" "$fake_ps_dir" "$no_clawcontrol_dir"' EXIT
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'sleep 5' \
  > "$fake_ps_dir/ps"
chmod +x "$fake_ps_dir/ps"
timeout_output="$(
  MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
  MEMD_HOST_IO_REPORT="$fake_ps_dir/no-report" \
  MEMD_HOST_IO_PS_TIMEOUT_SECS=1 \
  PATH="$fake_ps_dir:$PATH" \
  memd_cargo_host_blockers
)"
grep -q 'project_hint=host-process-scan' <<<"$timeout_output"
grep -q 'state=timeout' <<<"$timeout_output"
grep -q 'timeout_s=1' <<<"$timeout_output"

fresh_report="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-fresh-report.XXXXXX")"
trap 'rm -f "$fixture" "$clear_fixture" "$fresh_report"; rm -rf "$fake_ps_dir" "$no_clawcontrol_dir"' EXIT
cat > "$fresh_report" <<EOF
ts=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
repo=/Volumes/T7/projects/memd
pid=77
status=blocked
repo project_hint=memd pid=77 state=U command=git -C /Volumes/T7/projects/memd status --short
EOF
fresh_report_output="$(
  MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
  MEMD_HOST_IO_REPORT="$fresh_report" \
  MEMD_HOST_IO_REPORT_TTL_SECS=120 \
  PATH="$fake_ps_dir:$PATH" \
  memd_cargo_host_blockers
)"
grep -q 'project_hint=host-io-report' <<<"$fresh_report_output"
grep -q 'state=cached' <<<"$fresh_report_output"
grep -q 'age_s=' <<<"$fresh_report_output"
grep -q 'repo project_hint=memd pid=77 state=U' <<<"$fresh_report_output"
if grep -q 'host-process-scan' <<<"$fresh_report_output"; then
  echo "memd host I/O guard test: fresh report still ran ps scan" >&2
  exit 1
fi
fresh_report_before="$(cat "$fresh_report")"
set +e
MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
MEMD_HOST_IO_REPORT="$fresh_report" \
MEMD_HOST_IO_REPORT_TTL_SECS=120 \
PATH="$fake_ps_dir:$PATH" \
memd_cargo_refuse_on_host_blockers >/tmp/memd-host-io-cached-report-test.out 2>&1
cached_report_status=$?
set -e
if [[ "$cached_report_status" -ne 75 ]]; then
  echo "memd host I/O guard test: cached report did not refuse" >&2
  exit 1
fi
if [[ "$(cat "$fresh_report")" != "$fresh_report_before" ]]; then
  echo "memd host I/O guard test: cached report reuse rewrote report" >&2
  exit 1
fi

stale_report="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-stale-report.XXXXXX")"
trap 'rm -f "$fixture" "$clear_fixture" "$fresh_report" "$stale_report"; rm -rf "$fake_ps_dir" "$no_clawcontrol_dir"' EXIT
cat > "$stale_report" <<'EOF'
ts=2000-01-01T00:00:00Z
repo=/Volumes/T7/projects/memd
pid=88
status=blocked
repo project_hint=memd pid=88 state=U command=git -C /Volumes/T7/projects/memd status --short
EOF
stale_report_output="$(
  MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
  MEMD_HOST_IO_REPORT="$stale_report" \
  MEMD_HOST_IO_REPORT_TTL_SECS=1 \
  MEMD_HOST_IO_PS_TIMEOUT_SECS=1 \
  PATH="$fake_ps_dir:$PATH" \
  memd_cargo_host_blockers
)"
grep -q 'project_hint=host-process-scan' <<<"$stale_report_output"

fake_git_marker="$fake_ps_dir/git-called"
cat > "$fake_ps_dir/git" <<SH
#!/usr/bin/env bash
echo git-called > "$fake_git_marker"
exit 99
SH
chmod +x "$fake_ps_dir/git"
cat > "$fake_ps_dir/curl" <<'SH'
#!/usr/bin/env bash
cat <<'JSON'
{
  "git_branch": "main",
  "git_commit": "69d531b9",
  "git_dirty": "clean",
  "benchmark_gate": "fail",
  "latency_p95_ms": 4096.0
}
JSON
SH
chmod +x "$fake_ps_dir/curl"
continuity_config="$fake_ps_dir/config.json"
continuity_wake="$fake_ps_dir/wake.md"
continuity_awareness="$fake_ps_dir/host-io-awareness.txt"
cat > "$continuity_config" <<'JSON'
{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "session-test",
  "hive_system": "codex",
  "hive_role": "agent",
  "hive_groups": [
    "project:memd",
    "role:test"
  ],
  "hive_project_enabled": true,
  "authority": "participant",
  "capabilities": [
    "coordination",
    "memory"
  ],
  "authority_state": {
    "mode": "shared",
    "shared_base_url": "http://example.invalid",
    "blocked_capabilities": []
  }
}
JSON
cat > "$continuity_wake" <<'EOF'
# wake
- recovery voice=caveman-ultra | quality=ready:0.99 | dirty=0 | next=test: continue continuity
EOF
cat > "$continuity_awareness" <<'EOF'
ts=2026-05-18T05:08:10Z
repo=/Volumes/T7/projects/memd
pid=55
status=observed
observation_count=1
hard_blocker_count=0
volume:/Volumes/T7 project_hint=clawcontrol pid=31 state=S command=node /Volumes/T7/projects/clawcontrol/deploy/agentshell/agent-shell-adapter.js reason=separate-existing-runtime
EOF
preflight_live_map="$fake_ps_dir/preflight-codebase-live-map.json"
continuity_live_map_events="$fake_ps_dir/codebase-live-map-events.ndjson"
continuity_fake_binary="$fake_ps_dir/memd-active"
continuity_fake_source="$fake_ps_dir/memd-active-source.rs"
touch -t 202605180101 "$continuity_fake_binary"
touch -t 202605180102 "$continuity_fake_source"
cat > "$preflight_live_map" <<'JSON'
{
  "repo_root": "/Volumes/T7/projects/memd",
  "fingerprint": "host-io-blocked-no-scan",
  "updated_at": "2026-05-18T05:08:10Z",
  "status": "blocked",
  "needs_reread": true,
  "autosync": "blocked_no_scan"
}
JSON
cat > "$continuity_live_map_events" <<'JSON'
{"ts":"2026-05-18T05:08:11Z","source":"host-io-guard:blocked","status":"blocked","paths":["/Volumes/T7/projects/memd/.memd/state/host-io-guard.txt"],"blocker_count":1,"blockers":["repo project_hint=memd pid=12 state=U command=git status --short"]}
{"ts":"2026-05-18T05:08:12Z","source":"host-io-awareness:observed","status":"observed","paths":["/Volumes/T7/projects/memd/.memd/state/host-io-awareness.txt"],"observation_count":1,"observations":["volume:/Volumes/T7 project_hint=clawcontrol pid=31 state=S command=agent-shell-adapter.js reason=separate-existing-runtime"]}
JSON
deploy_preflight_output="$(
  MEMD_ALLOW_DIRTY_DEPLOY=1 \
  MEMD_HOST_IO_REPORT="$fake_ps_dir/no-report" \
  MEMD_HOST_IO_REPORT_TTL_SECS=1 \
  MEMD_CODEBASE_LIVE_MAP_STATE="$preflight_live_map" \
  MEMD_CODEBASE_LIVE_MAP_EVENTS="$continuity_live_map_events" \
  MEMD_SERVER_STATUS_URL=http://example.invalid/api/status \
  PATH="$fake_ps_dir:$PATH" \
  "$ROOT/scripts/deploy-memd-server-preflight.sh" 2>&1
)"
grep -q 'MEMD_GIT_DIRTY=unknown' <<<"$deploy_preflight_output"
grep -q 'MEMD_AUTHORITY_STACK=memd-authority-stack' <<<"$deploy_preflight_output"
grep -q 'MEMD_AUTHORITY_CONTAINER=memd-authority' <<<"$deploy_preflight_output"
grep -q 'MEMD_AUTHORITY_IMAGE_REPO=memd-authority' <<<"$deploy_preflight_output"
grep -q 'MEMD_AUTHORITY_NETWORK=memd-authority-network' <<<"$deploy_preflight_output"
grep -q 'MEMD_AUTHORITY_DATA_VOLUME=memd_authority_data' <<<"$deploy_preflight_output"
grep -q 'project_hint=host-io-report' <<<"$deploy_preflight_output"
grep -q 'state=missing' <<<"$deploy_preflight_output"
grep -q 'MEMD_SERVER_STATUS=blocked' <<<"$deploy_preflight_output"
grep -q 'MEMD_SERVER_GIT_COMMIT=69d531b9' <<<"$deploy_preflight_output"
grep -q 'MEMD_CODEBASE_LIVE_MAP_STATUS=blocked' <<<"$deploy_preflight_output"
grep -q 'MEMD_CODEBASE_LIVE_MAP_REREAD_REQUIRED=true' <<<"$deploy_preflight_output"
grep -q 'MEMD_CODEBASE_LIVE_MAP_AUTOSYNC=blocked_no_scan' <<<"$deploy_preflight_output"
grep -q 'MEMD_CODEBASE_LIVE_MAP_FINGERPRINT=host-io-blocked-no-scan' <<<"$deploy_preflight_output"
grep -q 'MEMD_CODEBASE_LIVE_MAP_AGE_SECS=' <<<"$deploy_preflight_output"
grep -q 'MEMD_CODEBASE_LIVE_MAP_TTL_SECS=15' <<<"$deploy_preflight_output"
grep -q 'MEMD_CODEBASE_LIVE_MAP_FRESH=false' <<<"$deploy_preflight_output"
grep -q 'MEMD_CODEBASE_LIVE_MAP_ACTION=refresh_host_guard_before_trusting_live_map' <<<"$deploy_preflight_output"
grep -q 'server benchmark_gate=fail latency_p95_ms=4096.0' <<<"$deploy_preflight_output"
known_blocked_deploy_preflight_output="$(
  MEMD_ALLOW_DIRTY_DEPLOY=1 \
  MEMD_HOST_IO_REPORT="$fresh_report" \
  MEMD_HOST_IO_REPORT_TTL_SECS=120 \
  MEMD_CODEBASE_LIVE_MAP_STATE="$preflight_live_map" \
  MEMD_SERVER_STATUS_URL=http://example.invalid/api/status \
  PATH="$fake_ps_dir:$PATH" \
  "$ROOT/scripts/deploy-memd-server-preflight.sh" 2>&1
)"
grep -q 'MEMD_CODEBASE_LIVE_MAP_ACTION=wait_or_coordinate_before_broad_repo_work' <<<"$known_blocked_deploy_preflight_output"
grep -q 'project_hint=host-io-report' <<<"$known_blocked_deploy_preflight_output"
continuity_status_output="$(
  MEMD_ALLOW_DIRTY_DEPLOY=1 \
  MEMD_CONTINUITY_AUTO_HOST_GUARD=0 \
  MEMD_CONTINUITY_CONFIG="$continuity_config" \
  MEMD_CONTINUITY_WAKE="$continuity_wake" \
  MEMD_ACTIVE_MEMD_BINARY="$continuity_fake_binary" \
  MEMD_ACTIVE_MEMD_SOURCE_PATHS="$continuity_fake_source" \
  MEMD_HOST_IO_AWARENESS="$continuity_awareness" \
  MEMD_HOST_IO_REPORT="$fake_ps_dir/no-report" \
  MEMD_HOST_IO_REPORT_TTL_SECS=1 \
  MEMD_CODEBASE_LIVE_MAP_STATE="$preflight_live_map" \
  MEMD_CODEBASE_LIVE_MAP_EVENTS="$continuity_live_map_events" \
  MEMD_SERVER_STATUS_URL=http://example.invalid/api/status \
  PATH="$fake_ps_dir:$PATH" \
  "$ROOT/scripts/memd-continuity-status.sh" 2>&1
)"
grep -q '^memd continuity status$' <<<"$continuity_status_output"
grep -q 'WAKE_RECOVERY=.*next=test: continue continuity' <<<"$continuity_status_output"
grep -q 'CONFIG_PROJECT=memd' <<<"$continuity_status_output"
grep -q 'CONFIG_HIVE_SYSTEM=codex' <<<"$continuity_status_output"
grep -q 'CONFIG_HIVE_ROLE=agent' <<<"$continuity_status_output"
grep -q 'CONFIG_HIVE_GROUPS=project:memd,role:test' <<<"$continuity_status_output"
grep -q 'CONFIG_HIVE_PROJECT_ENABLED=true' <<<"$continuity_status_output"
grep -q 'CONFIG_HIVE_PROJECT_ACTION=project_hive_enabled' <<<"$continuity_status_output"
grep -q 'CONFIG_AUTHORITY_MODE=shared' <<<"$continuity_status_output"
grep -q 'CONFIG_SHARED_BASE_URL=http://example.invalid' <<<"$continuity_status_output"
grep -q 'CONFIG_CAPABILITIES=coordination,memory' <<<"$continuity_status_output"
grep -q '^CONFIG_BLOCKED_CAPABILITIES=$' <<<"$continuity_status_output"
grep -q "ACTIVE_MEMD_BINARY=$continuity_fake_binary" <<<"$continuity_status_output"
grep -q 'ACTIVE_MEMD_BINARY_STATE=stale' <<<"$continuity_status_output"
grep -q 'ACTIVE_MEMD_BINARY_ACTION=rebuild_active_memd_after_host_guard_clear' <<<"$continuity_status_output"
grep -q "ACTIVE_MEMD_SOURCE_NEWEST=$continuity_fake_source" <<<"$continuity_status_output"
grep -q 'PREFLIGHT_EXIT=0' <<<"$continuity_status_output"
grep -q 'MEMD_AUTHORITY_STACK=memd-authority-stack' <<<"$continuity_status_output"
grep -q 'MEMD_AUTHORITY_NETWORK=memd-authority-network' <<<"$continuity_status_output"
grep -q 'MEMD_AUTHORITY_DATA_VOLUME=memd_authority_data' <<<"$continuity_status_output"
grep -q 'MEMD_CODEBASE_LIVE_MAP_ACTION=refresh_host_guard_before_trusting_live_map' <<<"$continuity_status_output"
grep -q 'NEXT_CONTINUITY_ACTION=refresh_host_guard_before_trusting_live_map' <<<"$continuity_status_output"
grep -q 'MEMD_SERVER_BENCHMARK_GATE=fail' <<<"$continuity_status_output"
grep -q "HOST_IO_AWARENESS=$continuity_awareness" <<<"$continuity_status_output"
grep -q 'HOST_IO_AWARENESS_STATUS=OBSERVED' <<<"$continuity_status_output"
grep -q 'HOST_IO_AWARENESS_OBSERVATION_COUNT=1' <<<"$continuity_status_output"
grep -q 'HOST_IO_OBSERVATION_1=.*project_hint=clawcontrol' <<<"$continuity_status_output"
grep -q 'LIVE_MAP_EVENT_1=.*"source":"host-io-guard:blocked".*"blocker_count":1.*project_hint=memd' <<<"$continuity_status_output"
grep -q 'LIVE_MAP_EVENT_2=.*"source":"host-io-awareness:observed".*"observation_count":1.*project_hint=clawcontrol' <<<"$continuity_status_output"

continuity_auto_report="$fake_ps_dir/continuity-auto-host-report.txt"
continuity_auto_guard="$fake_ps_dir/continuity-auto-host-guard.sh"
cat > "$continuity_auto_guard" <<'SH'
#!/usr/bin/env bash
cat > "${MEMD_HOST_IO_REPORT:?}" <<EOF
ts=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
repo=/Volumes/T7/projects/memd
pid=99
status=blocked
repo project_hint=memd pid=99 state=U command=git -C /Volumes/T7/projects/memd status --short
EOF
exit 75
SH
chmod +x "$continuity_auto_guard"
continuity_auto_status_output="$(
  MEMD_ALLOW_DIRTY_DEPLOY=1 \
  MEMD_CONTINUITY_CONFIG="$continuity_config" \
  MEMD_CONTINUITY_WAKE="$continuity_wake" \
  MEMD_CONTINUITY_HOST_GUARD="$continuity_auto_guard" \
  MEMD_ACTIVE_MEMD_BINARY="$continuity_fake_binary" \
  MEMD_ACTIVE_MEMD_SOURCE_PATHS="$continuity_fake_source" \
  MEMD_HOST_IO_REPORT="$continuity_auto_report" \
  MEMD_HOST_IO_REPORT_TTL_SECS=120 \
  MEMD_CODEBASE_LIVE_MAP_STATE="$preflight_live_map" \
  MEMD_SERVER_STATUS_URL=http://example.invalid/api/status \
  PATH="$fake_ps_dir:$PATH" \
  "$ROOT/scripts/memd-continuity-status.sh" 2>&1
)"
grep -q 'HOST_GUARD_REFRESH_EXIT=75' <<<"$continuity_auto_status_output"
grep -q 'MEMD_CODEBASE_LIVE_MAP_ACTION=wait_or_coordinate_before_broad_repo_work' <<<"$continuity_auto_status_output"
grep -q 'NEXT_CONTINUITY_ACTION=wait_or_coordinate_before_broad_repo_work' <<<"$continuity_auto_status_output"

disabled_hive_config="$fake_ps_dir/config-disabled-hive.json"
cat > "$disabled_hive_config" <<'JSON'
{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "session-test",
  "hive_project_enabled": false,
  "authority": "participant",
  "capabilities": [
    "coordination",
    "memory"
  ],
  "authority_state": {
    "mode": "shared",
    "shared_base_url": "http://example.invalid",
    "blocked_capabilities": []
  }
}
JSON
disabled_hive_status_output="$(
  MEMD_ALLOW_DIRTY_DEPLOY=1 \
  MEMD_CONTINUITY_AUTO_HOST_GUARD=0 \
  MEMD_CONTINUITY_CONFIG="$disabled_hive_config" \
  MEMD_CONTINUITY_WAKE="$continuity_wake" \
  MEMD_ACTIVE_MEMD_BINARY="$continuity_fake_binary" \
  MEMD_ACTIVE_MEMD_SOURCE_PATHS="$continuity_fake_source" \
  MEMD_HOST_IO_REPORT="$fake_ps_dir/no-report" \
  MEMD_HOST_IO_REPORT_TTL_SECS=1 \
  MEMD_CODEBASE_LIVE_MAP_STATE="$preflight_live_map" \
  MEMD_SERVER_STATUS_URL=http://example.invalid/api/status \
  PATH="$fake_ps_dir:$PATH" \
  "$ROOT/scripts/memd-continuity-status.sh" 2>&1
)"
grep -q 'CONFIG_HIVE_PROJECT_ACTION=enable_project_hive_before_handoff' <<<"$disabled_hive_status_output"
grep -q 'NEXT_CONTINUITY_ACTION=enable_project_hive_before_handoff' <<<"$disabled_hive_status_output"
if [[ -e "$fake_git_marker" ]]; then
  echo "memd host I/O guard test: deploy preflight ran git despite missing host report" >&2
  exit 1
fi
stale_deploy_preflight_output="$(
  MEMD_ALLOW_DIRTY_DEPLOY=1 \
  MEMD_HOST_IO_REPORT="$stale_report" \
  MEMD_HOST_IO_REPORT_TTL_SECS=1 \
  MEMD_CODEBASE_LIVE_MAP_STATE="$preflight_live_map" \
  MEMD_SERVER_STATUS_URL=http://example.invalid/api/status \
  PATH="$fake_ps_dir:$PATH" \
  "$ROOT/scripts/deploy-memd-server-preflight.sh" 2>&1
)"
grep -q 'project_hint=host-io-report' <<<"$stale_deploy_preflight_output"
grep -q 'state=stale' <<<"$stale_deploy_preflight_output"
grep -q 'age_s=' <<<"$stale_deploy_preflight_output"
grep -q 'ttl_s=1' <<<"$stale_deploy_preflight_output"
if [[ -e "$fake_git_marker" ]]; then
  echo "memd host I/O guard test: deploy preflight ran git despite stale host report" >&2
  exit 1
fi

warmup_fake_dir="$(mktemp -d "${TMPDIR:-/tmp}/memd-status-warmup.XXXXXX")"
trap 'rm -f "$fixture" "$clear_fixture" "$fresh_report" "$stale_report"; rm -rf "$fake_ps_dir" "$no_clawcontrol_dir" "$warmup_fake_dir"' EXIT
warmup_count="$warmup_fake_dir/curl-count"
warmup_ref="$(sed -n 's#^ref: refs/heads/##p' "$ROOT/.git/HEAD" 2>/dev/null || true)"
warmup_commit="unknown"
if [[ -n "$warmup_ref" && -f "$ROOT/.git/refs/heads/$warmup_ref" ]]; then
  warmup_commit="$(cut -c1-8 "$ROOT/.git/refs/heads/$warmup_ref")"
elif [[ -s "$ROOT/.git/HEAD" ]]; then
  warmup_commit="$(cut -c1-8 "$ROOT/.git/HEAD")"
fi
cat > "$warmup_fake_dir/curl" <<'SH'
#!/usr/bin/env bash
if [[ "$*" == *"/healthz"* ]]; then
  printf 'ok\n'
  exit 0
fi
count_file="${MEMD_TEST_CURL_COUNT:?}"
count=0
if [[ -f "$count_file" ]]; then
  count="$(cat "$count_file")"
fi
count=$((count + 1))
printf '%s\n' "$count" > "$count_file"
if [[ "$count" -eq 1 ]]; then
  cat <<JSON
{
  "git_branch": "main",
  "git_commit": "${MEMD_TEST_SERVER_COMMIT:?}",
  "git_dirty": "clean",
  "benchmark_gate": "fail",
  "latency_p95_ms": 2048.0
}
JSON
else
  cat <<JSON
{
  "git_branch": "main",
  "git_commit": "${MEMD_TEST_SERVER_COMMIT:?}",
  "git_dirty": "clean",
  "benchmark_gate": "pass",
  "latency_p95_ms": 64.0
}
JSON
fi
SH
chmod +x "$warmup_fake_dir/curl"
status_warmup_preflight_output="$(
  MEMD_ALLOW_DIRTY_DEPLOY=1 \
  MEMD_HOST_IO_REPORT="$fake_ps_dir/no-report" \
  MEMD_HOST_IO_REPORT_TTL_SECS=1 \
  MEMD_CODEBASE_LIVE_MAP_STATE="$preflight_live_map" \
  MEMD_SERVER_STATUS_URL=http://example.invalid/api/status \
  MEMD_SERVER_STATUS_WARMUP_ATTEMPTS=3 \
  MEMD_SERVER_STATUS_WARMUP_INTERVAL=0 \
  MEMD_TEST_CURL_COUNT="$warmup_count" \
  MEMD_TEST_SERVER_COMMIT="$warmup_commit" \
  PATH="$warmup_fake_dir:$PATH" \
  "$ROOT/scripts/deploy-memd-server-preflight.sh" 2>&1
)"
grep -q 'MEMD_SERVER_STATUS=ready' <<<"$status_warmup_preflight_output"
grep -q "MEMD_SERVER_GIT_COMMIT=$warmup_commit" <<<"$status_warmup_preflight_output"
grep -q 'MEMD_SERVER_BENCHMARK_GATE=pass' <<<"$status_warmup_preflight_output"
grep -q 'MEMD_SERVER_LATENCY_P95_MS=64.0' <<<"$status_warmup_preflight_output"
grep -q 'MEMD_SERVER_STATUS_WARMUP=recovered' <<<"$status_warmup_preflight_output"

stale_authority_wake="$warmup_fake_dir/stale-authority-wake.md"
cat > "$stale_authority_wake" <<'EOF'
# wake
- recovery voice=caveman-ultra | quality=ready:0.99 | dirty=0 | next=test: continue continuity | server_authority_blockers=server benchmark_gate=fail latency_p95_ms=2048; authority is not proven ready
EOF
stale_authority_count="$warmup_fake_dir/continuity-curl-count"
stale_authority_continuity_output="$(
  MEMD_ALLOW_DIRTY_DEPLOY=1 \
  MEMD_CONTINUITY_AUTO_HOST_GUARD=0 \
  MEMD_CONTINUITY_CONFIG="$continuity_config" \
  MEMD_CONTINUITY_WAKE="$stale_authority_wake" \
  MEMD_ACTIVE_MEMD_BINARY="$continuity_fake_binary" \
  MEMD_ACTIVE_MEMD_SOURCE_PATHS="$continuity_fake_source" \
  MEMD_HOST_IO_REPORT="$fake_ps_dir/no-report" \
  MEMD_HOST_IO_REPORT_TTL_SECS=1 \
  MEMD_CODEBASE_LIVE_MAP_STATE="$preflight_live_map" \
  MEMD_SERVER_STATUS_URL=http://example.invalid/api/status \
  MEMD_SERVER_STATUS_WARMUP_ATTEMPTS=3 \
  MEMD_SERVER_STATUS_WARMUP_INTERVAL=0 \
  MEMD_TEST_CURL_COUNT="$stale_authority_count" \
  MEMD_TEST_SERVER_COMMIT="$warmup_commit" \
  PATH="$warmup_fake_dir:$PATH" \
  "$ROOT/scripts/memd-continuity-status.sh" 2>&1
)"
grep -q 'MEMD_SERVER_STATUS=ready' <<<"$stale_authority_continuity_output"
grep -q 'MEMD_SERVER_BENCHMARK_GATE=pass' <<<"$stale_authority_continuity_output"
grep -q 'WAKE_RECOVERY_STALE_FIELDS=server_authority_blockers' <<<"$stale_authority_continuity_output"
grep -q 'WAKE_RECOVERY_ACTION=prefer_current_continuity_status_and_refresh_handoff' <<<"$stale_authority_continuity_output"

fake_guard="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-fake-guard.XXXXXX")"
report="$(mktemp "${TMPDIR:-/tmp}/memd-host-io-report.XXXXXX")"
public_bench_report="$(mktemp "${TMPDIR:-/tmp}/memd-public-bench-host-io-report.XXXXXX")"
live_map_events="$(mktemp "${TMPDIR:-/tmp}/memd-codebase-live-map-events.XXXXXX")"
live_map_awareness="$(mktemp "${TMPDIR:-/tmp}/memd-codebase-live-map-awareness.XXXXXX")"
live_map_state="$fake_ps_dir/codebase-live-map.json"
trap 'rm -f "$fixture" "$clear_fixture" "$fresh_report" "$stale_report" "$fake_guard" "$report" "$public_bench_report" "$live_map_events" "$live_map_awareness"; rm -rf "$fake_ps_dir" "$no_clawcontrol_dir" "$warmup_fake_dir"' EXIT
cat > "$fake_guard" <<'SH'
#!/usr/bin/env bash
echo "fake guard blocked host work" >&2
exit 75
SH
chmod +x "$fake_guard"

set +e
HOST_IO_GUARD="$fake_guard" "$ROOT/scripts/live-state-sync-memd.sh" >/tmp/memd-live-state-guard-test.out 2>&1
live_state_status=$?
set -e
if [[ "$live_state_status" -ne 75 ]]; then
  echo "memd host I/O guard test: live-state sync did not stop at guard" >&2
  sed -n '1,20p' /tmp/memd-live-state-guard-test.out >&2
  exit 1
fi
grep -q 'fake guard blocked host work' /tmp/memd-live-state-guard-test.out

set +e
HOST_IO_GUARD="$fake_guard" \
MEMD_LIVE_STATE_SYNC_DAEMON=1 \
"$ROOT/scripts/live-state-sync-memd.sh" >/tmp/memd-live-state-daemon-guard-test.out 2>&1
live_state_daemon_guard_status=$?
set -e
if [[ "$live_state_daemon_guard_status" -ne 0 ]]; then
  echo "memd host I/O guard test: daemon live-state sync should not poison launchd on guard block" >&2
  sed -n '1,20p' /tmp/memd-live-state-daemon-guard-test.out >&2
  exit 1
fi
grep -q 'fake guard blocked host work' /tmp/memd-live-state-daemon-guard-test.out
grep -q 'daemon mode: host I/O guard blocked this run' /tmp/memd-live-state-daemon-guard-test.out

set +e
HOST_IO_GUARD="$fake_guard" "$ROOT/scripts/dev-server-guard.sh" --port 1 -- true >/tmp/memd-dev-server-guard-test.out 2>&1
dev_server_status=$?
set -e
if [[ "$dev_server_status" -ne 75 ]]; then
  echo "memd host I/O guard test: dev-server guard did not stop at guard" >&2
  sed -n '1,20p' /tmp/memd-dev-server-guard-test.out >&2
  exit 1
fi
grep -q 'fake guard blocked host work' /tmp/memd-dev-server-guard-test.out

set +e
HOST_IO_GUARD="$fake_guard" MEMD_PREFIX="${TMPDIR:-/tmp}/memd-install-guard-test" "$ROOT/scripts/install-memd.sh" >/tmp/memd-install-guard-test.out 2>&1
install_status=$?
set -e
if [[ "$install_status" -ne 75 ]]; then
  echo "memd host I/O guard test: install script did not stop at guard" >&2
  sed -n '1,20p' /tmp/memd-install-guard-test.out >&2
  exit 1
fi
grep -q 'fake guard blocked host work' /tmp/memd-install-guard-test.out

set +e
MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
MEMD_HOST_IO_PS_FILE="$fixture" \
MEMD_HOST_IO_REPORT="$public_bench_report" \
"$ROOT/scripts/public-bench-reproduce.sh" longmemeval >/tmp/memd-public-bench-guard-test.out 2>&1
public_bench_status=$?
set -e
if [[ "$public_bench_status" -ne 75 ]]; then
  echo "memd host I/O guard test: public bench did not stop at guard" >&2
  sed -n '1,20p' /tmp/memd-public-bench-guard-test.out >&2
  exit 1
fi
grep -q 'host I/O blockers visible' /tmp/memd-public-bench-guard-test.out

set +e
HOST_IO_GUARD="$fake_guard" "$ROOT/scripts/repo-hygiene-report.sh" "$ROOT" >/tmp/memd-repo-hygiene-report-guard-test.out 2>&1
repo_hygiene_report_status=$?
set -e
if [[ "$repo_hygiene_report_status" -ne 75 ]]; then
  echo "memd host I/O guard test: repo hygiene report did not stop at guard" >&2
  sed -n '1,20p' /tmp/memd-repo-hygiene-report-guard-test.out >&2
  exit 1
fi
grep -q 'fake guard blocked host work' /tmp/memd-repo-hygiene-report-guard-test.out

set +e
HOST_IO_GUARD="$fake_guard" "$ROOT/scripts/backlog-index.sh" --out /tmp/memd-backlog-index-guard.md >/tmp/memd-backlog-index-guard-test.out 2>&1
backlog_index_status=$?
set -e
if [[ "$backlog_index_status" -ne 75 ]]; then
  echo "memd host I/O guard test: backlog index did not stop at guard" >&2
  sed -n '1,20p' /tmp/memd-backlog-index-guard-test.out >&2
  exit 1
fi
grep -q 'fake guard blocked host work' /tmp/memd-backlog-index-guard-test.out

set +e
HOST_IO_GUARD="$fake_guard" "$ROOT/scripts/sync-integration-hooks.sh" >/tmp/memd-sync-integration-hooks-guard-test.out 2>&1
sync_hooks_status=$?
set -e
if [[ "$sync_hooks_status" -ne 75 ]]; then
  echo "memd host I/O guard test: sync-integration-hooks did not stop at guard" >&2
  sed -n '1,20p' /tmp/memd-sync-integration-hooks-guard-test.out >&2
  exit 1
fi
grep -q 'fake guard blocked host work' /tmp/memd-sync-integration-hooks-guard-test.out

set +e
MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
MEMD_HOST_IO_PS_FILE="$fixture" \
MEMD_HOST_IO_REPORT="$report" \
MEMD_HOST_IO_AWARENESS="$live_map_awareness" \
MEMD_HOST_IO_LIVE_MAP_EVENTS="$live_map_events" \
MEMD_CODEBASE_LIVE_MAP_STATE="$live_map_state" \
memd_cargo_refuse_on_host_blockers >/tmp/memd-host-io-report-test.out 2>&1
report_status=$?
set -e
if [[ "$report_status" -ne 75 ]]; then
  echo "memd host I/O guard test: report blocker did not return 75" >&2
  exit 1
fi
grep -q 'status=blocked' "$report"
grep -q '^pid=' "$report"
grep -q 'project_hint=memd' "$report"
if grep -q 'project_hint=clawcontrol' "$report"; then
  echo "memd host I/O guard test: sibling project leaked into hard blocker report" >&2
  exit 1
fi
grep -q '"source":"host-io-guard:blocked"' "$live_map_events"
grep -q '"status":"blocked"' "$live_map_events"
grep -q '"blocker_count":4' "$live_map_events"
grep -q '"blockers":\[.*project_hint=memd' "$live_map_events"
grep -q "$report" "$live_map_events"
grep -q '"source":"host-io-awareness:blocked"' "$live_map_events"
grep -q '"observation_count":5' "$live_map_events"
grep -q '"observations":\[.*project_hint=clawcontrol' "$live_map_events"
grep -q '"status": "blocked"' "$live_map_state"
grep -q '"needs_reread": true' "$live_map_state"
grep -q '"autosync": "blocked_no_scan"' "$live_map_state"
grep -q 'project_hint=memd' "$live_map_state"
if grep -q 'project_hint=clawcontrol' "$live_map_state"; then
  echo "memd host I/O guard test: sibling project leaked into blocked live map" >&2
  exit 1
fi

MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
MEMD_HOST_IO_PS_FILE="$clear_fixture" \
MEMD_HOST_IO_REPORT="$report" \
MEMD_HOST_IO_AWARENESS="$live_map_awareness" \
MEMD_HOST_IO_LIVE_MAP_EVENTS="$live_map_events" \
MEMD_CODEBASE_LIVE_MAP_STATE="$live_map_state" \
memd_cargo_refuse_on_host_blockers
grep -q 'status=clear' "$report"
grep -q '^pid=' "$report"
grep -q '"source":"host-io-guard:clear"' "$live_map_events"
grep -q '"status":"clear"' "$live_map_events"
grep -q '"blocker_count":0' "$live_map_events"
grep -q '"fingerprint": "host-io-clear-no-scan"' "$live_map_state"
grep -q '"status": "out_of_sync"' "$live_map_state"
grep -q '"autosync": "host_io_clear_rescan_required"' "$live_map_state"
grep -q '"needs_reread": true' "$live_map_state"
if grep -q 'project_hint=clawcontrol' "$live_map_state"; then
  echo "memd host I/O guard test: clear live map retained stale blocker" >&2
  exit 1
fi

MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
MEMD_HOST_IO_PS_FILE="$fixture" \
MEMD_HOST_IO_REPORT="$report" \
MEMD_HOST_IO_AWARENESS="$live_map_awareness" \
MEMD_HOST_IO_LIVE_MAP_EVENTS="$live_map_events" \
MEMD_CODEBASE_LIVE_MAP_STATE="$live_map_state" \
memd_cargo_refuse_on_host_blockers >/tmp/memd-host-io-report-test.out 2>&1 || true
grep -q '"fingerprint": "host-io-blocked-no-scan"' "$live_map_state"
grep -q '"status": "blocked"' "$live_map_state"
grep -q 'project_hint=memd' "$live_map_state"
if grep -q 'project_hint=clawcontrol' "$live_map_state"; then
  echo "memd host I/O guard test: sibling project leaked into second blocked live map" >&2
  exit 1
fi

printf '{\n  "fingerprint": "real-rust-map",\n  "status": "fresh"\n}\n' > "$live_map_state"
MEMD_CARGO_REPO_ROOT=/Volumes/T7/projects/memd \
MEMD_HOST_IO_PS_FILE="$fixture" \
MEMD_HOST_IO_REPORT="$report" \
MEMD_HOST_IO_AWARENESS="$live_map_awareness" \
MEMD_HOST_IO_LIVE_MAP_EVENTS="$live_map_events" \
MEMD_CODEBASE_LIVE_MAP_STATE="$live_map_state" \
memd_cargo_refuse_on_host_blockers >/tmp/memd-host-io-report-test.out 2>&1 || true
grep -q '"fingerprint": "real-rust-map"' "$live_map_state"
if grep -q 'project_hint=clawcontrol' "$live_map_state"; then
  echo "memd host I/O guard test: guard overwrote non-guard live map" >&2
  exit 1
fi
if find "$(dirname "$report")" -maxdepth 1 -name '.host-io-guard.*' | grep -q .; then
  echo "memd host I/O guard test: temp report file leaked" >&2
  exit 1
fi

echo "memd host I/O guard test: ok"
