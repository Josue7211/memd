#!/usr/bin/env bash
set -euo pipefail

MEMD_BASE_URL="${MEMD_BASE_URL:-http://127.0.0.1:8787}"

exec memd --base-url "$MEMD_BASE_URL" hook spill "$@"
