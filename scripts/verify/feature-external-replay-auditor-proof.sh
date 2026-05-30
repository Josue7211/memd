#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

python3 - "$ROOT" <<'PY_PROOF'
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

root = Path(sys.argv[1])
errors = []
notes = []

EXPECTED_DATASETS = ["longmemeval", "locomo", "membench", "convomem"]
SHA_RE = re.compile(r"^sha256:[0-9a-f]{64}$")


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
    if p.is_symlink():
        fail(f"required file must be immutable regular file, not symlink: {path}")
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


def require_regular_artifact(path_text):
    if any(ch in path_text for ch in "*?[]") or " " in path_text:
        fail(f"external replay proof artifact must be a concrete path, got: {path_text}")
        return None
    p = root / path_text
    if not p.is_file():
        fail(f"missing external replay proof artifact: {path_text}")
        return None
    if p.is_symlink():
        fail(f"external replay proof artifact must not be symlink: {path_text}")
    digest = sha256_file(p)
    notes.append(f"artifact_sha256 {path_text} {digest}")
    return p


def git_porcelain():
    try:
        return subprocess.check_output(
            ["git", "status", "--porcelain"], cwd=root, text=True, stderr=subprocess.DEVNULL
        )
    except Exception:
        return None


def validate_external_public_report(report_path, *, generated):
    report_path = Path(report_path)
    try:
        data = json.loads(report_path.read_text(encoding="utf-8"))
    except Exception as exc:
        fail(f"invalid external-public-smoke report json {rel(report_path)}: {exc}")
        return
    label = "generated" if generated else "registered"
    if data.get("status") != "pass":
        fail(f"{label} external-public-smoke report status must be pass, got {data.get('status')!r}")
    if data.get("rag_url") is not None:
        fail(f"{label} external-public-smoke report must prove no external RAG URL, got {data.get('rag_url')!r}")
    if data.get("datasets") != EXPECTED_DATASETS:
        fail(f"{label} external-public-smoke datasets drift: {data.get('datasets')!r}")
    if not isinstance(data.get("limit"), int) or data.get("limit") < 1:
        fail(f"{label} external-public-smoke limit must be positive integer")
    if not isinstance(data.get("offset"), int) or data.get("offset") < 0:
        fail(f"{label} external-public-smoke offset must be non-negative integer")
    if data.get("failed") not in ([], None):
        fail(f"{label} external-public-smoke report has failed rows")
    rows = data.get("rows")
    if not isinstance(rows, list) or len(rows) != len(EXPECTED_DATASETS):
        fail(f"{label} external-public-smoke rows must cover {len(EXPECTED_DATASETS)} datasets")
        return
    seen = []
    for row in rows:
        if not isinstance(row, dict):
            fail(f"{label} external-public-smoke row must be object")
            continue
        dataset = row.get("dataset")
        seen.append(dataset)
        if dataset not in EXPECTED_DATASETS:
            fail(f"{label} external-public-smoke unexpected dataset {dataset!r}")
        if row.get("backend") != "memd":
            fail(f"{label} {dataset}: backend must be memd")
        if row.get("limit") != data.get("limit"):
            fail(f"{label} {dataset}: row limit does not match summary limit")
        url = row.get("dataset_source_url")
        if not isinstance(url, str) or not url.startswith("http"):
            fail(f"{label} {dataset}: dataset_source_url must be public http(s) URL")
        checksum = row.get("dataset_checksum")
        if not isinstance(checksum, str) or not SHA_RE.match(checksum):
            fail(f"{label} {dataset}: dataset_checksum must be sha256:<64 hex>")
        if row.get("dataset_items") is not None and row.get("dataset_items") < row.get("limit", 0):
            fail(f"{label} {dataset}: dataset_items smaller than limit")
        if row.get("failures"):
            fail(f"{label} {dataset}: failures must be empty")
        for metric in ["accuracy", "hit_rate", "recall_at_k"]:
            value = row.get(metric)
            if value is not None and value < 1.0:
                fail(f"{label} {dataset}: {metric} below 1.0 ({value})")
        answer_rate = row.get("answer_supported_top1_hit_rate")
        if answer_rate is not None and answer_rate < 1.0:
            fail(f"{label} {dataset}: answer_supported_top1_hit_rate below 1.0 ({answer_rate})")
        if row.get("answer_supported_top1_gaps"):
            fail(f"{label} {dataset}: answer_supported_top1_gaps must be empty")
        items = row.get("items")
        if not isinstance(items, list) or len(items) != row.get("limit"):
            fail(f"{label} {dataset}: item count must equal limit")
        else:
            for idx, item in enumerate(items):
                if not isinstance(item, dict):
                    fail(f"{label} {dataset}: item {idx} must be object")
                    continue
                if not item.get("question_id") or not item.get("question"):
                    fail(f"{label} {dataset}: item {idx} missing question_id/question")
                if item.get("hit") is not True:
                    fail(f"{label} {dataset}: item {idx} was not a hit")
                if not item.get("top_id"):
                    fail(f"{label} {dataset}: item {idx} missing top_id")
    if seen != EXPECTED_DATASETS:
        fail(f"{label} external-public-smoke row order/coverage drift: {seen!r}")
    notes.append(f"{label}_external_public_smoke_report_validated={rel(report_path)}")


def run_external_public_smoke_without_dirty_noise():
    script = root / "scripts/verify/25-5-external-public-smoke.sh"
    if not script.is_file():
        fail("missing external public smoke script")
        return
    if os.environ.get("SKIP_EXTERNAL_PUBLIC_SMOKE") == "1":
        fail("SKIP_EXTERNAL_PUBLIC_SMOKE=1 is not allowed for strong local external replay/auditor proof")
        return
    before = git_porcelain()
    tmp = Path(tempfile.mkdtemp(prefix="memd-external-auditor-proof."))
    try:
        env = os.environ.copy()
        env.update({
            "OUT_DIR": str(tmp / "out"),
            "DATASET_CACHE_DIR": str(tmp / "dataset-cache"),
            "RUN_DATE": "local-readiness",
            "RUN_LABEL": "external-public-smoke",
            "SUITE_NAME": "25_5_external_public_smoke_local_readiness",
            "PUBLIC_BENCH_LIMIT": env.get("PUBLIC_BENCH_LIMIT", "1"),
            "PUBLIC_BENCH_OFFSET": env.get("PUBLIC_BENCH_OFFSET", "0"),
            "PUBLIC_BENCH_TIMEOUT": env.get("PUBLIC_BENCH_TIMEOUT", "900"),
        })
        env.pop("MEMD_RAG_URL", None)
        completed = subprocess.run(
            ["bash", str(script)],
            cwd=root,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=int(env["PUBLIC_BENCH_TIMEOUT"]) * len(EXPECTED_DATASETS) + 240,
            check=False,
        )
        if completed.returncode != 0:
            fail(
                "generated external-public-smoke command failed with "
                f"{completed.returncode}\nstdout:\n{completed.stdout[-4000:]}\nstderr:\n{completed.stderr[-4000:]}"
            )
            return
        report = tmp / "out" / "local-readiness-external-public-smoke.json"
        if not report.is_file():
            fail(f"generated external-public-smoke report missing: {report}")
            return
        validate_external_public_report(report, generated=True)
        stdout_lines = completed.stdout.strip().splitlines()
        if stdout_lines:
            notes.append("generated_external_public_smoke_stdout=" + stdout_lines[-1])
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    after = git_porcelain()
    if before is not None and after is not None and before != after:
        fail("external-public-smoke generation changed git porcelain; proof must not leave dirty noise")


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
registered_smoke_reports = []
if registry:
    matches = [f for f in registry.get("features", []) if f.get("id") == "feature.external_replay_auditor_proof"]
    if len(matches) != 1:
        fail(f"expected exactly one external replay/auditor registry entry, found {len(matches)}")
    else:
        f = matches[0]
        expected = {"current_status": "partial", "proof_status": "strong", "dogfood_status": "none", "external_status": "planned", "blocks_25_25": True}
        for key, value in expected.items():
            if f.get(key) != value:
                fail(f"registry {key} expected {value!r}, got {f.get(key)!r}")
        docs = f.get("docs") or []
        commands = f.get("proof_commands") or []
        artifacts = f.get("proof_artifacts") or []
        if "docs/verification/feature-external-replay-auditor-proof-25.md" not in docs:
            fail("registry docs missing local external replay/auditor proof doc")
        for doc in docs:
            require_regular_artifact(doc)
        if "bash scripts/verify/feature-external-replay-auditor-proof.sh" not in commands:
            fail("registry proof_commands missing local readiness proof command")
        if "bash scripts/verify/25-5-external-public-smoke.sh" not in commands:
            fail("registry proof_commands missing external public smoke replay preparation command")
        for artifact in artifacts:
            p = require_regular_artifact(artifact)
            if p and p.name.endswith("external-public-smoke.json"):
                registered_smoke_reports.append(p)
        if not registered_smoke_reports:
            fail("registry proof_artifacts must include a concrete external-public-smoke JSON report")
        forbidden = " ".join(f.get("forbidden_claims") or []).lower()
        if "externally verified" not in forbidden or "independent" not in forbidden:
            fail("registry forbidden_claims must block externally verified/independent replay claims")
        allowed = " ".join(f.get("allowed_claims") or []).lower()
        if "strong local" not in allowed or "readiness" not in allowed:
            fail("registry allowed_claims should only allow strong local readiness claims")

require_contains("docs/verification/feature-external-replay-auditor-proof-25.md", [
    "Secondary/reference doc",
    "Proof status: strong local readiness proof",
    "External status: planned",
    "Independent external replay: not verified",
    "do not claim externally verified",
    "Artifact immutability/checksums",
    "25-5-external-public-smoke artifact generation/validation",
    "bash scripts/verify/feature-external-replay-auditor-proof.sh",
])
require_contains("scripts/verify/25-5-external-public-smoke.sh", [
    "External public-dataset proof runner",
    "PUBLIC_BENCH_LIMIT",
    "REPORT=",
    "DATASET_CACHE_DIR",
    "MEMD_RAG_URL",
    "dataset_checksum",
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

for report in registered_smoke_reports:
    validate_external_public_report(report, generated=False)

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

if not errors:
    run_external_public_smoke_without_dirty_noise()

if errors:
    for error in errors:
        print(f"feature-external-replay-auditor-proof: ERROR: {error}", file=sys.stderr)
    sys.exit(1)
for note in notes:
    print(f"feature-external-replay-auditor-proof: {note}")
print("feature-external-replay-auditor-proof: ok")
PY_PROOF
