#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PRODUCT="$ROOT/docs/product/INDEX.md"
PROOF="$ROOT/docs/verification/feature-docs-product-education-25.md"
REGISTRY="$ROOT/docs/verification/features.registry.json"
FEATURES="$ROOT/docs/verification/FEATURES.md"
REPORT="$ROOT/docs/verification/feature-coverage-report.md"
CLI_ARGS="$ROOT/crates/memd-client/src/cli/args_memory.rs"
CLI_RUNTIME_ARGS="$ROOT/crates/memd-client/src/cli/args_runtime.rs"

fail() { echo "feature-docs-product-education-proof: ERROR: $*" >&2; exit 1; }
require_file() { [[ -f "$1" ]] || fail "missing file: ${1#$ROOT/}"; }
require_text() { local file=$1 pattern=$2 label=$3; grep -Eiq -- "$pattern" "$file" || fail "${file#$ROOT/} missing $label"; }
require_literal() { local file=$1 literal=$2 label=$3; grep -Fq -- "$literal" "$file" || fail "${file#$ROOT/} missing $label: $literal"; }

for file in "$PRODUCT" "$PROOF" "$REGISTRY" "$FEATURES" "$REPORT" "$ROOT/START-HERE.md" "$ROOT/README.md" "$ROOT/docs/setup/README.md" "$ROOT/docs/setup/install.md" "$ROOT/docs/setup/first-run.md" "$ROOT/docs/setup/troubleshooting.md" "$CLI_ARGS" "$CLI_RUNTIME_ARGS"; do
  require_file "$file"
done

require_text "$PRODUCT" "New-User Path" "new-user path heading"
require_literal "$PRODUCT" "START-HERE.md" "START-HERE pointer"
require_literal "$PRODUCT" "README.md" "README pointer"
require_literal "$PRODUCT" "docs/WHERE-AM-I.md" "WHERE-AM-I pointer"
require_literal "$PRODUCT" "docs/setup/README.md" "setup/getting-started pointer"
require_text "$PRODUCT" "Plain-Language Product Summary" "plain-language summary"
require_text "$PRODUCT" "Setup Command Examples" "setup command examples"
require_text "$PRODUCT" "Jargon Guardrail" "jargon guardrail"
require_text "$PRODUCT" "Claim-to-Proof Rule" "claim-to-proof rule"
require_text "$PRODUCT" "External validation status.*pending|external validation remains pending" "honest external pending language"
require_text "$PRODUCT" "Dogfood evidence remains pending|dogfood evidence pending" "honest dogfood pending language"
require_text "$PRODUCT" "Local 25/5 target" "local 25/5 honesty language"

common_commands=("memd setup --guided --summary" "memd setup --interactive" "memd doctor --summary" "memd setup-demo --summary")
for cmd in "${common_commands[@]}"; do
  require_literal "$ROOT/README.md" "$cmd" "README quickstart command"
  require_literal "$ROOT/docs/setup/README.md" "$cmd" "setup README command"
  require_literal "$PRODUCT" "$cmd" "product education command"
done
for cmd in "memd setup --guided --summary" "memd setup --interactive" "memd setup-demo --summary"; do
  require_literal "$ROOT/START-HERE.md" "$cmd" "START-HERE command"
done
require_literal "$ROOT/START-HERE.md" "README.md#quickstart" "README quickstart link"
require_literal "$ROOT/README.md" "./docs/setup/README.md" "setup README link"
require_literal "$ROOT/README.md" "./docs/setup/troubleshooting.md" "setup troubleshooting link"

require_text "$ROOT/crates/memd-client/src/cli/args.rs" "Setup\(SetupArgs\)" "setup subcommand"
require_text "$ROOT/crates/memd-client/src/cli/args.rs" "name = \"setup-demo\"" "setup-demo subcommand"
require_text "$ROOT/crates/memd-client/src/cli/args.rs" "Doctor\(DoctorArgs\)" "doctor subcommand"
require_text "$CLI_ARGS" "struct SetupArgs" "SetupArgs definition"
require_text "$CLI_ARGS" "guided" "setup --guided flag"
require_text "$CLI_ARGS" "interactive" "setup --interactive flag"
require_text "$CLI_ARGS" "summary" "setup/setup-demo/doctor --summary flag"
require_text "$CLI_ARGS" "struct SetupDemoArgs" "SetupDemoArgs definition"
require_text "$CLI_ARGS" "struct DoctorArgs" "DoctorArgs definition"
require_text "$CLI_ARGS" "repair" "doctor --repair flag"
require_text "$CLI_RUNTIME_ARGS" "struct ResumeArgs" "ResumeArgs definition"
require_text "$CLI_RUNTIME_ARGS" "intent" "resume --intent flag"
require_text "$CLI_ARGS" "struct StatusArgs" "StatusArgs definition"

require_literal "$ROOT/docs/setup/install.md" "scripts/install-memd.sh" "install command example"
require_literal "$ROOT/docs/setup/install.md" "memd doctor --summary" "install verification command"
require_literal "$ROOT/docs/setup/first-run.md" "memd status --output .memd --summary" "first-run status command"
require_literal "$ROOT/docs/setup/first-run.md" "memd resume --output .memd --intent current_task" "first-run resume command"
require_literal "$ROOT/docs/setup/troubleshooting.md" "memd doctor --repair --summary" "doctor repair troubleshooting command"

python3 - "$REGISTRY" "$FEATURES" "$REPORT" "$PROOF" <<'PY'
import json, re, sys
from pathlib import Path
registry_path, features_path, report_path, proof_path = map(Path, sys.argv[1:])
registry = json.loads(registry_path.read_text())
rows = [f for f in registry.get("features", []) if f.get("id") == "feature.docs_product_education"]
if len(rows) != 1:
    raise SystemExit(f"expected exactly one feature.docs_product_education row, found {len(rows)}")
f = rows[0]
required_docs = {"README.md","START-HERE.md","docs/setup/README.md","docs/product/INDEX.md","docs/verification/feature-docs-product-education-25.md"}
missing_docs = sorted(required_docs - set(f.get("docs", [])))
if missing_docs:
    raise SystemExit("registry row missing docs: " + ", ".join(missing_docs))
required_commands = {"bash scripts/doc-lint.sh","bash scripts/verify/feature-registry-audit.sh","bash scripts/verify/feature-docs-product-education-proof.sh"}
missing_commands = sorted(required_commands - set(f.get("proof_commands", [])))
if missing_commands:
    raise SystemExit("registry row missing proof commands: " + ", ".join(missing_commands))
required_artifacts = {"docs/verification/feature-coverage-report.md","docs/verification/feature-docs-product-education-25.md","scripts/verify/feature-docs-product-education-proof.sh"}
missing_artifacts = sorted(required_artifacts - set(f.get("proof_artifacts", [])))
if missing_artifacts:
    raise SystemExit("registry row missing proof artifacts: " + ", ".join(missing_artifacts))
if f.get("proof_status") != "strong":
    raise SystemExit(f"expected proof_status strong, found {f.get('proof_status')!r}")
if f.get("external_status") not in {"none", "planned"}:
    raise SystemExit("docs/product education must not claim external verification")
if f.get("blocks_25_25") is not True:
    raise SystemExit("docs/product education must still block whole-product 25/25")
allowed = " ".join(f.get("allowed_claims", [])).lower()
for token in ["local", "navigation", "claim-to-proof", "pending"]:
    if token not in allowed:
        raise SystemExit(f"allowed_claims missing honest local scope token: {token}")
forbidden = " ".join(f.get("forbidden_claims", [])).lower()
for token in ["do not claim", "external", "dogfood", "production", "25/25"]:
    if token not in forbidden:
        raise SystemExit(f"forbidden_claims missing blocker token: {token}")
features_row = re.search(r"^\| `feature\.docs_product_education` \| .* \|$", features_path.read_text(), re.MULTILINE)
if not features_row:
    raise SystemExit("FEATURES table missing docs_product_education row")
if "| `strong` |" not in features_row.group(0):
    raise SystemExit("FEATURES table row must show proof strong")
report_row = re.search(r"^\| `feature\.docs_product_education` \| .* \|$", report_path.read_text(), re.MULTILINE)
if not report_row:
    raise SystemExit("coverage report missing docs_product_education row")
if "| `strong` |" not in report_row.group(0):
    raise SystemExit("coverage report row must show proof strong")
if "external" not in report_row.group(0).lower() or "25/25" not in report_row.group(0).lower():
    raise SystemExit("coverage report row must keep external/25/25 honesty blocker language")
proof_text = proof_path.read_text().lower()
for phrase in ["start-here/readme/setup/cli alignment","broken internal references","setup command examples","registry claim honesty","unsupported 25/25","external validation status: pending","not external validation"]:
    if phrase not in proof_text:
        raise SystemExit(f"proof doc missing required scope phrase: {phrase}")
PY

python3 - "$ROOT" <<'PY'
import re, sys
from pathlib import Path
root = Path(sys.argv[1])
files = [root/"docs/product/INDEX.md", root/"docs/verification/feature-docs-product-education-25.md", root/"START-HERE.md", root/"README.md", root/"docs/setup/README.md", root/"docs/setup/install.md", root/"docs/setup/first-run.md", root/"docs/setup/troubleshooting.md"]
errors=[]; heading_cache={}
def slugify(s):
    s=re.sub(r"`([^`]*)`", r"\1", s.strip().lower()); s=re.sub(r"<[^>]+>", "", s); s=re.sub(r"[^a-z0-9\s-]", "", s); return re.sub(r"\s+", "-", s).strip("-")
def headings(path):
    if path not in heading_cache:
        vals=set()
        for line in path.read_text(errors="replace").splitlines():
            m=re.match(r"^#{1,6}\s+(.+?)\s*$", line)
            if m: vals.add(slugify(m.group(1)))
        heading_cache[path]=vals
    return heading_cache[path]
def resolve(src,target):
    target=target.strip()
    if not target or target.startswith(("http://","https://","mailto:","#")):
        return None,None
    path_part,_,anchor=target.partition("#")
    dst=src if not path_part else (src.parent/path_part).resolve()
    return dst,anchor
for src in files:
    text=src.read_text(errors="replace")
    for m in re.finditer(r"(?<!!)\[[^\]]+\]\(([^)]+)\)", text):
        raw=m.group(1).split()[0]; dst,anchor=resolve(src,raw)
        if dst is None:
            continue
        if not dst.exists():
            errors.append(f"{src.relative_to(root)} broken markdown link: {raw}"); continue
        if anchor and slugify(anchor) not in headings(dst):
            errors.append(f"{src.relative_to(root)} broken markdown anchor: {raw}")
    for m in re.finditer(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]", text):
        raw=m.group(1); candidates=[]
        if raw.endswith(".md") or "/" in raw:
            candidates.append(root/raw)
        else:
            candidates.extend([root/f"{raw}.md", root/raw])
        if not any(c.exists() for c in candidates):
            errors.append(f"{src.relative_to(root)} broken wiki link: [[{raw}]]")
if errors:
    raise SystemExit("; ".join(errors))
PY

python3 - "$PRODUCT" "$PROOF" "$ROOT/README.md" "$ROOT/START-HERE.md" "$REPORT" "$FEATURES" <<'PY'
import re, sys
from pathlib import Path
errors=[]
for file_arg in sys.argv[1:]:
    path=Path(file_arg); lower=path.read_text(errors="replace").lower()
    for term in ["25/25 achieved","25/25 complete","full 25/25","production ready"]:
        if term in lower:
            errors.append(f"{path.name}: unsupported wording: {term}")
if Path(sys.argv[1]).read_text(errors="replace").lower().count("pending") < 3:
    errors.append("product education should repeat pending language for external/dogfood limits")
if Path(sys.argv[2]).read_text(errors="replace").lower().count("pending") < 3:
    errors.append("proof doc should repeat pending language for external/dogfood limits")
if errors:
    raise SystemExit("; ".join(errors))
PY

echo "feature-docs-product-education-proof: ok (strong local docs/product education proof)"
