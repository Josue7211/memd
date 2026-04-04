#!/usr/bin/env bash
set -euo pipefail

load_bundle_env() {
  local bundle_root="${MEMD_BUNDLE_ROOT:-.memd}"
  local env_file="$bundle_root/env"
  if [ -f "$env_file" ]; then
    set -a
    # shellcheck disable=SC1090
    source "$env_file"
    set +a
  fi
}

load_bundle_env

MEMD_BASE_URL="${MEMD_BASE_URL:-http://127.0.0.1:8787}"
MEMD_PROJECT="${MEMD_PROJECT:?MEMD_PROJECT is required}"
MEMD_AGENT="${MEMD_AGENT:?MEMD_AGENT is required}"
MEMD_ROUTE="${MEMD_ROUTE:-auto}"
MEMD_INTENT="${MEMD_INTENT:-general}"
MEMD_LIMIT="${MEMD_LIMIT:-8}"
MEMD_MAX_CHARS="${MEMD_MAX_CHARS:-280}"

exec memd --base-url "$MEMD_BASE_URL" hook context \
  --project "$MEMD_PROJECT" \
  --agent "$MEMD_AGENT" \
  --route "$MEMD_ROUTE" \
  --intent "$MEMD_INTENT" \
  --limit "$MEMD_LIMIT" \
  --max-chars-per-item "$MEMD_MAX_CHARS"
