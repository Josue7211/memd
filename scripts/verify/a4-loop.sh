#!/usr/bin/env bash
# A4 — loop the compaction-survival + breach-detection scenarios N times to
# gate the 10-STAR axis-1 rescore. No network, no LLM, deterministic.
#
# Usage:
#   scripts/verify/a4-loop.sh            # 10 iterations (default)
#   scripts/verify/a4-loop.sh 25         # 25 iterations
#
# Exit 0 iff every iteration passes. On failure, prints cargo output and exits
# with the first non-zero rc.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT/scripts/lib/memd-cargo-env.sh"
memd_cargo_refuse_on_host_blockers
ITERATIONS="${1:-10}"
TARGET_DIR="${CARGO_TARGET_DIR:-$MEMD_CARGO_TARGET_DIR}"
TESTS="continuity_compaction_tests::a4_compaction_survival_5_files \
continuity_compaction_tests::a4_compaction_breach_detection"

cd "$ROOT"

export CARGO_TARGET_DIR="$TARGET_DIR"

pass=0
fail=0
for i in $(seq 1 "$ITERATIONS"); do
  if cargo test --quiet -p memd-client --bin memd -- $TESTS >/dev/null 2>&1; then
    pass=$((pass + 1))
    printf "[%02d/%s] PASS\n" "$i" "$ITERATIONS"
  else
    fail=$((fail + 1))
    printf "[%02d/%s] FAIL — rerun verbose:\n" "$i" "$ITERATIONS"
    cargo test -p memd-client --bin memd -- $TESTS || true
    break
  fi
done

echo "----"
echo "A4 loop result: pass=${pass}/${ITERATIONS} fail=${fail}"
if [[ "$fail" -ne 0 ]]; then
  exit 1
fi
