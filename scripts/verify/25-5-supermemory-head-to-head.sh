#!/usr/bin/env bash
# Honest Supermemory head-to-head gate.
#
# Supermemory is a managed API. This gate refuses market-best claims unless a
# live API replay or explicit replay artifact exists for the same item-limited
# fixtures as the memd public scale report.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
REPORT="${REPORT:-$OUT_DIR/${RUN_DATE}-supermemory-head-to-head.json}"
MEMD_REPORT="${MEMD_REPORT:-}"
SUPERMEMORY_REPLAYS="${SUPERMEMORY_REPLAYS:-$ROOT/.memd/benchmarks/baselines/supermemory_replays.json}"
SUPERMEMORY_REQUEST="${SUPERMEMORY_REQUEST:-$ROOT/.memd/state/supermemory-replay-request.json}"
TRY_REPLAY="${TRY_REPLAY:-0}"
LIMIT="${LIMIT:-50}"
EPSILON="${EPSILON:-0.000001}"
BW_STATUS_JSON="${BW_STATUS_JSON:-}"
MEMD_BIN="${MEMD_BIN:-$ROOT/target/debug/memd}"
MEMD_ACCESS_ROUTE_JSON="${MEMD_ACCESS_ROUTE_JSON:-}"

mkdir -p "$OUT_DIR"

if [[ -z "$BW_STATUS_JSON" ]] && command -v bw >/dev/null 2>&1; then
  BW_STATUS_JSON="$(bw status 2>/dev/null || true)"
fi

if [[ -z "$MEMD_ACCESS_ROUTE_JSON" ]]; then
  if [[ -x "$MEMD_BIN" ]]; then
    MEMD_ACCESS_ROUTE_JSON="$("$MEMD_BIN" access route --output "$ROOT/.memd" --provider bitwarden --purpose supermemory-api-key --agent codex --json 2>/dev/null || true)"
  elif command -v memd >/dev/null 2>&1; then
    MEMD_ACCESS_ROUTE_JSON="$(memd access route --output "$ROOT/.memd" --provider bitwarden --purpose supermemory-api-key --agent codex --json 2>/dev/null || true)"
  fi
fi

if [[ -z "$MEMD_REPORT" ]]; then
  MEMD_REPORT="$(python3 - "$OUT_DIR" <<'PY'
import json
import pathlib
import re
import sys

out_dir = pathlib.Path(sys.argv[1])
candidates = []

def report_status(path):
    try:
        return json.loads(path.read_text(encoding="utf-8")).get("status")
    except Exception:
        return None

for path in out_dir.glob("*external-public-full.json"):
    if report_status(path) == "pass":
        candidates.append((2, 0, path.stat().st_mtime, path))

for path in out_dir.glob("*external-public-scale-*.json"):
    match = re.search(r"external-public-scale-(\d+)\.json$", path.name)
    if match:
        candidates.append((1, int(match.group(1)), path.stat().st_mtime, path))
if candidates:
    print(max(candidates)[3])
PY
)"
fi

if [[ "$TRY_REPLAY" == "1" && ! -e "$SUPERMEMORY_REPLAYS" && -n "${SUPERMEMORY_API_KEY:-}" ]]; then
  "$ROOT/scripts/bench-supermemory.py" \
    --benchmark longmemeval \
    --benchmark locomo \
    --benchmark membench \
    --benchmark convomem \
    --limit "$LIMIT"
fi

python3 - "$REPORT" "$MEMD_REPORT" "$SUPERMEMORY_REPLAYS" "$SUPERMEMORY_REQUEST" "$EPSILON" "$BW_STATUS_JSON" "$MEMD_ACCESS_ROUTE_JSON" <<'PY'
import json
import os
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
memd_report_path = pathlib.Path(sys.argv[2]) if sys.argv[2] else None
replays_path = pathlib.Path(sys.argv[3])
request_path = pathlib.Path(sys.argv[4])
epsilon = float(sys.argv[5])
bw_status_raw = sys.argv[6] if len(sys.argv) > 6 else ""
memd_access_route_raw = sys.argv[7] if len(sys.argv) > 7 else ""
datasets = ["longmemeval", "locomo", "membench", "convomem"]


def bitwarden_status():
    if not bw_status_raw.strip():
        return {"available": False, "status": "unknown"}
    try:
        parsed = json.loads(bw_status_raw)
    except Exception:
        return {"available": True, "status": "unparseable"}
    return {
        "available": True,
        "status": parsed.get("status") or "unknown",
        "serverUrl": parsed.get("serverUrl"),
        "userEmail": parsed.get("userEmail"),
    }


def memd_access_route():
    if not memd_access_route_raw.strip():
        return {"available": False, "routes": []}
    try:
        parsed = json.loads(memd_access_route_raw)
    except Exception:
        return {"available": False, "parse_error": True, "routes": []}
    routes = []
    for route in parsed.get("routes", []):
        routes.append({
            "provider": route.get("provider"),
            "status": route.get("status"),
            "scope": route.get("scope"),
            "secret_values_stored": bool(route.get("secret_values_stored")),
            "guidance": route.get("guidance"),
            "source": route.get("source"),
        })
    return {
        "available": True,
        "status": parsed.get("status"),
        "routes": routes,
        "notes": parsed.get("notes", []),
    }


def memd_access_route_guidance():
    route = memd_access_route()
    for item in route.get("routes", []):
        if item.get("provider") == "bitwarden" and item.get("guidance"):
            return item["guidance"]
    return None


def has_approved_supermemory_route():
    route = memd_access_route()
    if route.get("status") not in {"working", "partial"}:
        return False
    for item in route.get("routes", []):
        if item.get("scope") != "supermemory-api-key":
            continue
        if item.get("secret_values_stored"):
            continue
        if item.get("provider") in {"bitwarden", "agent-secrets", "macos-keychain", "process-env"}:
            if item.get("status") not in {"unavailable", "missing", "error"}:
                return True
    return False


def access_route_hint():
    bw = bitwarden_status()
    memd_guidance = memd_access_route_guidance()
    if os.environ.get("SUPERMEMORY_API_KEY"):
        return "Approved Supermemory credential is present for this process; run with TRY_REPLAY=1 to create same-fixture replay artifacts."
    if memd_guidance:
        return (
            f"memd access route says: {memd_guidance} "
            "After the approved route is available, provide the credential only to this process "
            "or provide an explicit replay artifact. Do not store the secret in memd."
        )
    if bw.get("available") and bw.get("status") == "locked":
        return (
            "Bitwarden is configured but locked. Ask the user to unlock Bitwarden, "
            "then use the approved access route for this process or provide an explicit replay artifact. "
            "Do not store the secret in memd."
        )
    if bw.get("available") and bw.get("status") == "unlocked":
        return (
            "Bitwarden is unlocked. Retrieve the approved Supermemory API-key route into an ephemeral "
            "process-local credential, run TRY_REPLAY=1, and do not persist the secret."
        )
    return (
        "Use the approved Supermemory credential route or provide SUPERMEMORY_REPLAYS. If the key lives in a password manager, "
        "ask the user for the approved access route."
    )


def write_request(**extra):
    request_path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "memd.supermemory-replay-request.v1",
        "status": "needs_replay_artifact_or_process_credential",
        "competitor": "supermemory",
        "datasets": datasets,
        "same_fixture_contract": {
            "memd_report": str(memd_report_path) if memd_report_path else None,
            "required_limit_scope": "items",
            "required_limit": None,
            "replay_path": str(replays_path),
        },
        "approved_routes": {
            "process_env": "SUPERMEMORY_API_KEY",
            "bitwarden": "memd access route --output .memd --provider bitwarden --purpose supermemory-api-key --agent codex",
            "run_replay": "TRY_REPLAY=1 scripts/verify/25-5-supermemory-head-to-head.sh",
            "provide_artifact": "SUPERMEMORY_REPLAYS=/path/to/supermemory-replays scripts/verify/25-5-supermemory-head-to-head.sh",
        },
        "privacy_contract": [
            "Do not store SUPERMEMORY_API_KEY in memd artifacts.",
            "Use credentials only as process-local environment for replay.",
            "Replay artifacts must cover longmemeval, locomo, membench, and convomem with matching item limit.",
            "Artifacts may be aggregate JSON or <dir>/<dataset>/latest/summary.json directories.",
        ],
        "access_route_hint": access_route_hint(),
        "bitwarden": bitwarden_status(),
        "memd_access_route": memd_access_route(),
        **extra,
    }
    if memd_report_path and memd_report_path.exists():
        try:
            report = json.loads(memd_report_path.read_text(encoding="utf-8"))
            payload["same_fixture_contract"]["required_limit"] = report.get("limit")
        except Exception:
            pass
    request_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return str(request_path)


def write_and_exit(status, exit_code, **extra):
    request = None
    if status == "blocked":
        request = write_request(**extra)
    payload = {
        "suite": "25_5_supermemory_head_to_head",
        "status": status,
        "competitor": "supermemory",
        "memd_report": str(memd_report_path) if memd_report_path else None,
        "competitor_replays": str(replays_path),
        "credential_env_present": bool(os.environ.get("SUPERMEMORY_API_KEY")),
        "bitwarden": bitwarden_status(),
        "memd_access_route": memd_access_route(),
        "supermemory_request_path": request,
        "access_route_hint": access_route_hint(),
        **extra,
    }
    report_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"25_5_supermemory_head_to_head {status} report={report_path}")
    raise SystemExit(exit_code)


def load_replays(path):
    if path.is_file():
        return json.loads(path.read_text(encoding="utf-8"))
    if not path.is_dir():
        return None
    replays = {}
    for dataset in datasets:
        candidates = [
            path / dataset / "latest" / "summary.json",
            path / "supermemory-replays" / dataset / "latest" / "summary.json",
        ]
        for candidate in candidates:
            if candidate.exists():
                row = json.loads(candidate.read_text(encoding="utf-8"))
                row.setdefault("source", str(candidate))
                replays[dataset] = row
                break
    return replays


if not memd_report_path or not memd_report_path.exists():
    write_and_exit("blocked", 2, reason="missing memd external public scale report")

memd_report = json.loads(memd_report_path.read_text(encoding="utf-8"))
if memd_report.get("status") != "pass":
    write_and_exit("blocked", 2, reason="memd report is not passing")

replays = load_replays(replays_path)
if replays is None:
    reason = "missing Supermemory live replay artifacts"
    required = "use the approved Supermemory access route with TRY_REPLAY=1 or provide SUPERMEMORY_REPLAYS"
    missing_requirements = ["supermemory_same_fixture_replay_artifact"]
    if not os.environ.get("SUPERMEMORY_API_KEY") and not has_approved_supermemory_route():
        reason = "missing approved Supermemory credential and replay artifacts"
        missing_requirements.insert(0, "approved_supermemory_access_route_or_process_credential")
    elif not os.environ.get("SUPERMEMORY_API_KEY"):
        reason = "missing Supermemory replay artifacts; approved route exists but no process credential was resolved"
    write_and_exit(
        "blocked",
        2,
        reason=reason,
        required=required,
        missing_requirements=missing_requirements,
    )

memd_rows = {row.get("dataset"): row for row in memd_report.get("rows", [])}
memd_limit = memd_report.get("limit")
missing = []
rows = []
failures = []
for dataset in datasets:
    memd_row = memd_rows.get(dataset)
    competitor_row = replays.get(dataset)
    if memd_row is None:
        missing.append({"dataset": dataset, "missing": "memd_row"})
        continue
    if competitor_row is None:
        missing.append({"dataset": dataset, "missing": "supermemory_replay"})
        continue
    if competitor_row.get("status") != "replayed":
        missing.append({"dataset": dataset, "missing": "replayed_status"})
        continue
    if competitor_row.get("limit_scope") != "items" or competitor_row.get("limit") != memd_limit:
        missing.append(
            {
                "dataset": dataset,
                "missing": "matching_item_limit",
                "memd_limit": memd_limit,
                "competitor_limit": competitor_row.get("limit"),
                "competitor_limit_scope": competitor_row.get("limit_scope"),
            }
        )
        continue
    competitor_score = competitor_row.get("accuracy")
    memd_score = memd_row.get("accuracy")
    if competitor_score is None or memd_score is None:
        missing.append({"dataset": dataset, "missing": "accuracy"})
        continue
    competitor_score = float(competitor_score)
    memd_score = float(memd_score)
    delta = memd_score - competitor_score
    row = {
        "dataset": dataset,
        "memd_score": memd_score,
        "competitor_score": competitor_score,
        "delta": delta,
        "competitor_limit": competitor_row.get("limit"),
        "competitor_limit_scope": competitor_row.get("limit_scope"),
        "competitor_command": competitor_row.get("command"),
        "competitor_artifact_path": competitor_row.get("artifact_path"),
    }
    rows.append(row)
    if delta + epsilon < 0.0:
        failures.append(row)

if missing:
    write_and_exit("blocked", 2, reason="incomplete local same-fixture Supermemory coverage", missing=missing, rows=rows)
if failures:
    write_and_exit("fail", 1, reason="memd below Supermemory on at least one same-fixture replay", failed=failures, rows=rows)
write_and_exit("pass", 0, reason="memd meets or exceeds Supermemory same-fixture replay on every covered dataset", rows=rows)
PY
