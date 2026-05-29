#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/verification/hive-runs/2026-05-26-internal-alpha"
DOC="$ROOT/docs/verification/feature-hive-hivemind-coordination-25.md"
REGISTRY="$ROOT/docs/verification/features.registry.json"

python3 - "$ROOT" "$ARTIFACT_DIR" "$DOC" "$REGISTRY" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
artifact_dir = Path(sys.argv[2])
doc = Path(sys.argv[3])
registry_path = Path(sys.argv[4])
errors = []

def fail(msg):
    errors.append(msg)

def require_file(path):
    p = root / path
    if not p.is_file():
        fail(f"missing required file: {path}")
        return ""
    return p.read_text(errors="replace")

def require_text(path, needles):
    text = require_file(path)
    for needle in needles:
        if needle not in text:
            fail(f"{path} missing required text: {needle}")
    return text

def rel(path):
    try:
        return str(path.relative_to(root))
    except Exception:
        return str(path)

def load_json(path):
    try:
        return json.loads(path.read_text())
    except Exception as exc:
        fail(f"invalid json {rel(path)}: {exc}")
        return None

require_text("docs/contracts/hive-live-map-guard.md", [
    "memd is the coordination/memory runtime",
    "separate projects, separate lifecycles",
    "must not launch ClawControl",
    "Observation does not give memd permission to restart it",
    "Every durable handoff must include a user-copyable next-agent prompt",
])
require_text("scripts/verify/hive-live-map-guard-contract.sh", [
    "require_absent \"live-state-sync-clawcontrol\" \"scripts/live-state-sync-memd.sh\"",
    "require_absent \"IMPORT_CLAWCONTROL_BUNDLE\" \"scripts/live-state-sync-memd.sh\"",
    "must not launch ClawControl",
    "reason=separate-existing-runtime",
])
require_text("scripts/verify/hive-production-proof.sh", [
    "MEMD_LOCALHOST_FALLBACK_POLICY='deny'",
    "MEMD_AUTHORITY_MODE",
    "MEMD_AUTHORITY_DEGRADED",
    "authority_policy",
    "blocked_capabilities",
])
require_text("scripts/dev-server-guard.sh", ["refusing to launch ClawControl from memd"])
require_text("scripts/live-state-sync-memd.sh", ["live-state-sync-memd"])
require_text("scripts/live-state-sync-clawcontrol.sh", ["This script must not launch ClawControl", "refusing by default"])
require_text("crates/memd-client/src/render/render_summary.rs", [
    "give next agent:",
    "do not launch ClawControl, Tauri, Vite, or app dev servers",
])

doc_text = require_file("docs/verification/feature-hive-hivemind-coordination-25.md")
for needle in [
    "Proof status: partial",
    "Dogfood status: ad_hoc",
    "External status: none",
    "No Cross-Agent Leakage Assumption",
    "Staleness Limit",
]:
    if needle not in doc_text:
        fail(f"proof doc missing honest gate text: {needle}")

registry = load_json(registry_path)
if registry:
    matches = [f for f in registry.get("features", []) if f.get("id") == "feature.hive_hivemind_coordination"]
    if len(matches) != 1:
        fail(f"expected exactly one hive registry entry, found {len(matches)}")
    else:
        f = matches[0]
        expected = {
            "current_status": "partial",
            "proof_status": "partial",
            "dogfood_status": "ad_hoc",
            "external_status": "none",
            "blocks_25_25": True,
        }
        for key, value in expected.items():
            if f.get(key) != value:
                fail(f"registry {key} expected {value!r}, got {f.get(key)!r}")
        if "docs/verification/feature-hive-hivemind-coordination-25.md" not in f.get("docs", []):
            fail("registry docs missing feature proof doc")
        if "docs/verification/hive-runs/2026-05-26-internal-alpha" not in f.get("proof_artifacts", []):
            fail("registry proof_artifacts missing archived hive run")
        if "bash scripts/verify/feature-hive-hivemind-coordination-proof.sh" not in f.get("proof_commands", []):
            fail("registry missing feature hive proof command")
        joined_allowed = " ".join(f.get("allowed_claims", []))
        joined_forbidden = " ".join(f.get("forbidden_claims", []))
        if "current static/local proof" not in joined_allowed:
            fail("registry allowed_claims do not describe current static/local proof")
        if "Do not claim production hive/hivemind reliability" not in joined_forbidden:
            fail("registry forbidden_claims missing production reliability prohibition")

if not artifact_dir.is_dir():
    fail(f"missing artifact directory: {rel(artifact_dir)}")
else:
    sums = load_json(artifact_dir / "SHA256SUMS.json") or {}
    for name, expected in sorted(sums.items()):
        path = artifact_dir / name
        if not path.is_file():
            if name.endswith(".log"):
                continue
            fail(f"checksum entry missing file: {name}")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            fail(f"checksum mismatch for {name}: expected {expected}, got {actual}")
    required_json = [
        "roster.json", "sessions.json", "inbox-worker-a.json", "inbox-worker-a-after-ack.json",
        "inbox-worker-b.json", "coord-inbox-worker-a.json", "coord-inbox-worker-a-owned.json",
        "coord-inbox-worker-b.json", "receipts.json", "dev-server-receipts.json",
        "claims-after-transfer.json", "claims-after-release.json", "follow-worker-a.json",
    ]
    artifacts = {name: load_json(artifact_dir / name) for name in required_json}
    if all(v is not None for v in artifacts.values()):
        roster = artifacts["roster.json"]
        bees = roster.get("bees", [])
        sessions = {bee.get("session"): bee for bee in bees}
        for session in ["queen", "worker-a", "worker-b"]:
            if session not in sessions:
                fail(f"roster missing session {session}")
        expected_caps = {"queen": "coordination", "worker-a": "memory", "worker-b": "review"}
        for session, cap in expected_caps.items():
            bee = sessions.get(session, {})
            if cap not in bee.get("capabilities", []):
                fail(f"roster session {session} missing capability {cap}")
            if bee.get("authority") != "participant":
                fail(f"roster session {session} authority is not participant")
            if bee.get("project") != "hive-proof" or not str(bee.get("namespace", "")).startswith("hive-proof-"):
                fail(f"roster session {session} not scoped to isolated hive-proof namespace")
        if len({bee.get("effective_agent") for bee in bees}) < 3:
            fail("roster does not show distinct effective agents")

        inbox_a = artifacts["inbox-worker-a.json"].get("messages", [])
        if not any(m.get("from_session") == "queen" and m.get("to_session") == "worker-a" and m.get("kind") == "note" for m in inbox_a):
            fail("worker-a inbox missing queen note")
        if artifacts["inbox-worker-a-after-ack.json"].get("messages") != []:
            fail("worker-a ack artifact still has messages")
        inbox_b = artifacts["inbox-worker-b.json"].get("messages", [])
        handoffs = [m for m in inbox_b if m.get("kind") == "handoff" and m.get("from_session") == "worker-a" and m.get("to_session") == "worker-b"]
        if not handoffs:
            fail("worker-b inbox missing worker-a handoff")
        elif "next_agent_prompt:" not in handoffs[0].get("content", ""):
            fail("handoff message missing next_agent_prompt")
        for label in ["inbox-worker-a.json", "inbox-worker-b.json"]:
            for m in artifacts[label].get("messages", []):
                if m.get("to_session") not in {"worker-a", "worker-b"}:
                    fail(f"{label} contains unexpected to_session {m.get('to_session')!r}")

        owned = artifacts["coord-inbox-worker-a-owned.json"].get("owned_tasks", [])
        if not any(t.get("coordination_mode") == "exclusive_write" and t.get("session") == "worker-a" for t in owned):
            fail("worker-a owned task artifact missing exclusive_write ownership")
        coord_a = artifacts["coord-inbox-worker-a.json"]
        coord_b = artifacts["coord-inbox-worker-b.json"]
        if not any(t.get("coordination_mode") == "help_only" for t in coord_a.get("help_tasks", [])):
            fail("worker-a coordination inbox missing help_only task")
        if not any(t.get("coordination_mode") == "shared_review" for t in coord_b.get("review_tasks", [])):
            fail("worker-b coordination inbox missing shared_review task")
        receipts = artifacts["receipts.json"].get("receipts", [])
        kinds = {r.get("kind") for r in receipts}
        for kind in ["task_assignment", "task_help_request", "task_review_request", "queen_handoff"]:
            if kind not in kinds:
                fail(f"receipts missing {kind}")
        dev_kinds = {r.get("kind") for r in artifacts["dev-server-receipts.json"].get("receipts", [])}
        for kind in ["dev_server_acquire", "dev_server_heartbeat", "dev_server_conflict"]:
            if kind not in dev_kinds:
                fail(f"dev server receipts missing {kind}")
        if artifacts["claims-after-release.json"].get("claims") != []:
            fail("claims-after-release not empty")
        if not artifacts["claims-after-transfer.json"].get("claims"):
            fail("claims-after-transfer missing transferred claim")
        if artifacts["follow-worker-a.json"].get("recommended_action") != "safe_to_continue":
            fail("follow-worker-a recommended_action is not safe_to_continue")

if errors:
    for error in errors:
        print(f"feature-hive-hivemind-coordination-proof: ERROR: {error}", file=sys.stderr)
    sys.exit(1)

print("feature-hive-hivemind-coordination-proof: ok")
print("validated: current authority docs/scripts, no cross-agent leakage assumptions, archived coordination artifacts")
print("honesty: local/static proof only; archived hive run remains ad hoc and externally unverified")
PY
