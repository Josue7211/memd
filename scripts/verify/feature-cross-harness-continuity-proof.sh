#!/usr/bin/env bash
# Strong local 25/5 proof for feature.cross_harness_continuity.
# This validates local continuity across Codex/Claude/OpenCode/OpenClaw/Hermes
# config surfaces, generated/native recovery and handoff bundles, cross-process
# memory object consistency, artifact cleanliness, and claim boundaries. It does
# not claim independent external replay, sustained dogfood, or production-grade
# seamless continuity.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

python3 - "$ROOT" <<'PYPROOF'
from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

root = Path(sys.argv[1])
errors: list[str] = []
notes: list[str] = []

TARGET_HARNESSES = {
    "codex": {"display": "Codex", "doc": "integrations/codex/README.md", "rust": "crates/memd-client/src/harness/codex.rs", "agent": ".memd/agents/codex.sh", "preset_id": "codex"},
    "claude-code": {"display": "Claude Code", "doc": "integrations/claude-code/README.md", "rust": "crates/memd-client/src/harness/claude_code.rs", "agent": ".memd/agents/claude-code.sh", "preset_id": "claude-code"},
    "opencode": {"display": "OpenCode", "doc": "integrations/opencode/README.md", "rust": "crates/memd-client/src/harness/opencode.rs", "agent": ".memd/agents/opencode.sh", "preset_id": "opencode"},
    "openclaw": {"display": "OpenClaw", "doc": "integrations/openclaw/README.md", "rust": "crates/memd-client/src/harness/openclaw.rs", "agent": ".memd/agents/openclaw.sh", "preset_id": "openclaw"},
    "hermes": {"display": "Hermes", "doc": "integrations/hermes/README.md", "rust": "crates/memd-client/src/harness/hermes.rs", "agent": ".memd/agents/hermes.sh", "preset_id": "hermes"},
}

REQUIRED_BUNDLE_SURFACES = [".memd/wake.md", ".memd/mem.md", ".memd/events.md"]
CONTINUITY_TERMS = ["wake", "resume"]
WRITE_TERMS = ["checkpoint", "handoff", "spill", "hook capture", "teach", "remember", "capture"]


def fail(message: str) -> None:
    errors.append(message)


def read_required(rel: str) -> str:
    path = root / rel
    if not path.is_file():
        fail(f"missing required file: {rel}")
        return ""
    if path.is_symlink():
        fail(f"required proof surface must not be a symlink: {rel}")
    return path.read_text(encoding="utf-8", errors="replace")


def git_porcelain(paths: list[str] | None = None) -> str | None:
    cmd = ["git", "status", "--porcelain"]
    if paths:
        cmd.extend(["--", *paths])
    try:
        return subprocess.check_output(cmd, cwd=root, text=True, stderr=subprocess.DEVNULL)
    except Exception:
        return None


def validate_harness_config_surfaces() -> None:
    preset = read_required("crates/memd-client/src/harness/preset.rs")
    index = read_required("crates/memd-client/src/harness/index.rs")
    mod = read_required("crates/memd-client/src/harness/mod.rs")
    shared = read_required("crates/memd-client/src/harness/shared.rs")
    if "SHARED_VISIBLE_SURFACES" not in preset or "wake.md" not in preset or "mem.md" not in preset or "events.md" not in preset:
        fail("shared preset must define wake/mem/events visible surfaces")
    if "strict_context_command" not in shared or "include-capabilities" not in shared or "include-access" not in shared:
        fail("shared harness config must include strict context capability/access route command")

    for name, spec in TARGET_HARNESSES.items():
        doc_text = read_required(spec["doc"])
        rust_text = read_required(spec["rust"])
        combined = f"{doc_text}\n{rust_text}".lower()
        for surface in REQUIRED_BUNDLE_SURFACES:
            if surface.lower() not in combined and surface.replace(".memd/", "") not in combined:
                fail(f"{name}: missing shared bundle surface reference {surface}")
        for term in CONTINUITY_TERMS:
            if term not in combined:
                fail(f"{name}: missing continuity term {term}")
        if not any(term in combined for term in WRITE_TERMS):
            fail(f"{name}: missing write/handoff continuity term from {WRITE_TERMS}")
        if spec["agent"].lower() not in combined and name != "claude-code":
            fail(f"{name}: missing generated agent entrypoint {spec['agent']}")
        if name == "claude-code" and ".memd/agents/claude-code.sh" not in combined and ".memd/agents/claude_imports.md" not in combined:
            fail("claude-code: missing Claude native/generated entrypoint/import surface")
        for surface_name, text in [("preset", preset), ("index", index), ("mod", mod)]:
            needle = "claude_code" if surface_name == "mod" and name == "claude-code" else ("claude" if name == "claude-code" else name)
            if needle not in text.lower() and spec["preset_id"] not in text.lower():
                fail(f"{surface_name}: missing {name} shared harness reference")
    notes.append("validated_config_surfaces=codex,claude-code,opencode,openclaw,hermes")


def validate_native_recovery_handoff_surfaces() -> None:
    required = {
        "scripts/handoff-latest.sh": ["handoff"],
        "scripts/memd-continuity-status.sh": ["wake", "mem", "events"],
        "scripts/verify/25-5-harness-process-replay.sh": ["claude-code", "codex", "ollama", "private"],
        "crates/memd-client/src/runtime/resume/mod.rs": ["resume"],
        "crates/memd-client/src/runtime/resume/wakeup.rs": ["wake"],
        "crates/memd-client/src/runtime/resume/recovery_signals.rs": ["signal"],
    }
    for rel, needles in required.items():
        text = read_required(rel).lower()
        for needle in needles:
            if needle.lower() not in text:
                fail(f"{rel}: missing native recovery/handoff term {needle}")
    handoff_docs = root / "docs/handoff"
    if not handoff_docs.is_dir() or not any(handoff_docs.iterdir()):
        fail("docs/handoff must exist and contain native handoff documentation")
    notes.append("validated_native_surfaces=handoff,resume,wake,recovery")


def validate_replay_report(report: Path, *, generated: bool) -> None:
    try:
        data = json.loads(report.read_text(encoding="utf-8"))
    except Exception as exc:
        fail(f"invalid replay artifact {report}: {exc}")
        return
    label = "generated" if generated else "registered"
    if data.get("status") != "pass":
        fail(f"{label} replay status must be pass: {report}")
    ids = data.get("ids")
    if not isinstance(ids, dict):
        fail(f"{label} replay ids must be an object")
        return
    for key in ["private", "stale", "correction", "procedure"]:
        if not isinstance(ids.get(key), str) or not ids[key]:
            fail(f"{label} replay missing memory object id {key}")
    if len(set(ids.values())) != len(ids):
        fail(f"{label} replay memory object ids must be unique")
    if data.get("codex_truth_top_id") != ids.get("correction"):
        fail(f"{label} replay must prove corrected memory object is top truth")
    if data.get("codex_private_visible") is not False:
        fail(f"{label} replay must prove private Claude memory is not visible to Codex")
    sections = set(data.get("ollama_packet_sections") or [])
    for section in ["System Guard", "Pinned Corrections", "Active Truth", "Evidence", "Procedures", "Open Conflicts", "Source IDs"]:
        if section not in sections:
            fail(f"{label} replay missing strict context section {section}")
    if not isinstance(data.get("ollama_packet_chars"), int) or data["ollama_packet_chars"] <= 0:
        fail(f"{label} replay must report non-empty strict packet chars")
    notes.append(f"validated_{label}_memory_object_consistency={report}")


def run_clean_local_replay() -> None:
    script = root / "scripts/verify/25-5-harness-process-replay.sh"
    if not script.is_file():
        fail("missing cross-harness process replay script")
        return
    before_dynamic = git_porcelain(["docs/verification/25-5-memory-os-runs", ".memd"])
    tmp = Path(tempfile.mkdtemp(prefix="memd-cross-harness-continuity."))
    try:
        env = os.environ.copy()
        env.update({"OUT_DIR": str(tmp / "out"), "RUN_DATE": "local-25-5-cross-harness-continuity", "MEMD_CARGO_TARGET_DIR": str(root / "target")})
        completed = subprocess.run(["bash", str(script)], cwd=root, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=900, check=False)
        if completed.returncode != 0:
            fail("generated harness process replay failed with " f"{completed.returncode}\nstdout:\n{completed.stdout[-4000:]}\nstderr:\n{completed.stderr[-4000:]}")
            return
        report = tmp / "out" / "local-25-5-cross-harness-continuity-harness-process-replay.json"
        if not report.is_file():
            fail(f"generated replay report missing: {report}")
            return
        validate_replay_report(report, generated=True)
        if completed.stdout.strip():
            notes.append("generated_replay_stdout=" + completed.stdout.strip().splitlines()[-1])
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    after_dynamic = git_porcelain(["docs/verification/25-5-memory-os-runs", ".memd"])
    if before_dynamic is not None and after_dynamic is not None and before_dynamic != after_dynamic:
        fail("process replay changed registered dynamic artifact status")
    notes.append("validated_local_artifact_cleanliness=temp_OUT_DIR_no_registered_dynamic_changes")


def validate_registered_replay_if_present() -> None:
    replay_dir = root / "docs/verification/25-5-memory-os-runs"
    replay_files = sorted(replay_dir.glob("*-harness-process-replay.json")) if replay_dir.exists() else []
    if replay_files:
        validate_replay_report(replay_files[-1], generated=False)
    else:
        notes.append("no registered dated replay JSON found; generated temp replay supplies current local proof")


def validate_claim_boundaries() -> None:
    registry = json.loads(read_required("docs/verification/features.registry.json"))
    feature = next((f for f in registry.get("features", []) if f.get("id") == "feature.cross_harness_continuity"), None)
    if not feature:
        fail("registry missing feature.cross_harness_continuity")
        return
    expected = {"current_status": "partial", "proof_status": "strong", "dogfood_status": "ad_hoc", "external_status": "none"}
    for key, value in expected.items():
        if feature.get(key) != value:
            fail(f"registry {key} must be {value!r}, got {feature.get(key)!r}")
    if feature.get("blocks_25_25") is not True:
        fail("registry blocks_25_25 must remain true")
    expected_commands = {"bash scripts/verify/feature-cross-harness-continuity-proof.sh", "bash scripts/verify/25-5-harness-process-replay.sh"}
    missing_commands = expected_commands - set(feature.get("proof_commands") or [])
    if missing_commands:
        fail(f"registry row missing proof commands: {sorted(missing_commands)}")
    expected_artifacts = {"docs/verification/feature-cross-harness-continuity-25.md", "scripts/verify/feature-cross-harness-continuity-proof.sh", "docs/verification/25-5-memory-os-runs/*-harness-process-replay.json"}
    missing_artifacts = expected_artifacts - set(feature.get("proof_artifacts") or [])
    if missing_artifacts:
        fail(f"registry row missing proof artifacts: {sorted(missing_artifacts)}")
    allowed = "\n".join(feature.get("allowed_claims") or []).lower()
    for needle in ["strong local 25/5", "codex", "claude", "opencode", "openclaw", "hermes", "memory object", "artifact cleanliness"]:
        if needle not in allowed:
            fail(f"allowed claim must mention {needle!r}")
    forbidden = "\n".join(feature.get("forbidden_claims") or []).lower()
    for needle in ["do not claim", "25/25", "production", "external", "sustained dogfood"]:
        if needle not in forbidden:
            fail(f"forbidden claim must mention {needle!r}")

    for rel in ["docs/verification/feature-cross-harness-continuity-25.md", "docs/verification/feature-coverage-report.md", "docs/verification/FEATURES.md"]:
        text = read_required(rel)
        lower = text.lower()
        if rel.endswith("feature-coverage-report.md") or rel.endswith("FEATURES.md"):
            m = re.search(r"feature\.cross_harness_continuity.*", lower)
            lower = m.group(0) if m else ""
        if "external_verified" in lower or "externally verified" in lower:
            fail(f"{rel}: contains external verification wording")
    notes.append("validated_claim_boundaries=local_strong_external_none_dogfood_ad_hoc")


validate_harness_config_surfaces()
validate_native_recovery_handoff_surfaces()
validate_registered_replay_if_present()
run_clean_local_replay()
validate_claim_boundaries()

if errors:
    for e in errors:
        print(f"feature-cross-harness-continuity-proof: ERROR: {e}", file=sys.stderr)
    sys.exit(1)

print("feature-cross-harness-continuity-proof: ok")
for note in notes:
    print("note: " + note)
PYPROOF
