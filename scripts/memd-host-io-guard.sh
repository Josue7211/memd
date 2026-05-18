#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export MEMD_CARGO_REPO_ROOT="${MEMD_CARGO_REPO_ROOT:-$ROOT}"
export MEMD_HOST_IO_GUARD_LABEL="${MEMD_HOST_IO_GUARD_LABEL:-memd host I/O guard}"

source "$ROOT/scripts/lib/memd-cargo-env.sh"

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'USAGE'
usage: scripts/memd-host-io-guard.sh

Checks for same-repo uninterruptible host I/O blockers, filesystem trouble, and
unknown app/tooling I/O before broad Git, Cargo, test, or scan work. Sibling
project I/O is reported as awareness but does not hard-block by default. Exits
0 when clear enough, 75 when work should wait for host I/O recovery.
USAGE
  exit 0
fi

memd_cargo_refuse_on_host_blockers
