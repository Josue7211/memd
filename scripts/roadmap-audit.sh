#!/usr/bin/env bash
# Fail if any docs/backlog/**/*.md with status: open has a phase: that does not
# resolve to a live phase doc. Portable across macOS Bash 3 and newer shells.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOST_IO_GUARD="${HOST_IO_GUARD:-$SCRIPT_DIR/memd-host-io-guard.sh}"
if [ "${HOST_IO_GUARD_ENABLED:-1}" != "0" ] && [ "${HOST_IO_GUARD_ENABLED:-1}" != "false" ]; then
  "$HOST_IO_GUARD"
fi

REPO_ROOT="${REPO_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"

python3 - "$REPO_ROOT" <<'PY'
from pathlib import Path
import sys

repo = Path(sys.argv[1])
backlog_dir = repo / "docs/backlog"
phases_dir = repo / "docs/phases"

def frontmatter_value(path: Path, key: str) -> str:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != "---":
        return ""
    for line in lines[1:]:
        if line == "---":
            return ""
        prefix = f"{key}:"
        if line.startswith(prefix):
            return line[len(prefix):].strip()
    return ""

live_phases: set[str] = set()
for path in phases_dir.glob("**/phase-*.md"):
    phase = frontmatter_value(path, "phase")
    if phase:
        live_phases.add(phase)

failures: list[str] = []
total_open = 0
deferred = 0

for path in backlog_dir.glob("**/*.md"):
    if path.name in {"INDEX.md", "TEMPLATE.md"}:
        continue
    status = frontmatter_value(path, "status")
    phase = frontmatter_value(path, "phase")

    if status in {"closed", "resolved"}:
        continue
    if status == "in_progress":
        status = "open"
    if status == "deferred":
        deferred += 1
        continue

    total_open += 1

    if not phase or phase == "unassigned":
        failures.append(
            f"{path}: open item has phase='{phase or '<empty>'}' -- assign a live phase or mark status: deferred"
        )
        continue
    if phase not in live_phases:
        failures.append(
            f"{path}: phase '{phase}' does not resolve to a live phase doc under {phases_dir}/"
        )

if failures:
    print("roadmap-audit: FAIL", file=sys.stderr)
    for failure in failures:
        print(f"  - {failure}", file=sys.stderr)
    sys.exit(1)

print(f"roadmap-audit: ok -- {total_open} open items, all assigned to live phases ({deferred} deferred)")
PY
