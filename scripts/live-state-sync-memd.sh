#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export CAPTURE_HTTP="${CAPTURE_HTTP:-0}"
export IMPORT_CLAWCONTROL_BUNDLE="${IMPORT_CLAWCONTROL_BUNDLE:-0}"

exec "$ROOT/scripts/live-state-sync-clawcontrol.sh" "$@"
