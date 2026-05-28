#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=0
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=1
fi

BIN="${MEMD_BIN:-$HOME/.local/bin/memd}"
echo "uninstall: binary target=$BIN"
echo "uninstall: memory is preserved by default (.memd is not deleted)"

if [[ "$DRY_RUN" == "1" ]]; then
  echo "uninstall: dry-run ok; would remove binary only"
  exit 0
fi

rm -f "$BIN"
echo "uninstall: removed binary if present; memory preserved"
