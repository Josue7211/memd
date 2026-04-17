#!/usr/bin/env bash
set -euo pipefail

PREFIX="${1:-${PREFIX:-$HOME/.local/bin}}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MEMD_BIN="${MEMD_BIN:-memd}"

mkdir -p "$PREFIX"
install -m 0755 "$SCRIPT_DIR/memd-context.sh" "$PREFIX/memd-context"
install -m 0755 "$SCRIPT_DIR/memd-capture.sh" "$PREFIX/memd-capture"
install -m 0755 "$SCRIPT_DIR/memd-spill.sh" "$PREFIX/memd-spill"
install -m 0755 "$SCRIPT_DIR/memd-stop-save.sh" "$PREFIX/memd-stop-save"
install -m 0755 "$SCRIPT_DIR/memd-precompact-save.sh" "$PREFIX/memd-precompact-save"
install -m 0755 "$SCRIPT_DIR/memd-file-interaction.sh" "$PREFIX/memd-file-interaction"
install -m 0755 "$SCRIPT_DIR/memd-lifecycle-probe.sh" "$PREFIX/memd-lifecycle-probe"
install -m 0755 "$SCRIPT_DIR/memd-bootstrap.sh" "$PREFIX/memd-bootstrap"

cat > "$PREFIX/memd-hook-context" <<EOF
#!/usr/bin/env bash
exec "$PREFIX/memd-context" "\$@"
EOF
chmod +x "$PREFIX/memd-hook-context"

cat > "$PREFIX/memd-hook-spill" <<EOF
#!/usr/bin/env bash
exec "$MEMD_BIN" hook spill "\$@"
EOF
chmod +x "$PREFIX/memd-hook-spill"

cat > "$PREFIX/memd-hook-capture" <<EOF
#!/usr/bin/env bash
exec "$PREFIX/memd-capture" "\$@"
EOF
chmod +x "$PREFIX/memd-hook-capture"

cat > "$PREFIX/memd-hook-stop-save" <<EOF
#!/usr/bin/env bash
exec "$PREFIX/memd-stop-save" "\$@"
EOF
chmod +x "$PREFIX/memd-hook-stop-save"

cat > "$PREFIX/memd-hook-precompact-save" <<EOF
#!/usr/bin/env bash
exec "$PREFIX/memd-precompact-save" "\$@"
EOF
chmod +x "$PREFIX/memd-hook-precompact-save"

cat > "$PREFIX/memd-hook-bootstrap" <<EOF
#!/usr/bin/env bash
exec "$PREFIX/memd-bootstrap" "\$@"
EOF
chmod +x "$PREFIX/memd-hook-bootstrap"

cat > "$PREFIX/memd-hook-file-interaction" <<EOF
#!/usr/bin/env bash
exec "$PREFIX/memd-file-interaction" "\$@"
EOF
chmod +x "$PREFIX/memd-hook-file-interaction"

cat > "$PREFIX/memd-hook-lifecycle-probe" <<EOF
#!/usr/bin/env bash
exec "$PREFIX/memd-lifecycle-probe" "\$@"
EOF
chmod +x "$PREFIX/memd-hook-lifecycle-probe"

echo "Installed memd hooks to $PREFIX"
echo "Add $PREFIX to PATH if needed."
echo "Set MEMD_BIN if the memd CLI is not already on PATH."
