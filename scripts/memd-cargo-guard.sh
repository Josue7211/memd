#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
  cat >&2 <<'USAGE'
usage: scripts/memd-cargo-guard.sh <cargo-args...>

Runs Cargo for memd with memd-owned cache/target directories so hive agents do
not collide with ClawControl or another project's Cargo package cache.
USAGE
  exit 2
fi

if [[ "${1:-}" == "--" ]]; then
  shift
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export MEMD_CARGO_REPO_ROOT="${MEMD_CARGO_REPO_ROOT:-$ROOT}"
export MEMD_HOST_IO_GUARD_LABEL="${MEMD_HOST_IO_GUARD_LABEL:-memd cargo guard}"
source "$ROOT/scripts/lib/memd-cargo-env.sh"
MEMD_AR="${MEMD_AR:-$ROOT/scripts/memd-ar-wrapper.sh}"

cd "$ROOT"
memd_cargo_refuse_on_host_blockers
exec env \
  CARGO_HOME="$MEMD_CARGO_HOME" \
  CARGO_TARGET_DIR="$MEMD_CARGO_TARGET_DIR" \
  CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}" \
  CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}" \
  AR="${AR:-$MEMD_AR}" \
  HOST_AR="${HOST_AR:-$MEMD_AR}" \
  cargo "$@"
