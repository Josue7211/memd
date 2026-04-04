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

exec memd --base-url "$MEMD_BASE_URL" hook spill "$@"
