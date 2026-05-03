#!/usr/bin/env bash
# memd public benchmark — third-party reproduce script.
# Anyone can clone the repo and run this to land within ±0.03 of the
# canonical V6 numbers in docs/verification/PUBLIC_BENCHMARKS.md.
#
# Status: scaffold-symmetric. Until V5 calendar gate (2026-05-02) the
# script exercises the V6 build path end-to-end without claiming
# canonical lifts; runtime activation graduates with A6.9/B6/C6/D6/E6.
#
# Usage:
#   scripts/public-bench-reproduce.sh [bench]
#   scripts/public-bench-reproduce.sh --all [--regenerate-10star] [--allow-below-target]
#
# bench ∈ { longmemeval | locomo | membench | convomem }
# (alias `lme` accepted for longmemeval)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUTPUT="${OUTPUT:-${REPO_ROOT}/.memd/benchmarks/public/results}"
REPORT="${REPORT:-${REPO_ROOT}/docs/verification/PUBLIC_BENCHMARKS.md}"
TYPED_INGEST="episodic+semantic+canonical"
COMPILER="on"
DEPTH_ROUTING="on"
REASONING="on"
ALL=0
REGEN_REPORT=0
REGEN_10STAR=0
ALLOW_BELOW=0
BENCH=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all) ALL=1; shift ;;
    --regenerate-report) REGEN_REPORT=1; shift ;;
    --regenerate-10star) REGEN_10STAR=1; shift ;;
    --allow-below-target) ALLOW_BELOW=1; shift ;;
    --output) OUTPUT="$2"; shift 2 ;;
    --report) REPORT="$2"; shift 2 ;;
    -h|--help)
      sed -n '2,17p' "$0"
      exit 0
      ;;
    lme) BENCH="longmemeval"; shift ;;
    longmemeval|locomo|membench|convomem) BENCH="$1"; shift ;;
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

run_one_bench() {
  local b="$1"
  echo ">>> running $b (typed-ingest=$TYPED_INGEST compiler=$COMPILER depth-routing=$DEPTH_ROUTING reasoning=$REASONING)" >&2
  "$BIN" benchmark public "$b" \
    --typed-ingest "$TYPED_INGEST" \
    --compiler "$COMPILER" \
    --depth-routing "$DEPTH_ROUTING" \
    --reasoning "$REASONING" \
    --out "$OUTPUT"
}

if [[ "$ALL" -eq 1 ]]; then
  for b in longmemeval locomo membench convomem; do
    run_one_bench "$b"
  done
  if [[ "$REGEN_REPORT" -eq 1 ]]; then
    echo ">>> regenerating PUBLIC_BENCHMARKS.md" >&2
    "$BIN" benchmark public --regenerate-report --out "$REPORT"
  fi
  if [[ "$REGEN_10STAR" -eq 1 ]]; then
    echo ">>> regenerating MEMD-10-STAR.md" >&2
    REGEN_ARGS=(bench public --regenerate-10star)
    if [[ "$ALLOW_BELOW" -eq 1 ]]; then
      REGEN_ARGS+=(--allow-below-target)
    fi
    "$BIN" "${REGEN_ARGS[@]}"
  fi
  exit 0
fi

if [[ -z "$BENCH" ]]; then
  echo "usage: $(basename "$0") [--all] [bench]" >&2
  exit 64
fi

run_one_bench "$BENCH"
