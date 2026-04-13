#!/usr/bin/env bash
set -euo pipefail

root_docs=(
  README.md
  START-HERE.md
  ROADMAP.md
  CONTRIBUTING.md
  SECURITY.md
  CODE_OF_CONDUCT.md
  CHANGELOG.md
  AGENTS.md
  CLAUDE.md
  THEORY.md
  DESIGN.md
)

fail() {
  echo "doc-lint: $*" >&2
  exit 1
}

check_contains() {
  local file=$1
  local pattern=$2
  local label=$3
  rg -q "$pattern" "$file" || fail "$file missing $label"
}

check_contains "README.md" "## Start Here" "Start Here section"
check_contains "START-HERE.md" "## Read In This Order" "read order"
check_contains "ROADMAP.md" "## Status Snapshot" "status snapshot"
check_contains "docs/WHERE-AM-I.md" "## Current Truth" "current truth section"
check_contains "docs/verification/milestones/MILESTONE-v1.md" "## Current State" "milestone current state"
check_contains "docs/core/INDEX.md" "## Read In This Order" "core index read order"
check_contains "docs/policy/INDEX.md" "## Recommended Order" "policy index order"
check_contains "docs/reference/INDEX.md" "## Recommended Order" "reference index order"
check_contains "docs/strategy/INDEX.md" "## Recommended Order" "strategy index order"
check_contains "docs/verification/INDEX.md" "## Read In This Order" "verification index read order"

for f in docs/core/*.md docs/policy/*.md docs/reference/*.md docs/strategy/*.md docs/verification/*.md; do
  check_contains "$f" "\\[\\[ROADMAP\\]\\]|Planning artifact|Reference doc|fresh-session recovery|Secondary/reference doc" "secondary-doc banner"
done

for f in docs/phases/*.md; do
  check_contains "$f" "PHASE_STATE" "phase state block"
done

for f in docs/backlog/*.md; do
  check_contains "$f" "BACKLOG_STATE" "backlog state block"
done

for f in *.md; do
  allowed=false
  for a in "${root_docs[@]}"; do
    if [[ "$f" == "$a" ]]; then
      allowed=true
      break
    fi
  done
  [[ "$allowed" == true ]] || fail "unexpected root markdown file: $f"
done

echo "doc-lint: ok"
