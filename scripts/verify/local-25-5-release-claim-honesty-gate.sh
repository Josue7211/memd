#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

fail() {
  echo "local-25-5-release-claim-honesty-gate: ERROR: $*" >&2
  exit 1
}

dynamic_status() {
  git status --porcelain -- docs/verification/release-0-1-0 docs/verification/release-1-0-0 .memd 2>/dev/null || true
}

before_dynamic="$(dynamic_status)"

bash scripts/verify/feature-registry-audit.sh
bash scripts/verify/feature-release-claim-honesty-gates-proof.sh
scripts/doc-lint.sh
git diff --check

after_dynamic="$(dynamic_status)"
if [[ "$before_dynamic" != "$after_dynamic" ]]; then
  printf before dynamic status:n%sn "$before_dynamic" >&2
  printf after dynamic status:n%sn "$after_dynamic" >&2
  fail "dynamic verification artifacts changed during local 25/5 gate"
fi
if [[ -n "$after_dynamic" ]]; then
  printf dirty dynamic status:n%sn "$after_dynamic" >&2
  fail "dynamic verification artifacts are dirty"
fi

echo "local-25-5-release-claim-honesty-gate: ok"
