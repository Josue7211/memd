#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

fail() {
  echo "feature-release-claim-honesty-gates-proof: ERROR: $*" >&2
  exit 1
}

require_file() {
  [[ -f "$1" ]] || fail "missing file: $1"
}

require_executable() {
  [[ -x "$1" ]] || fail "not executable: $1"
}

require_file docs/verification/features.registry.json
require_file docs/verification/feature-coverage-report.md
require_file docs/verification/FEATURES.md
require_file docs/verification/feature-release-claim-honesty-gates-25.md
require_file docs/policy/archive/release-process.md
require_file scripts/verify/feature-registry-audit.sh
require_file scripts/verify/local-25-5-release-claim-honesty-gate.sh
require_executable scripts/verify/feature-registry-audit.sh
require_executable scripts/verify/feature-release-claim-honesty-gates-proof.sh
require_executable scripts/verify/local-25-5-release-claim-honesty-gate.sh

before_dynamic="$(git status --porcelain -- docs/verification/release-0-1-0 docs/verification/release-1-0-0 .memd 2>/dev/null || true)"

bash scripts/verify/feature-registry-audit.sh

python3 - <<'PY'
from __future__ import annotations

import json
import re
import shlex
from pathlib import Path

root = Path('.')
registry_path = root / 'docs/verification/features.registry.json'
report_path = root / 'docs/verification/feature-coverage-report.md'
features_md_path = root / 'docs/verification/FEATURES.md'
proof_doc_path = root / 'docs/verification/feature-release-claim-honesty-gates-25.md'
release_process_path = root / 'docs/policy/archive/release-process.md'

registry = json.loads(registry_path.read_text(encoding='utf-8'))
feature = next((f for f in registry.get('features', []) if f.get('id') == 'feature.release_claim_honesty_gates'), None)
if feature is None:
    raise SystemExit('missing feature.release_claim_honesty_gates registry row')

errors: list[str] = []
if feature.get('current_status') != 'partial':
    errors.append('current_status must remain partial for local-only honesty gate')
if feature.get('proof_status') != 'strong':
    errors.append('proof_status must be strong for verified local 25/5 proof')
if feature.get('dogfood_status') not in {'ad_hoc', 'none', 'planned'}:
    errors.append('dogfood_status must not imply sustained release-flow integration')
if feature.get('external_status') != 'none':
    errors.append('external_status must remain none until external release-flow evidence exists')
if feature.get('blocks_25_25') is not True:
    errors.append('blocks_25_25 must remain true')

allowed = ' '.join(feature.get('allowed_claims') or []).lower()
for needle in ['strong local 25/5', 'registry audit', '25/25', 'doc lint', 'git diff hygiene', 'dynamic-artifact cleanliness']:
    if needle not in allowed:
        errors.append(f'allowed_claims must mention {needle!r}')
forbidden = ' '.join(feature.get('forbidden_claims') or []).lower()
for needle in ['do not claim', '25/25', 'local 25/5', 'release-flow', 'external']:
    if needle not in forbidden:
        errors.append(f'forbidden_claims must mention {needle!r}')

expected_docs = {
    'docs/verification/FEATURES.md',
    'docs/verification/feature-coverage-report.md',
    'docs/verification/feature-release-claim-honesty-gates-25.md',
    'docs/policy/archive/release-process.md',
    'ROADMAP.md',
}
missing_docs = expected_docs - set(feature.get('docs') or [])
if missing_docs:
    errors.append(f'registry docs missing expected release honesty docs: {sorted(missing_docs)}')

expected_commands = {
    'bash scripts/verify/feature-registry-audit.sh',
    'bash scripts/verify/feature-release-claim-honesty-gates-proof.sh',
    'bash scripts/verify/local-25-5-release-claim-honesty-gate.sh',
    'scripts/doc-lint.sh',
    'git diff --check',
}
missing_commands = expected_commands - set(feature.get('proof_commands') or [])
if missing_commands:
    errors.append(f'registry proof_commands missing: {sorted(missing_commands)}')

expected_artifacts = {
    'docs/verification/feature-coverage-report.md',
    'docs/verification/feature-release-claim-honesty-gates-25.md',
}
missing_artifacts = expected_artifacts - set(feature.get('proof_artifacts') or [])
if missing_artifacts:
    errors.append(f'registry proof_artifacts missing: {sorted(missing_artifacts)}')

for command in feature.get('proof_commands') or []:
    parts = shlex.split(command)
    if not parts:
        errors.append('empty proof command')
        continue
    if parts[0] in {'bash', 'sh'} and len(parts) >= 2:
        script = root / parts[1]
        if not script.is_file():
            errors.append(f'proof command script missing: {parts[1]}')
        elif not script.stat().st_mode & 0o111:
            errors.append(f'proof command script not executable: {parts[1]}')
    elif parts[:3] == ['git', 'diff', '--check'] or parts[:2] == ['git', 'diff']:
        continue
    elif parts[0].startswith('scripts/'):
        script = root / parts[0]
        if not script.is_file():
            errors.append(f'proof command script missing: {parts[0]}')
        elif not script.stat().st_mode & 0o111:
            errors.append(f'proof command script not executable: {parts[0]}')
    else:
        if parts[0] not in {'cargo'}:
            errors.append(f'unrecognized proof command shape: {command}')

release_text = release_process_path.read_text(encoding='utf-8').lower()
for needle in [
    'feature-registry-audit.sh',
    'feature-release-claim-honesty-gates-proof.sh',
    'local-25-5-release-claim-honesty-gate.sh',
    'local `25/5`',
    '25/25',
    'unsupported',
]:
    if needle not in release_text:
        errors.append(f'release process missing honesty gate text: {needle}')

report_text = report_path.read_text(encoding='utf-8')
if '`feature.release_claim_honesty_gates` | `partial` | `strong` |' not in report_text:
    errors.append('coverage report row must show strong local 25/5 proof for release honesty gates')
for needle in ['strong local 25/5 honesty proof', 'dynamic-artifact cleanliness', 'unsupported 25/25']:
    if needle not in report_text.lower():
        errors.append(f'coverage report blocker must describe {needle!r}')

features_md = features_md_path.read_text(encoding='utf-8')
if '`feature.release_claim_honesty_gates` | release/claim honesty gates | `partial` | `strong` |' not in features_md:
    errors.append('FEATURES.md row must mirror strong local proof status')

truth_files = [
    Path('README.md'),
    Path('START-HERE.md'),
    Path('ROADMAP.md'),
    Path('CHANGELOG.md'),
    features_md_path,
    report_path,
    proof_doc_path,
    release_process_path,
]
negative_markers = re.compile(r"\b(do not|don't|not|no|block|blocks|blocked|unsupported|forbidden|pending|before any|without|until|does not|honest|backed|supported)\b", re.I)
for path in truth_files:
    if not path.exists():
        continue
    for lineno, line in enumerate(path.read_text(encoding='utf-8', errors='replace').splitlines(), 1):
        if '25/25' in line and not negative_markers.search(line):
            errors.append(f'possible unsupported 25/25 overclaim in {path}:{lineno}: {line.strip()}')

if errors:
    for err in errors:
        print(f'feature-release-claim-honesty-gates-proof: ERROR: {err}')
    raise SystemExit(1)
PY

after_dynamic="$(git status --porcelain -- docs/verification/release-0-1-0 docs/verification/release-1-0-0 .memd 2>/dev/null || true)"
if [[ "$before_dynamic" != "$after_dynamic" ]]; then
  printf 'before dynamic status:\n%s\n' "$before_dynamic" >&2
  printf 'after dynamic status:\n%s\n' "$after_dynamic" >&2
  fail "dynamic verification artifacts changed during proof"
fi
if [[ -n "$after_dynamic" ]]; then
  printf 'dirty dynamic status:\n%s\n' "$after_dynamic" >&2
  fail "dynamic verification artifacts are dirty"
fi

echo "feature-release-claim-honesty-gates-proof: ok"
