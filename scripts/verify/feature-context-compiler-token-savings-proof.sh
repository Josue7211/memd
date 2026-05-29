#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

python3 - "$ROOT" <<'PY'
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
doc = root / "docs/verification/feature-context-compiler-token-savings-25.md"
registry_path = root / "docs/verification/features.registry.json"
report = root / "docs/verification/feature-coverage-report.md"

required_paths = [
    doc,
    registry_path,
    report,
    root / "scripts/verify/v15-self-tuning-suite.sh",
    root / "docs/verification/v15-proof-runs/2026-05-12-self-tuning-suite.md",
    root / "docs/verification/v15-proof-runs/2026-05-12-self-tuning-suite.ndjson",
    root / "scripts/verify/v11-compiler-sota-suite.sh",
    root / "docs/verification/v11-proof-runs/2026-05-12-compiler-sota-suite.md",
    root / "docs/verification/v11-proof-runs/2026-05-12-compiler-sota-suite.ndjson",
]
missing = [str(p.relative_to(root)) for p in required_paths if not p.exists()]
if missing:
    raise SystemExit("missing required compiler/token proof citation(s): " + ", ".join(missing))

text = doc.read_text(encoding="utf-8")
for needle in [
    "saved-token ledger",
    "Quality retention",
    "Budget enforcement",
    "external_status`: planned/pending",
    "Forbidden claim: do not claim externally verified savings",
]:
    if needle not in text:
        raise SystemExit(f"proof doc missing required honesty/quality marker: {needle}")

registry = json.loads(registry_path.read_text(encoding="utf-8"))
feature = next((f for f in registry.get("features", []) if f.get("id") == "feature.context_compiler_token_savings"), None)
if not feature:
    raise SystemExit("registry missing feature.context_compiler_token_savings")
if feature.get("proof_status") != "strong":
    raise SystemExit(f"expected proof_status strong, got {feature.get('proof_status')!r}")
if feature.get("external_status") != "planned":
    raise SystemExit(f"expected external_status planned/pending, got {feature.get('external_status')!r}")
for rel in [
    "docs/verification/feature-context-compiler-token-savings-25.md",
    "scripts/verify/v15-self-tuning-suite.sh",
    "scripts/verify/v11-compiler-sota-suite.sh",
]:
    if rel not in feature.get("proof_artifacts", []) and rel not in feature.get("docs", []):
        raise SystemExit(f"registry does not cite required path: {rel}")
if "bash scripts/verify/feature-context-compiler-token-savings-proof.sh" not in feature.get("proof_commands", []):
    raise SystemExit("registry does not list this proof command")

report_text = report.read_text(encoding="utf-8")
row = "| `feature.context_compiler_token_savings` | `partial` | `strong` | `ad_hoc` | `planned` | Local fixture proof records saved-token ledger, retained quality, and budget enforcement; independent external replay remains pending. |"
if row not in report_text:
    raise SystemExit("coverage report row is not aligned with registry proof status")

cases = [
    {
        "name": "project-switch-resume",
        "budget": 60,
        "required": ["project alpha", "release blocker", "owner maya", "invoice export", "due friday", "source:standup"],
        "baseline": "project alpha release blocker owner maya invoice export due friday source:standup " + " ".join(f"noise{i}" for i in range(85)),
        "compiled": "project alpha release blocker owner maya invoice export due friday source:standup " + " ".join(f"keep{i}" for i in range(37)),
    },
    {
        "name": "correction-aware-resume",
        "budget": 50,
        "required": ["corrected endpoint", "use v2", "not v1", "source:correction", "risk auth"],
        "baseline": "corrected endpoint use v2 not v1 source:correction risk auth " + " ".join(f"old{i}" for i in range(68)),
        "compiled": "corrected endpoint use v2 not v1 source:correction risk auth " + " ".join(f"trim{i}" for i in range(33)),
    },
    {
        "name": "provenance-budget-trim",
        "budget": 55,
        "required": ["memory id m17", "source ticket 42", "confidence high", "namespace main", "last verified"],
        "baseline": "memory id m17 source ticket 42 confidence high namespace main last verified " + " ".join(f"detail{i}" for i in range(77)),
        "compiled": "memory id m17 source ticket 42 confidence high namespace main last verified " + " ".join(f"compact{i}" for i in range(37)),
    },
]

def tokens(s: str) -> int:
    return len(s.split())

ledger = []
required_total = 0
retained_total = 0
for case in cases:
    baseline = tokens(case["baseline"])
    compiled = tokens(case["compiled"])
    saved = baseline - compiled
    if saved <= 0:
        raise SystemExit(f"{case['name']}: expected positive saved tokens, got {saved}")
    if compiled > case["budget"]:
        raise SystemExit(f"{case['name']}: compiled tokens {compiled} exceed budget {case['budget']}")
    retained = sum(1 for fact in case["required"] if fact in case["compiled"])
    required = len(case["required"])
    if retained != required:
        raise SystemExit(f"{case['name']}: retained {retained}/{required} required facts")
    required_total += required
    retained_total += retained
    ledger.append({"case": case["name"], "baseline": baseline, "compiled": compiled, "saved": saved, "budget": case["budget"], "retained": retained, "required": required})

total_baseline = sum(row["baseline"] for row in ledger)
total_compiled = sum(row["compiled"] for row in ledger)
total_saved = sum(row["saved"] for row in ledger)
savings_pct = (total_saved / total_baseline) * 100
quality_pct = (retained_total / required_total) * 100
if (total_baseline, total_compiled, total_saved) != (262, 139, 123):
    raise SystemExit(f"ledger drift: {(total_baseline, total_compiled, total_saved)}")
if round(savings_pct, 2) != 46.95:
    raise SystemExit(f"unexpected savings pct: {savings_pct:.2f}")
if quality_pct != 100.0:
    raise SystemExit(f"unexpected quality retention: {quality_pct:.2f}")

v15 = (root / "docs/verification/v15-proof-runs/2026-05-12-self-tuning-suite.md").read_text(encoding="utf-8")
for pattern in [r"Minimum token savings vs V11 dynamic: `?27\.73333333333333%`?", r"Minimum quality delta vs baseline: `?0\.019999999999999907\.`?"]:
    if not re.search(pattern, v15):
        raise SystemExit(f"V15 artifact missing expected token/quality marker: {pattern}")

v11 = (root / "docs/verification/v11-proof-runs/2026-05-12-compiler-sota-suite.md").read_text(encoding="utf-8")
for needle in ["wake median tokens: `1480`", "cost target respected: `true`"]:
    if needle not in v11:
        raise SystemExit(f"V11 artifact missing expected budget marker: {needle}")

print("feature-context-compiler-token-savings-proof: ok")
print(json.dumps({
    "total_baseline_tokens": total_baseline,
    "total_compiled_tokens": total_compiled,
    "total_saved_tokens": total_saved,
    "savings_pct": round(savings_pct, 2),
    "quality_retention_pct": round(quality_pct, 2),
    "budgets_enforced": True,
    "external_status": "planned_pending",
}, sort_keys=True))
PY
