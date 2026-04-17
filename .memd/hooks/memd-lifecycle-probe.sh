#!/usr/bin/env bash
# A3-D3 working-memory lifecycle self-test probe.
# Runs store → recall → expire → verify-expired against the live memd server.
# Exits 0 on green, 1 on red. Safe to wire from cron, wake, or checkpoint.
set -euo pipefail

BUNDLE_ROOT="${MEMD_BUNDLE_ROOT:-.memd}"
BASE_URL="${MEMD_BASE_URL:-http://127.0.0.1:8787}"

exec memd --base-url "${BASE_URL}" diagnostics lifecycle-probe \
    --output "${BUNDLE_ROOT}" \
    --summary
