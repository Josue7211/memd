#!/usr/bin/env bash
# A5 substrate benchmark — third-party reproduce script.
# Anyone can clone the repo and run this to land within ±0.03 of the
# canonical floor in docs/verification/substrate-baselines/a5-*.json.
#
# Usage: scripts/substrate-bench-reproduce.sh [--seed N] [--output DIR]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SEED="${SEED:-42}"
OUTPUT="${OUTPUT:-${REPO_ROOT}/.memd/benchmarks/substrate/results}"
REPORT="${REPORT:-${REPO_ROOT}/docs/verification/SUBSTRATE_BENCHMARKS.md}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --seed) SEED="$2"; shift 2 ;;
    --output) OUTPUT="$2"; shift 2 ;;
    --report) REPORT="$2"; shift 2 ;;
    -h|--help)
      sed -n '2,8p' "$0"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 64 ;;
  esac
done

mkdir -p "$OUTPUT"

echo ">>> building memd-client (release)" >&2
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/memd-target}" \
  cargo build --release --target-dir "${CARGO_TARGET_DIR:-/tmp/memd-target}" -p memd-client --bin memd

BIN="${CARGO_TARGET_DIR:-/tmp/memd-target}/release/memd"
if [[ ! -x "$BIN" ]]; then
  echo "memd binary not found at $BIN" >&2
  exit 3
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
