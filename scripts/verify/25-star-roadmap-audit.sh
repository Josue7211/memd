#!/usr/bin/env bash
# Audit the V21-V35 25-star roadmap contract without activating V21+.

set -euo pipefail

REPO_ROOT="${REPO_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$REPO_ROOT"

python3 - <<'PY'
from pathlib import Path
import re
import sys

root = Path.cwd()
strategy = root / "docs/strategy/25-star-master-roadmap.md"
contract = root / "docs/verification/25-star-CONTRACT.md"
ledger = root / "docs/verification/25-star-phase-ledger.md"
roadmap = root / "ROADMAP.md"
verification_index = root / "docs/verification/INDEX.md"

failures: list[str] = []

def require(condition: bool, message: str) -> None:
    if not condition:
        failures.append(message)

def read(path: Path) -> str:
    if not path.exists():
        failures.append(f"missing required file: {path}")
        return ""
    return path.read_text(encoding="utf-8")

strategy_text = read(strategy)
contract_text = read(contract)
ledger_text = read(ledger)
roadmap_text = read(roadmap)
index_text = read(verification_index)
all_text = "\n".join([contract_text, strategy_text, ledger_text])
all_text_flat = re.sub(r"\s+", " ", all_text)

versions = list(range(21, 36))
phase_letters = list("ABCDEFG")

# Docs lint: core docs and index pointers exist.
require("25-Star memd Master Roadmap" in strategy_text or "25-Star Master Roadmap" in strategy_text, "strategy doc missing title")
require("25-star-CONTRACT" in roadmap_text, "ROADMAP.md missing 25-star contract pointer")
require("25-star-master-roadmap" in roadmap_text, "ROADMAP.md missing 25-star roadmap pointer")
require("25-star-CONTRACT" in index_text, "verification index missing 25-star contract pointer")
require("25-star-phase-ledger" in index_text, "verification index missing 25-star phase ledger pointer")

# Contract audit: every version must have metric, artifact path, kill, recovery.
contract_rows: dict[int, list[str]] = {}
for line in contract_text.splitlines():
    match = re.match(r"\|\s*V(2[1-9]|3[0-5])\b([^|]*)\|(.+)\|", line)
    if match:
        version = int(match.group(1))
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        contract_rows[version] = cells

for version in versions:
    cells = contract_rows.get(version)
    require(cells is not None, f"contract missing V{version} row")
    if not cells:
        continue
    require(len(cells) >= 5, f"contract V{version} row missing required cells")
    metric = cells[1] if len(cells) > 1 else ""
    artifact = cells[2] if len(cells) > 2 else ""
    kill = cells[3] if len(cells) > 3 else ""
    recovery = cells[4] if len(cells) > 4 else ""
    require(len(metric) > 12, f"V{version} metric too thin")
    require(f"docs/verification/v{version}-proof-runs/" in artifact, f"V{version} artifact path missing proof-run directory")
    require(len(kill) > 8, f"V{version} kill criterion missing")
    if version == 35:
        require("reset" in recovery.lower() or "v35.5" in recovery.lower(), "V35 recovery/reset rule missing")
    else:
        require(f"V{version}.5" in recovery, f"V{version} recovery version missing")

# Status audit: V21+ must be deferred/planned only before 1.0.0.
require("V21-V35 are strategy-seeded but **not active** until honest `1.0.0` close" in roadmap_text, "ROADMAP missing explicit V21+ inactive rule")
require("This document does not activate V21+" in strategy_text, "strategy doc missing no-activation rule")
require("V21+ may not be marked `active` while V20 real gates are open" in contract_text, "contract missing V21+ active ban")
bad_status_patterns = [
    r"v2[1-9]_status:\s*active",
    r"v3[0-5]_status:\s*active",
    r"V2[1-9][^\n|]*\|\s*active\s*\|",
    r"V3[0-5][^\n|]*\|\s*active\s*\|",
]
for pattern in bad_status_patterns:
    require(re.search(pattern, roadmap_text + "\n" + contract_text + "\n" + ledger_text, re.I) is None, f"forbidden active V21+ status pattern: {pattern}")

# Evidence audit: real gates cannot close from synthetic proof alone.
required_evidence_phrases = [
    "Synthetic proof may unblock engineering, but it cannot close any gate",
    "No version advances on synthetic proof alone",
    "Synthetic proof can exercise harnesses, but cannot close",
]
for phrase in required_evidence_phrases:
    require(phrase in all_text_flat, f"missing evidence rule phrase: {phrase}")

# Atomicity audit: every version has A-G phases and rollback/recovery notes.
for version in versions:
    heading = re.search(rf"^### V{version}\b.*$", ledger_text, re.M)
    require(heading is not None, f"ledger missing V{version} heading")
    if heading is None:
        continue
    next_heading = re.search(r"^### V(2[1-9]|3[0-5])\b.*$", ledger_text[heading.end():], re.M)
    section = ledger_text[heading.end(): heading.end() + next_heading.start()] if next_heading else ledger_text[heading.end():]
    require(f"docs/verification/v{version}-proof-runs/" in section, f"ledger V{version} missing artifact root")
    for phase in phase_letters:
        require(re.search(rf"^\|\s*{phase}\s*\|", section, re.M) is not None, f"ledger V{version} missing phase {phase}")
    require("rollback" in section.lower() or "recovery" in section.lower(), f"ledger V{version} missing rollback/recovery note")
    require(section.lower().count("revert") + section.lower().count("opens v") + section.lower().count("blocks close") >= 3, f"ledger V{version} rollback notes too thin")

if failures:
    print("25-star-roadmap-audit: FAIL", file=sys.stderr)
    for failure in failures:
        print(f"  - {failure}", file=sys.stderr)
    sys.exit(1)

print("25-star-roadmap-audit: ok")
print(f"  versions audited: {len(versions)}")
print("  checks: docs lint, contract audit, status audit, evidence audit, atomicity audit")
PY
