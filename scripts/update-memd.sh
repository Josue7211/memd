#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=0
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ -d .memd ]]; then
  echo "update: memory bundle detected at $ROOT/.memd (will preserve)"
else
  echo "update: no project .memd bundle detected"
fi

echo "update: steps"
echo "1. git pull --ff-only"
echo "2. cargo build -p memd-client --bin memd"
echo "3. scripts/install-memd.sh"
echo "4. memd doctor --summary"

if [[ "$DRY_RUN" == "1" ]]; then
  echo "update: dry-run ok; no files changed"
  exit 0
fi

git pull --ff-only
cargo build -p memd-client --bin memd
scripts/install-memd.sh
memd doctor --summary
echo "update: ok; .memd preserved"
