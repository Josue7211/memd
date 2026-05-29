#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOC="$ROOT/docs/verification/feature-memory-core-25.md"
REGISTRY="$ROOT/docs/verification/features.registry.json"

fail() {
  echo "feature-memory-core-proof: ERROR: $*" >&2
  exit 1
}

require_file() {
  local path=$1
  [[ -f "$ROOT/$path" ]] || fail "missing file: $path"
}

require_glob() {
  local pattern=$1
  compgen -G "$ROOT/$pattern" >/dev/null || fail "missing artifact glob: $pattern"
}

require_doc_text() {
  local text=$1
  grep -Fq "$text" "$DOC" || fail "doc missing text: $text"
}

require_file "docs/verification/feature-memory-core-25.md"
require_file "docs/verification/features.registry.json"
require_file "scripts/verify/feature-registry-audit.sh"
require_file "scripts/memd-cargo-guard.sh"

for axis in capture lookup resume corrections provenance trust; do
  require_doc_text "| ${axis^} |"
done
require_doc_text "not an external benchmark"
require_doc_text "does **not** claim perfect recall"
require_doc_text 'external_status`: none'

# Source and test surfaces cited by the proof.
require_file "crates/memd-client/src/cli/cli_memory_runtime.rs"
require_file "crates/memd-client/src/cli/cli_correction_runtime.rs"
require_file "crates/memd-client/src/runtime/recall/mod.rs"
require_file "crates/memd-core/src/correction/mod.rs"
require_file "crates/memd-schema/src/memory_surfaces.rs"
require_file "crates/memd-server/src/store_memory_runtime.rs"
require_file "crates/memd-server/src/store_memory_domains.rs"
require_file "crates/memd-client/src/main_tests/correction_e2e_tests/mod.rs"
require_file "crates/memd-client/src/main_tests/recall_depth_tests/mod.rs"
require_file "crates/memd-client/src/main_tests/runtime_memory_tests/mod.rs"
require_file "crates/memd-server/src/tests/memory_behaviors.rs"
require_file "crates/memd-client/src/benchmark/substrate/provenance_auditor.rs"
require_file "crates/memd-client/src/benchmark/substrate/provenance_integrity.rs"
require_glob "docs/verification/release-0-1-0/*axis-correction_retention*"
require_glob "docs/verification/release-0-1-0/*axis-trust_provenance*"
require_file "scripts/verify/v18-correction-graph-suite.sh"
require_file "scripts/verify/v19-zk-provenance-suite.sh"
require_file "docs/backlog/m3/2026-04-14-trust-hierarchy-unproven.md"

python3 - "$REGISTRY" <<'PY'
import json, sys
from pathlib import Path
registry = json.loads(Path(sys.argv[1]).read_text())
features = [f for f in registry.get("features", []) if f.get("id") == "feature.memory_core"]
if len(features) != 1:
    raise SystemExit(f"expected exactly one feature.memory_core entry, got {len(features)}")
f = features[0]
assert f["category"] == "memory capture/lookup/recall/corrections/provenance/trust"
assert f["current_status"] == "partial"
assert f["proof_status"] == "strong"
assert f["external_status"] == "none"
assert "docs/verification/feature-memory-core-25.md" in f["docs"]
assert "bash scripts/verify/feature-memory-core-proof.sh" in f["proof_commands"]
for term in ["perfect recall", "trust", "external"]:
    if term not in " ".join(f.get("forbidden_claims", [])).lower():
        raise SystemExit(f"forbidden_claims missing {term!r}")
print("feature-memory-core-proof: registry entry ok")
PY

bash "$ROOT/scripts/verify/feature-registry-audit.sh"

echo "feature-memory-core-proof: ok"
