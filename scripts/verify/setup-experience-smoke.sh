#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

OUT_DIR="docs/verification/setup-runs"
mkdir -p "$OUT_DIR"
STAMP="$(date +%F-%H%M%S)"
REPORT="$OUT_DIR/${STAMP}-setup-experience-smoke.md"

{
  echo "# Setup Experience Smoke"
  echo
  echo "- date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "- host: $(hostname 2>/dev/null || echo unknown)"
  echo "- os: $(uname -a)"
  echo
  echo "## Commands"
  echo
  echo '```text'
  echo "command -v memd"
  command -v memd || true
  echo
  echo "memd setup --summary --force"
  memd setup --summary --force
  echo
  echo "memd doctor --summary"
  memd doctor --summary
  echo
  echo "memd status --output .memd --summary"
  memd status --output .memd --summary
  echo
  echo "memd resume --output .memd --intent current_task"
  memd resume --output .memd --intent current_task >/tmp/memd-setup-resume.out
  sed -n '1,40p' /tmp/memd-setup-resume.out
  echo '```'
  echo
  echo "## Result"
  echo
  echo "setup-experience-smoke=pass"
} | tee "$REPORT"

echo "setup smoke report: $REPORT"
