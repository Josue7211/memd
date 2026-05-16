#!/usr/bin/env bash
# Larger external public-dataset proof run. Uses the same upstream adapters and
# real memd-server path as the smoke gate, but raises the item count by default.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

: "${PUBLIC_BENCH_LIMIT:=10}"
: "${PUBLIC_BENCH_OFFSET:=0}"
: "${PUBLIC_BENCH_TIMEOUT:=1800}"
: "${RUN_LABEL:=external-public-scale-${PUBLIC_BENCH_LIMIT}-offset-${PUBLIC_BENCH_OFFSET}}"
: "${SUITE_NAME:=25_5_external_public_scale}"

export PUBLIC_BENCH_LIMIT
export PUBLIC_BENCH_OFFSET
export PUBLIC_BENCH_TIMEOUT
export RUN_LABEL
export SUITE_NAME

exec "$ROOT/scripts/verify/25-5-external-public-smoke.sh"
