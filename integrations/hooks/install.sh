#!/usr/bin/env bash
set -euo pipefail

PREFIX="${1:-${PREFIX:-$HOME/.local/bin}}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

mkdir -p "$PREFIX"
install -m 0755 "$SCRIPT_DIR/memd-context.sh" "$PREFIX/memd-context"
install -m 0755 "$SCRIPT_DIR/memd-spill.sh" "$PREFIX/memd-spill"

echo "Installed memd hooks to $PREFIX"
echo "Add $PREFIX to PATH if needed."
