#!/usr/bin/env bash
# memd substrate benchmark — third-party reproduce script.
# Anyone can clone the repo and run this to land within ±0.03 of the
# canonical floors in docs/verification/substrate-baselines/*.json.
#
# Modes:
#   default    Run A5 cross-session-recall (legacy single-suite call).
#   --all      Run every V5 substrate suite via the G5 aggregator gate
#              and regenerate SUBSTRATE_BENCHMARKS.md. Exits non-zero
#              if any suite fails the pass-gate.
#
# Usage:
#   scripts/substrate-bench-reproduce.sh [--seed N] [--output DIR] [--report PATH]
#   scripts/substrate-bench-reproduce.sh --all [--seed N] [--report PATH]
#                                              [--regenerate-10star]
#                                              [--allow-below-target]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
source "$REPO_ROOT/scripts/lib/memd-cargo-env.sh"
SEED="${SEED:-42}"
OUTPUT="${OUTPUT:-${REPO_ROOT}/.memd/benchmarks/substrate/results}"
REPORT="${REPORT:-${REPO_ROOT}/docs/verification/SUBSTRATE_BENCHMARKS.md}"
ALL=0
REGEN_10STAR=0
ALLOW_BELOW=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all) ALL=1; shift ;;
    --seed) SEED="$2"; shift 2 ;;
    --output) OUTPUT="$2"; shift 2 ;;
    --report) REPORT="$2"; shift 2 ;;
    --regenerate-10star) REGEN_10STAR=1; shift ;;
    --allow-below-target) ALLOW_BELOW=1; shift ;;
    -h|--help)
      sed -n '2,18p' "$0"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 64 ;;
  esac
done

memd_cargo_refuse_on_host_blockers
mkdir -p "$OUTPUT"

echo ">>> building memd-client (release)" >&2
cargo build --release --target-dir "$MEMD_CARGO_TARGET_DIR" -p memd-client --bin memd

BIN="$MEMD_CARGO_TARGET_DIR/release/memd"
if [[ ! -x "$BIN" ]]; then
  echo "memd binary not found at $BIN" >&2
  exit 3
fi

if [[ "$ALL" -eq 1 ]]; then
  ARGS=(benchmark substrate --all --output "$OUTPUT" --report "$REPORT" --regenerate-report)
  if [[ -n "${SEED:-}" ]]; then
    ARGS+=(--seed "$SEED")
  fi
  if [[ "$REGEN_10STAR" -eq 1 ]]; then
    ARGS+=(--regenerate-10star)
  fi
  if [[ "$ALLOW_BELOW" -eq 1 ]]; then
    ARGS+=(--allow-below-target)
  fi
  echo ">>> running V5 aggregator gate (--all)" >&2
  "$BIN" "${ARGS[@]}"
  EXIT=$?
  echo ">>> aggregator exit code: $EXIT" >&2
  echo ">>> SUBSTRATE_BENCHMARKS.md regenerated at: $REPORT" >&2
  exit $EXIT
fi

echo ">>> running A5 cross-session-recall (seed=$SEED)" >&2
"$BIN" benchmark substrate \
  --suite cross-session-recall \
  --seed "$SEED" \
  --output "$OUTPUT" \
  --report "$REPORT"

EXIT=$?
echo ">>> A5 reproduce exit code: $EXIT" >&2
echo ">>> NDJSON written under: $OUTPUT" >&2
exit $EXIT
