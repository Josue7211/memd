#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail() {
  echo "hive live-map guard contract: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$ROOT/$path" ]] || fail "missing $path"
}

require_grep() {
  local pattern="$1"
  local path="$2"
  grep -Eq "$pattern" "$ROOT/$path" || fail "missing pattern in $path: $pattern"
}

require_absent() {
  local pattern="$1"
  local path="$2"
  if grep -Eq "$pattern" "$ROOT/$path"; then
    fail "forbidden pattern in $path: $pattern"
  fi
}

require_file "docs/contracts/hive-live-map-guard.md"
require_file "docs/contracts/memd-authority-deploy.md"
require_file "AGENTS.md"
require_file "scripts/dev-server-guard.sh"
require_file "scripts/memd-cargo-guard.sh"
require_file "scripts/lib/memd-cargo-env.sh"
require_file "scripts/deploy-memd-authority.sh"
require_file "scripts/live-state-sync-memd.sh"
require_file "scripts/live-state-sync-clawcontrol.sh"
require_file "scripts/test-host-io-guard.sh"
require_file "crates/memd-client/src/bundle/maintenance_runtime/mod.rs"
require_file "crates/memd-client/src/render/render_summary.rs"

require_grep "memd is the coordination/memory runtime" "docs/contracts/hive-live-map-guard.md"
require_grep "separate projects, separate lifecycles" "docs/contracts/hive-live-map-guard.md"
require_grep "run \`cargo tauri dev\`" "docs/contracts/hive-live-map-guard.md"
require_grep "codebase-live-map\\.json" "docs/contracts/hive-live-map-guard.md"
require_grep "codebase-live-map-events\\.ndjson" "docs/contracts/hive-live-map-guard.md"
require_grep "scripts/memd-host-io-guard\\.sh" "docs/contracts/hive-live-map-guard.md"
require_grep "scripts/memd-cargo-guard\\.sh -- <cargo args>" "docs/contracts/hive-live-map-guard.md"
require_grep "reason=separate-existing-runtime" "docs/contracts/hive-live-map-guard.md"
require_grep "must not launch ClawControl" "docs/contracts/hive-live-map-guard.md"
require_grep "user-copyable next-agent prompt" "docs/contracts/hive-live-map-guard.md"

require_grep "Contract: \`docs/contracts/hive-live-map-guard.md\`" "AGENTS.md"
require_grep "scripts/memd-cargo-guard\\.sh -- <cargo args>" "AGENTS.md"
require_grep "codebase-live-map\\.json" "AGENTS.md"
require_grep "Do not kill or restart ClawControl" "AGENTS.md"

require_grep "refusing to launch ClawControl from memd" "scripts/dev-server-guard.sh"
require_grep "MEMD_ALLOW_CLAWCONTROL_DEV_SERVER" "scripts/dev-server-guard.sh"
require_grep "memd_cargo_refuse_on_host_blockers" "scripts/memd-cargo-guard.sh"
require_grep 'CARGO_HOME="\$MEMD_CARGO_HOME"' "scripts/memd-cargo-guard.sh"
require_grep 'CARGO_TARGET_DIR="\$MEMD_CARGO_TARGET_DIR"' "scripts/memd-cargo-guard.sh"
require_grep "reason=separate-existing-runtime" "scripts/lib/memd-cargo-env.sh"
require_grep "project_hint=host-process-scan" "scripts/lib/memd-cargo-env.sh"
require_grep "project_hint=clawcontrol" "scripts/test-host-io-guard.sh"
require_grep "refusing to launch ClawControl from memd" "scripts/test-host-io-guard.sh"
require_grep "memd sync delegates to ClawControl sync" "scripts/test-host-io-guard.sh"
require_grep "must not launch ClawControl" "scripts/test-host-io-guard.sh"

require_grep "touches_clawcontrol=false" "scripts/deploy-memd-authority.sh"
require_grep "starts_clawcontrol=false" "scripts/deploy-memd-authority.sh"
require_grep "stops_clawcontrol=false" "scripts/deploy-memd-authority.sh"
require_grep "memd-authority-network" "scripts/deploy-memd-authority.sh"
require_grep "memd_authority_data" "scripts/deploy-memd-authority.sh"
require_grep "clawcontrol-\\*" "scripts/deploy-memd-authority.sh"

require_grep "live-state-sync-clawcontrol" "scripts/live-state-sync-clawcontrol.sh"
require_grep "refusing by default" "scripts/live-state-sync-clawcontrol.sh"
require_grep "This script must not launch ClawControl" "scripts/live-state-sync-clawcontrol.sh"
require_absent "live-state-sync-clawcontrol" "scripts/live-state-sync-memd.sh"
require_absent "IMPORT_CLAWCONTROL_BUNDLE" "scripts/live-state-sync-memd.sh"

require_grep "test heartbeat publication to shared memd authority is blocked" "crates/memd-client/src/bundle/maintenance_runtime/mod.rs"
require_grep "## User Prompt" "crates/memd-client/src/render/render_summary.rs"
require_grep "give next agent:" "crates/memd-client/src/render/render_summary.rs"
require_grep "do not launch ClawControl, Tauri, Vite, or app dev servers" "crates/memd-client/src/render/render_summary.rs"

echo "hive live-map guard contract: ok"
