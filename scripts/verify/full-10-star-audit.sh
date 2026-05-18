#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT/scripts/lib/memd-cargo-env.sh"
memd_cargo_refuse_on_host_blockers
RUN_DATE="${RUN_DATE:-$(date +%F)}"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/full-10-star-audit}"
LOG_DIR="$OUT_DIR/$RUN_DATE"
JOBS="${CARGO_BUILD_JOBS:-1}"
export CARGO_TERM_COLOR=never
export CARGO_BUILD_JOBS="$JOBS"

mkdir -p "$LOG_DIR"

run() {
  local name="$1"
  shift
  printf '==> %s\n' "$name"
  (
    cd "$ROOT"
    "$@"
  ) >"$LOG_DIR/$name.log" 2>&1
}

run apps-build bash -lc 'cd apps && npm run build'
run dashboard-build bash -lc 'cd apps/dashboard && npm run build'
run cargo-fmt cargo fmt --check
run cargo-test cargo test --workspace --quiet
run cargo-clippy cargo clippy --workspace --all-targets -- -D warnings

for suite in \
  scripts/verify/v8-operator-proof.sh \
  scripts/verify/v9-adversarial-suite.sh \
  scripts/verify/v10-self-improvement-suite.sh \
  scripts/verify/v11-compiler-sota-suite.sh \
  scripts/verify/v12-interop-sota-suite.sh \
  scripts/verify/v13-release-suite.sh \
  scripts/verify/v14-telemetry-suite.sh \
  scripts/verify/v15-self-tuning-suite.sh \
  scripts/verify/v16-sync-suite.sh \
  scripts/verify/v17-routine-marketplace-suite.sh \
  scripts/verify/v18-correction-graph-suite.sh \
  scripts/verify/v19-zk-provenance-suite.sh \
  scripts/verify/v20-release-suite.sh
do
  run "$(basename "$suite" .sh)" "$ROOT/$suite"
done

cat >"$LOG_DIR/summary.json" <<JSON
{
  "ok": true,
  "run_date": "$RUN_DATE",
  "logs": "${LOG_DIR#$ROOT/}",
  "note": "Executable local/proof gates passed. Real dogfood, auditor, and third-party replay windows remain non-synthetic release gates."
}
JSON

printf '%s\n' "$LOG_DIR/summary.json"
