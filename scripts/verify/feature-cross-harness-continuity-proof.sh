#!/usr/bin/env bash
# Local/static proof for feature.cross_harness_continuity.
# It validates documented/generated continuity surfaces across the harness packs
# that exist in this checkout, checks handoff/resume/wake parity, and rejects
# fake external cross-session claims. It does not claim independent external
# replay or production continuity.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

python3 - "$ROOT" <<'PYPROOF'
import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
errors = []
notes = []

harnesses = {
    "codex": {"doc": "integrations/codex/README.md", "rust": "crates/memd-client/src/harness/codex.rs", "agent": ".memd/agents/codex.sh"},
    "hermes": {"doc": "integrations/hermes/README.md", "rust": "crates/memd-client/src/harness/hermes.rs", "agent": ".memd/agents/hermes.sh"},
    "openclaw": {"doc": "integrations/openclaw/README.md", "rust": "crates/memd-client/src/harness/openclaw.rs", "agent": ".memd/agents/openclaw.sh"},
    "claude-code": {"doc": "integrations/claude-code/README.md", "rust": "crates/memd-client/src/harness/claude_code.rs", "agent": ".memd/agents/claude-code.sh"},
}
required_bundle_surfaces = [".memd/wake.md", ".memd/mem.md", ".memd/events.md"]
continuity_words = ["wake", "resume"]
write_words = ["checkpoint", "handoff", "spill", "hook capture", "teach", "remember"]

available = []
for name, spec in harnesses.items():
    doc = root / spec["doc"]
    rust = root / spec["rust"]
    if not doc.exists() and not rust.exists():
        notes.append(f"{name}: no local harness artifact found; skipped")
        continue
    available.append(name)
    text = (doc.read_text(encoding="utf-8") if doc.exists() else "") + "\n" + (rust.read_text(encoding="utf-8") if rust.exists() else "")
    lower = text.lower()
    if not doc.exists():
        errors.append(f"{name}: missing integration doc {spec['doc']}")
    if not rust.exists():
        errors.append(f"{name}: missing harness module {spec['rust']}")
    for surface in required_bundle_surfaces:
        if surface.lower() not in lower and surface.replace(".memd/", "") not in lower:
            errors.append(f"{name}: missing shared bundle surface reference {surface}")
    for word in continuity_words:
        if word not in lower:
            errors.append(f"{name}: missing {word} continuity verb")
    if not any(word in lower for word in write_words):
        errors.append(f"{name}: missing any write/handoff continuity verb from {write_words}")
    if spec["agent"].lower() not in lower and name != "claude-code":
        errors.append(f"{name}: missing generated agent entrypoint {spec['agent']}")
    if name == "claude-code" and ".memd/agents/claude-code.sh" not in lower and ".memd/agents/claude_imports.md" not in lower:
        errors.append("claude-code: missing Claude-style generated entrypoint/import surface")

if len(available) < 3:
    errors.append(f"expected at least three local harness surfaces, found {available}")

for path in ["crates/memd-client/src/harness/index.rs", "crates/memd-client/src/harness/preset.rs", "crates/memd-client/src/harness/mod.rs"]:
    p = root / path
    if not p.exists():
        errors.append(f"missing shared harness surface {path}")
        continue
    text = p.read_text(encoding="utf-8").lower()
    for name in available:
        needle = "claude" if name == "claude-code" else name
        if needle not in text:
            errors.append(f"{path}: missing {name} shared harness reference")

for path in ["scripts/handoff-latest.sh", "scripts/memd-continuity-status.sh", "scripts/verify/25-5-harness-process-replay.sh"]:
    if not (root / path).exists():
        errors.append(f"missing continuity parity artifact {path}")
combined_docs = "\n".join((root / harnesses[name]["doc"]).read_text(encoding="utf-8") for name in available if (root / harnesses[name]["doc"]).exists()).lower()
for term in ["handoff", "resume", "wake"]:
    if term not in combined_docs:
        errors.append(f"combined harness docs missing parity term {term}")

replay_dir = root / "docs/verification/25-5-memory-os-runs"
replay_files = sorted(replay_dir.glob("*-harness-process-replay.json")) if replay_dir.exists() else []
if replay_files:
    latest = replay_files[-1]
    try:
        data = json.loads(latest.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"latest replay artifact is invalid JSON: {latest}: {exc}")
    else:
        if data.get("status") != "pass":
            errors.append(f"latest replay artifact does not pass: {latest}")
        if data.get("codex_private_visible") is not False:
            errors.append(f"latest replay artifact does not prove private isolation false visibility: {latest}")
        sections = set(data.get("ollama_packet_sections") or [])
        for section in ["System Guard", "Pinned Corrections", "Active Truth", "Evidence", "Procedures", "Open Conflicts", "Source IDs"]:
            if section not in sections:
                errors.append(f"latest replay artifact missing packet section {section}: {latest}")
        notes.append(f"validated existing local process replay artifact {latest.relative_to(root)}")
else:
    notes.append("no dated process replay JSON artifact found; static continuity surface proof only")

registry_path = root / "docs/verification/features.registry.json"
registry = json.loads(registry_path.read_text(encoding="utf-8"))
feature = next((f for f in registry.get("features", []) if f.get("id") == "feature.cross_harness_continuity"), None)
if not feature:
    errors.append("registry missing feature.cross_harness_continuity")
else:
    if feature.get("external_status") != "none":
        errors.append("cross_harness_continuity external_status must remain none without independent external evidence")
    if feature.get("dogfood_status") != "ad_hoc":
        errors.append("cross_harness_continuity dogfood_status must remain ad_hoc for local proof")
    if feature.get("proof_status") not in {"partial", "smoke"}:
        errors.append("cross_harness_continuity proof_status should be partial/smoke for local proof")
    if "docs/verification/feature-cross-harness-continuity-25.md" not in feature.get("proof_artifacts", []):
        errors.append("registry row missing proof doc artifact")
    if "bash scripts/verify/feature-cross-harness-continuity-proof.sh" not in feature.get("proof_commands", []):
        errors.append("registry row missing proof command")
    allowed = "\n".join(feature.get("allowed_claims", [])).lower()
    if "external_verified" in allowed or "externally verified" in allowed or "independent external" in allowed:
        errors.append("registry allowed claims contain fake external verification wording")

for rel in ["docs/verification/feature-cross-harness-continuity-25.md", "docs/verification/feature-coverage-report.md", "docs/verification/FEATURES.md"]:
    p = root / rel
    if not p.exists():
        errors.append(f"missing doc {rel}")
        continue
    text = p.read_text(encoding="utf-8").lower()
    window = text
    if rel.endswith("feature-coverage-report.md") or rel.endswith("FEATURES.md"):
        m = re.search(r"feature\.cross_harness_continuity.*", text)
        window = m.group(0) if m else ""
    if "external_verified" in window or "externally verified" in window:
        errors.append(f"{rel}: contains external verification wording for local-only proof")

if errors:
    for e in errors:
        print(f"feature-cross-harness-continuity-proof: ERROR: {e}", file=sys.stderr)
    sys.exit(1)

print("feature-cross-harness-continuity-proof: ok")
print("available_harnesses=" + ",".join(available))
for note in notes:
    print("note: " + note)
PYPROOF
