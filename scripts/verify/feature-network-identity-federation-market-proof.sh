#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOC="$ROOT/docs/verification/feature-network-identity-federation-market-25.md"
REGISTRY="$ROOT/docs/verification/features.registry.json"
MEMD_BIN="${MEMD_BIN:-$ROOT/target/debug/memd}"
KEEP="${MEMD_NETWORK_IDENTITY_PROOF_KEEP:-0}"
TMP="$(mktemp -d)"

cleanup() {
  if [ "$KEEP" = "1" ]; then
    echo "feature-network-identity-federation-market-proof: kept temp dir: $TMP"
  else
    rm -rf "$TMP"
  fi
}
trap cleanup EXIT

fail() {
  echo "feature-network-identity-federation-market-proof: ERROR: $*" >&2
  exit 1
}

require_file() {
  local file="$1"
  [ -f "$ROOT/$file" ] || fail "missing required file: $file"
}

cd "$ROOT"

for file in \
  docs/verification/feature-network-identity-federation-market-25.md \
  docs/verification/features.registry.json \
  docs/verification/feature-coverage-report.md \
  docs/verification/FEATURES.md \
  integrations/codex/README.md \
  integrations/hermes/README.md \
  integrations/hooks/README.md \
  docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.md \
  docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.ndjson \
  docs/verification/25-star-CONTRACT.md \
  docs/verification/25-star-phase-ledger.md; do
  require_file "$file"
done

if [ ! -x "$MEMD_BIN" ]; then
  cargo build -p memd-client --bin memd >/dev/null
fi
[ -x "$MEMD_BIN" ] || fail "memd binary not found after build: $MEMD_BIN"

PROOF_ROOT="$TMP/single-user-org-proof"
BUNDLE="$PROOF_ROOT/.memd"
mkdir -p "$PROOF_ROOT"
(
  cd "$PROOF_ROOT"
  "$MEMD_BIN" setup \
    --output "$BUNDLE" \
    --project local-25-5-single-org \
    --namespace org-alpha \
    --agent codex \
    --session local-proof-session \
    --tab-id local-proof-tab \
    --route auto \
    --intent current_task \
    --summary \
    --force \
    --allow-localhost-read-only-fallback >/dev/null
  "$MEMD_BIN" wake --output "$BUNDLE" --route auto --intent current_task --write >/dev/null || true
  "$MEMD_BIN" handoff --output "$BUNDLE" >/dev/null || true
)

v26_status="absent_pending_not_claimed"
if [ -x "$ROOT/scripts/verify/v26-network-identity-proof.sh" ]; then
  bash "$ROOT/scripts/verify/v26-network-identity-proof.sh"
  v26_status="script_ran"
elif [ -f "$ROOT/scripts/verify/v26-network-identity-proof.sh" ]; then
  bash "$ROOT/scripts/verify/v26-network-identity-proof.sh"
  v26_status="script_ran"
fi

python3 - "$ROOT" "$DOC" "$REGISTRY" "$v26_status" "$BUNDLE" <<'PY'
import json
import shlex
import sys
from pathlib import Path

root = Path(sys.argv[1])
registry_path = Path(sys.argv[3])
v26_status = sys.argv[4]
bundle = Path(sys.argv[5])
errors = []

def rel(path: Path) -> str:
    try:
        return str(path.relative_to(root))
    except Exception:
        return str(path)

def require_text(path: str, needles):
    p = root / path
    if not p.is_file():
        errors.append(f"missing required file: {path}")
        return ""
    text = p.read_text(errors="replace")
    for needle in needles:
        if needle not in text:
            errors.append(f"{path} missing required text: {needle}")
    return text

def parse_env_value(line: str):
    key, value = line.split("=", 1)
    try:
        parsed = shlex.split(value, posix=True)
    except Exception:
        parsed = [value.strip().strip("'\"")]
    return key, parsed[0] if parsed else ""

doc_text = require_text("docs/verification/feature-network-identity-federation-market-25.md", [
    "strong local proof",
    "Single user/org across app surfaces",
    "one user/org memory identity across app surfaces",
    "deterministic generated-bundle check",
    "absent_pending_not_claimed",
    "Federation and market boundaries",
    "Forbidden claim: do not claim active network identity service",
    "external_status: none",
])

require_text("integrations/codex/README.md", [
    "memd owns the same memory control plane",
    "Codex should use the same `memd` surface as every other agent",
    "`.memd/wake.md`",
    "`.memd/mem.md`",
    "`.memd/events.md`",
    "same shared local files",
])
require_text("integrations/hermes/README.md", [
    "Hermes should use `memd` as the shared memory control plane",
    "same core memory loop",
    "`.memd/wake.md`",
    "`.memd/mem.md`",
    "`.memd/events.md`",
    "same visible truth as the other agent packs",
])
require_text("integrations/hooks/README.md", [
    "memd setup --output .memd --project <project> --namespace <namespace> --agent <agent>",
    "codex.sh",
    "claude-code.sh",
    "openclaw.sh",
    "hermes.sh",
    "same bundle",
])
require_text("README.md", [
    "portability across Codex, Claude Code, OpenClaw, Hermes, OpenCode, and future harnesses",
])

for required in ["config.json", "env", "mem.md", "COMMANDS.md"]:
    if not (bundle / required).is_file():
        errors.append(f"generated bundle missing {required}")
for agent in ["codex", "claude-code", "openclaw", "hermes", "opencode"]:
    script = bundle / "agents" / f"{agent}.sh"
    if not script.is_file():
        errors.append(f"generated bundle missing agent entrypoint {agent}.sh")
        continue
    text = script.read_text(errors="replace")
    for needle in [
        'source "$MEMD_BUNDLE_ROOT/env"',
        'memd wake --output "$MEMD_BUNDLE_ROOT"',
        f'export MEMD_AGENT="{agent}"',
    ]:
        if needle not in text:
            errors.append(f"{agent}.sh missing shared-bundle needle: {needle}")
    if "MEMD_PROJECT=" in text or "MEMD_NAMESPACE=" in text:
        errors.append(f"{agent}.sh must not fork project/namespace identity")
try:
    config = json.loads((bundle / "config.json").read_text())
except Exception as exc:
    errors.append(f"generated config invalid json: {exc}")
    config = {}
env = {}
if (bundle / "env").is_file():
    for line in (bundle / "env").read_text().splitlines():
        if line.startswith("MEMD_") and "=" in line:
            key, value = parse_env_value(line)
            env[key] = value
expected_identity = {
    "project": "local-25-5-single-org",
    "namespace": "org-alpha",
    "session": "local-proof-session",
    "tab_id": "local-proof-tab",
}
for key, expected in expected_identity.items():
    if config.get(key) != expected:
        errors.append(f"generated config {key} expected {expected!r}, got {config.get(key)!r}")
if env.get("MEMD_PROJECT") != expected_identity["project"]:
    errors.append(f"generated env MEMD_PROJECT mismatch: {env.get('MEMD_PROJECT')!r}")
if env.get("MEMD_NAMESPACE") != expected_identity["namespace"]:
    errors.append(f"generated env MEMD_NAMESPACE mismatch: {env.get('MEMD_NAMESPACE')!r}")
if env.get("MEMD_SESSION") != expected_identity["session"]:
    errors.append(f"generated env MEMD_SESSION mismatch: {env.get('MEMD_SESSION')!r}")
if config.get("agent") != "codex":
    errors.append(f"generated config initial agent expected codex, got {config.get('agent')!r}")
if len({config.get("project"), env.get("MEMD_PROJECT")}) != 1:
    errors.append("project identity forked between config and env")
if len({config.get("namespace"), env.get("MEMD_NAMESPACE")}) != 1:
    errors.append("namespace identity forked between config and env")
identity_tokens = [config.get("project"), env.get("MEMD_PROJECT"), config.get("namespace"), env.get("MEMD_NAMESPACE")]
if identity_tokens.count("local-25-5-single-org") != 2 or identity_tokens.count("org-alpha") != 2:
    errors.append("single user/org lane evidence missing or duplicated unexpectedly")

require_text("docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.md", [
    "1000-user synthetic federation preserved isolation",
    "Remaining gate: real 30-day marketplace dogfood with cross-user installs",
])
ndjson_path = root / "docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.ndjson"
try:
    rows = [json.loads(line) for line in ndjson_path.read_text().splitlines() if line.strip()]
except Exception as exc:
    errors.append(f"invalid V17 ndjson: {exc}")
    rows = []
if rows:
    if not any(r.get("check") == "federation_scale" and r.get("synthetic_users") == 1000 and r.get("isolation_violations") == 0 for r in rows):
        errors.append("V17 ndjson missing synthetic federation isolation evidence")
    if not any(r.get("check") == "axis_lift" and r.get("gate") == "code_complete_dogfood_pending" for r in rows):
        errors.append("V17 ndjson missing dogfood-pending gate")

require_text("docs/verification/25-star-CONTRACT.md", [
    "V26 Network Identity",
    "V27 Federation Protocol",
    "V28 Agent Work Market",
])
require_text("docs/verification/25-star-phase-ledger.md", [
    "V26",
    "network identity proof",
    "V27",
    "federation proof",
    "V28",
    "work market proof",
])

v26_artifacts = sorted((root / "docs/verification").glob("v26-proof-runs/*network-identity*"))
if v26_status == "script_ran" and "script_ran" not in doc_text:
    errors.append("V26 script ran but doc does not describe script-run handling")
if not v26_artifacts and v26_status != "absent_pending_not_claimed":
    errors.append("V26 status inconsistent with missing artifacts")
if not v26_artifacts and "V26 is reported as `absent_pending_not_claimed`" not in doc_text:
    errors.append("doc must honestly state absent V26 artifact handling")

try:
    registry = json.loads(registry_path.read_text())
except Exception as exc:
    errors.append(f"invalid registry json: {exc}")
    registry = {}
features = [f for f in registry.get("features", []) if f.get("id") == "feature.network_identity_federation_market"]
if len(features) != 1:
    errors.append(f"expected exactly one network identity/federation/market registry row, got {len(features)}")
else:
    f = features[0]
    expected = {
        "current_status": "partial",
        "proof_status": "strong",
        "dogfood_status": "none",
        "external_status": "none",
        "blocks_25_25": True,
    }
    for key, value in expected.items():
        if f.get(key) != value:
            errors.append(f"registry {key} expected {value!r}, got {f.get(key)!r}")
    for item in [
        "docs/verification/feature-network-identity-federation-market-25.md",
        "integrations/codex/README.md",
        "integrations/hermes/README.md",
        "integrations/hooks/README.md",
    ]:
        if item not in f.get("docs", []):
            errors.append(f"registry docs missing {item}")
    for item in [
        "bash scripts/verify/feature-network-identity-federation-market-proof.sh",
        "bash scripts/verify/feature-registry-audit.sh",
    ]:
        if item not in f.get("proof_commands", []):
            errors.append(f"registry missing proof command {item}")
    for item in [
        "docs/verification/feature-network-identity-federation-market-25.md",
        "scripts/verify/feature-network-identity-federation-market-proof.sh",
        "docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.md",
        "docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.ndjson",
    ]:
        if item not in f.get("proof_artifacts", []):
            errors.append(f"registry proof_artifacts missing {item}")
    joined_allowed = " ".join(f.get("allowed_claims", []))
    joined_forbidden = " ".join(f.get("forbidden_claims", []))
    if "Strong local proof" not in joined_allowed or "single user/org local memory identity" not in joined_allowed:
        errors.append("registry allowed_claims missing strong single user/org scope")
    for phrase in ["active network identity service", "cross-org federation", "public marketplace", "external verification"]:
        if phrase not in joined_forbidden:
            errors.append(f"registry forbidden_claims missing {phrase!r}")

require_text("docs/verification/feature-coverage-report.md", [
    "| `feature.network_identity_federation_market` | `partial` | `strong` | `none` | `none` |",
    "Strong local proof validates one user/org local memory identity across generated Codex/Hermes/Claude/OpenClaw/OpenCode surfaces",
])
require_text("docs/verification/FEATURES.md", [
    "| `feature.network_identity_federation_market` | network identity/federation/market layer | `partial` | `strong` | `none` | `none` | yes |",
])

if errors:
    for error in errors:
        print(f"feature-network-identity-federation-market-proof: ERROR: {error}", file=sys.stderr)
    sys.exit(1)

v26_artifact_text = ", ".join(rel(p) for p in v26_artifacts) if v26_artifacts else "none"
print("feature-network-identity-federation-market-proof: ok")
print("single_user_org_generated_bundle=pass project=local-25-5-single-org namespace=org-alpha surfaces=codex,claude-code,openclaw,hermes,opencode")
print(f"v26_network_identity={v26_status} artifacts={v26_artifact_text}")
print("federation_market_boundary=pass v17_local_synthetic_cited=true v27_v28_planned_not_claimed=true external_status=none")
PY

bash "$ROOT/scripts/verify/feature-registry-audit.sh"
