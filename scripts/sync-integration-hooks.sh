#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST_IO_GUARD="${HOST_IO_GUARD:-$ROOT/scripts/memd-host-io-guard.sh}"
if [ "${HOST_IO_GUARD_ENABLED:-1}" != "0" ] && [ "${HOST_IO_GUARD_ENABLED:-1}" != "false" ]; then
  "$HOST_IO_GUARD"
fi

SRC=".memd/hooks"
DST="integrations/hooks"

[ -d "$SRC" ] || { echo "no $SRC"; exit 1; }
[ -d "$DST" ] || mkdir -p "$DST"

# Per docs/handoff/2026-04-17-a3-part2-hooks-audit.md:
# .memd/hooks is canonical; integrations/hooks is derived.
# Most files sync from .memd, BUT 3 exceptions come from integrations (already corrected there).
#
# Exceptions (use integrations version as-is):
#   - memd-bootstrap.sh: global-only logic, correct staleness check
#   - memd-capture.sh: agent-neutral ending (no stale CODEX fallback)
#   - memd-context.sh: agent-neutral ending (no stale CODEX fallback)
#
# README.md: Syncs from .memd, but with a different notice prepended.
# - .memd/hooks/README.md: "This is the canonical source"
# - integrations/hooks/README.md: "Generated file" (auto-synced copy)
#
# No path-based rewrites needed.

# Step 1: Backup the shell script exceptions (they're already correct)
tmp_exceptions="$(mktemp -d)"
for f in memd-bootstrap.sh memd-capture.sh memd-context.sh; do
  if [ -f "$DST/$f" ]; then
    cp -p "$DST/$f" "$tmp_exceptions/$f"
  fi
done

# Step 2: Copy all shell scripts and PowerShell scripts from .memd
for f in "$SRC"/*.sh "$SRC"/*.ps1; do
  [ -f "$f" ] || continue
  name="$(basename "$f")"
  cp -p "$f" "$DST/$name"
done

# Step 2b: README.md special handling — sync from .memd but with different notice
if [ -f "$SRC/README.md" ]; then
  # Strip the "canonical source" notice from .memd version
  tail -n +8 "$SRC/README.md" > "$DST/README.md"
  # Prepend the "generated file" notice
  cat > "$DST/README.md.tmp" <<'EOF'
> **Generated file.** These scripts are synced from `.memd/hooks/` by `scripts/sync-integration-hooks.sh`.
> Edit the source at `.memd/hooks/` and re-run the script. Do not edit files in this directory directly.

EOF
  cat "$DST/README.md" >> "$DST/README.md.tmp"
  mv "$DST/README.md.tmp" "$DST/README.md"
fi

# Step 3: Restore the shell script exceptions (they're cleaner/newer), preserving their modes
for f in memd-bootstrap.sh memd-capture.sh memd-context.sh; do
  if [ -f "$tmp_exceptions/$f" ]; then
    cp -p "$tmp_exceptions/$f" "$DST/$f"
  fi
done

# Step 4: Ensure .sh files are executable
chmod +x "$DST"/*.sh 2>/dev/null || true

# Step 5: Ensure non-script files are NOT executable
chmod -x "$DST"/*.ps1 "$DST"/README.md 2>/dev/null || true

# Cleanup
rm -rf "$tmp_exceptions"

echo "synced: $DST"
