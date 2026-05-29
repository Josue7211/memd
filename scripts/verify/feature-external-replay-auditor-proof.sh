#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

python3 - "$ROOT" <<'PY_PROOF'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
errors = []
notes = []

def fail(msg):
    errors.append(msg)

def rel(path):
    try:
        return str(path.relative_to(root))
    except Exception:
        return str(path)

def require_file(path):
    p = root / path
    if not p.is_file():
        fail(f"missing required file: {path}")
        return ""
    return p.read_text(encoding="utf-8", errors="replace")

def require_contains(path, needles):
    text = require_file(path)
    for needle in needles:
        if needle not in text:
            fail(f"{path} missing required text: {needle}")
    return text

def load_json(path):
    try:
        return json.loads((root / path).read_text(encoding="utf-8"))
    except Exception as exc:
        fail(f"invalid json {path}: {exc}")
        return None

def sha256_file(path):
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()

schema = load_json("docs/verification/features.schema.json")
if schema:
    required = set(schema.get("required") or [])
    for key in ["version", "purpose", "features"]:
        if key not in required:
            fail(f"features.schema.json required list missing {key}")
    feature_required = set(schema.get("properties", {}).get("features", {}).get("items", {}).get("required", []))
    for key in ["proof_commands", "proof_artifacts", "external_status", "forbidden_claims"]:
        if key not in feature_required:
            fail(f"features.schema.json feature required list missing {key}")

registry = load_json("docs/verification/features.registry.json")
if registry:
    matches = [f for f in registry.get("features", []) if f.get("id") == "feature.external_replay_auditor_proof"]
    if len(matches) != 1:
        fail(f"expected exactly one external replay/auditor registry entry, found {len(matches)}")
    else:
        f = matches[0]
        expected = {"current_status": "partial", "proof_status": "partial", "dogfood_status": "none", "external_status": "planned", "blocks_25_25": True}
        for key, value in expected.items():
            if f.get(key) != value:
                fail(f"registry {key} expected {value!r}, got {f.get(key)!r}")
        docs = f.get("docs") or []
        commands = f.get("proof_commands") or []
        if "docs/verification/feature-external-replay-auditor-proof-25.md" not in docs:
            fail("registry docs missing local external replay/auditor proof doc")
        if "bash scripts/verify/feature-external-replay-auditor-proof.sh" not in commands:
            fail("registry proof_commands missing local readiness proof command")
        if "bash scripts/verify/25-5-external-public-smoke.sh" not in commands:
            fail("registry proof_commands missing external public smoke replay preparation command")
        forbidden = " ".join(f.get("forbidden_claims") or []).lower()
        if "externally verified" not in forbidden or "independent" not in forbidden:
            fail("registry forbidden_claims must block externally verified/independent replay claims")
        allowed = " ".join(f.get("allowed_claims") or []).lower()
        if "local" not in allowed or "readiness" not in allowed:
            fail("registry allowed_claims should only allow local readiness claims")

require_contains("docs/verification/feature-external-replay-auditor-proof-25.md", [
    "Secondary/reference doc",
    "External status: planned",
    "Independent external replay: not verified",
    "do not claim externally verified",
    "Artifact immutability/checksums",
    "bash scripts/verify/feature-external-replay-auditor-proof.sh",
])
require_contains("scripts/verify/25-5-external-public-smoke.sh", [
    "External public-dataset proof runner",
    "PUBLIC_BENCH_LIMIT",
    "REPORT=",
    "DATASET_CACHE_DIR",
])

checklist = require_contains("docs/verification/EXTERNAL-25-STAR-VERIFIERS.md", [
    "External 25-Star Verifier Checklist",
    "Pass evidence",
    "Evidence packet template",
    "True 25-star requires EV-01 through EV-12 pass",
])
for idx in range(1, 13):
    if f"EV-{idx:02d}" not in checklist:
        fail(f"external verifier checklist missing EV-{idx:02d}")
for field in ["verifier_id:", "person/role:", "os:", "harness:", "commit:", "commands_run:", "result: pass|fail|blocked", "help_needed: yes|no", "artifacts:"]:
    if field not in checklist:
        fail(f"external verifier checklist missing evidence field {field}")

require_contains("docs/verification/25-star-human-trial-template.md", [
    "Instructions for verifier",
    "Result form",
    "Minimum pass",
    "no maintainer help",
    "can explain where `.memd` data lives",
    "memd setup-demo --summary",
])

release_dir = root / "docs/verification/release-1-0-0"
if release_dir.exists():
    if not release_dir.is_dir():
        fail("release proof bundle path is not a directory")
    files = sorted(p for p in release_dir.iterdir() if p.is_file())
    if not files:
        fail("release proof bundle exists but contains no files")
    stems = {}
    digest_count = 0
    for p in files:
        if p.is_symlink():
            fail(f"release proof bundle file must not be symlink: {rel(p)}")
        sha256_file(p)
        digest_count += 1
        stems.setdefault(p.stem, set()).add(p.suffix)
        if p.suffix == ".ndjson":
            line_count = 0
            statuses = set()
            for line_no, line in enumerate(p.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
                if not line.strip():
                    continue
                line_count += 1
                try:
                    obj = json.loads(line)
                except Exception as exc:
                    fail(f"invalid ndjson {rel(p)} line {line_no}: {exc}")
                    continue
                if "status" in obj:
                    statuses.add(str(obj["status"]))
            if line_count == 0:
                fail(f"empty ndjson artifact: {rel(p)}")
            if not statuses:
                fail(f"ndjson artifact has no status records: {rel(p)}")
    for stem, suffixes in stems.items():
        if ".md" in suffixes and ".ndjson" not in suffixes:
            fail(f"release proof markdown lacks paired ndjson artifact: {stem}")
    notes.append(f"release_bundle_sha256_files={digest_count}")
else:
    notes.append("release_bundle=absent_skipped")

public_bench = root / "docs/verification/PUBLIC_BENCHMARKS.md"
if public_bench.is_file():
    checked = 0
    skipped = 0
    for line in public_bench.read_text(encoding="utf-8", errors="replace").splitlines():
        if "sha256:" not in line or "|" not in line:
            continue
        cols = [c.strip().strip("`") for c in line.strip().strip("|").split("|")]
        if len(cols) < 8 or not cols[6].startswith("sha256:"):
            continue
        dataset_path = cols[5]
        expected = cols[6].split("sha256:", 1)[1].strip()
        candidate = Path(dataset_path)
        if not candidate.is_absolute():
            candidate = root / dataset_path
        if candidate.is_file():
            checked += 1
            actual = sha256_file(candidate)
            if actual != expected:
                fail(f"checksum mismatch for {dataset_path}: expected {expected}, got {actual}")
        else:
            skipped += 1
    notes.append(f"public_benchmark_checksums_checked={checked}")
    notes.append(f"public_benchmark_checksums_skipped_missing_local_files={skipped}")

if errors:
    for error in errors:
        print(f"feature-external-replay-auditor-proof: ERROR: {error}", file=sys.stderr)
    sys.exit(1)
for note in notes:
    print(f"feature-external-replay-auditor-proof: {note}")
print("feature-external-replay-auditor-proof: ok")
PY_PROOF
