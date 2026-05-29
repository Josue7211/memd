#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP="$(mktemp -d)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

log() { printf 'proof: %s\n' "$*"; }
fail() { printf 'proof: FAIL: %s\n' "$*" >&2; exit 1; }

python3 - "$ROOT" "$TMP/proof-summary.md" <<'PY'
import json
import re
import sys
from datetime import date, datetime
from pathlib import Path

root = Path(sys.argv[1])
summary_path = Path(sys.argv[2])

SEARCH_ROOTS = [
    root / "docs",
    root / "dogfood",
    root / "reliability",
    root / "artifacts",
    root / "logs",
    root / ".memd" / "logs",
]
KEYWORDS = ("dogfood", "reliability", "evidence clock", "dogfood clock", "wake-budget", "wake-cost")
LOG_SUFFIXES = {".log", ".jsonl", ".ndjson", ".json", ".txt", ".md"}
EXCLUDED_RELATIVE_PATHS = {
    "docs/verification/features.registry.json",
    "docs/verification/feature-coverage-report.md",
    "docs/verification/feature-dogfood-reliability-windows-25.md",
}
DATE_RE = re.compile(r"(20\d{2})[-_/](\d{2})[-_/](\d{2})")
ISO_RE = re.compile(r"20\d{2}-\d{2}-\d{2}(?:[T ][0-2]\d:[0-5]\d(?::[0-5]\d)?(?:Z|[+-]\d{2}:?\d{2})?)?")


def parse_date(value: str):
    if not value:
        return None
    value = value.strip().strip('"\'')
    match = ISO_RE.search(value)
    if not match:
        return None
    token = match.group(0).replace(" ", "T")
    try:
        if "T" in token:
            normalized = token.replace("Z", "+00:00")
            if re.search(r"[+-]\d{4}$", normalized):
                normalized = normalized[:-2] + ":" + normalized[-2:]
            return datetime.fromisoformat(normalized).date()
        return date.fromisoformat(token[:10])
    except ValueError:
        return None


def date_from_filename(path: Path):
    match = DATE_RE.search(str(path.relative_to(root)))
    if not match:
        return None
    try:
        return date(int(match.group(1)), int(match.group(2)), int(match.group(3)))
    except ValueError:
        return None


def read_text(path: Path):
    try:
        return path.read_text(errors="ignore")
    except Exception:
        return ""


def artifact_kind(path: Path, text: str):
    low = f"{path.name}\n{text[:2000]}".lower()
    if "dogfood" in low or "reliability" in low or "evidence clock" in low or "dogfood clock" in low:
        return "window_candidate"
    if "wake-budget" in low or "wake-cost" in low or path.suffix in {".ndjson", ".jsonl", ".log"}:
        return "log"
    return "ad_hoc_doc"


def line_timestamps(path: Path):
    found = []
    if path.suffix not in LOG_SUFFIXES:
        return found
    text = read_text(path)
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("{"):
            try:
                obj = json.loads(stripped)
            except Exception:
                obj = None
            if isinstance(obj, dict):
                for key in ("ts", "timestamp", "time", "date", "opened", "created_at"):
                    d = parse_date(str(obj.get(key, "")))
                    if d:
                        found.append(d)
        for token in ISO_RE.findall(line):
            d = parse_date(token)
            if d:
                found.append(d)
    return found

artifacts = []
for base in SEARCH_ROOTS:
    if not base.exists():
        continue
    for path in base.rglob("*"):
        if not path.is_file():
            continue
        rel = str(path.relative_to(root))
        if rel in EXCLUDED_RELATIVE_PATHS:
            continue
        if any(part in {".git", "target", "node_modules"} for part in path.parts):
            continue
        text = read_text(path) if path.suffix.lower() in LOG_SUFFIXES else ""
        low = f"{path.name}\n{text[:5000]}".lower()
        span_eligible = (
            "dogfood" in low
            or "evidence clock" in low
            or "dogfood clock" in low
            or "/dogfood" in rel.lower()
        )
        real_use_confirmed = (
            "real use" in low
            or "real user" in low
            or "real-session" in low
            or "real session" in low
            or "weekly evidence review" in low
        )
        disqualified_span = (
            "in lieu of real-session" in low
            or "dogfood deferred" in low
            or "next actions" in low
            or "next step" in low
            or "pending" in low
        )
        if not any(keyword in low for keyword in KEYWORDS):
            continue
        dates = set()
        fn_date = date_from_filename(path)
        if fn_date:
            dates.add(fn_date)
        for label in ("opened", "closed", "date", "day7_earliest"):
            for m in re.finditer(rf"(?im)^\s*{label}\s*:\s*(.+)$", text):
                d = parse_date(m.group(1))
                if d:
                    dates.add(d)
        timestamps = line_timestamps(path)
        dates.update(timestamps)
        opened_dates = []
        closed_dates = []
        for m in re.finditer(r"(?im)^\s*opened\s*:\s*(.+)$", text):
            d = parse_date(m.group(1))
            if d:
                opened_dates.append(d)
        for m in re.finditer(r"(?im)^\s*(closed|ended|reviewed)\s*:\s*(.+)$", text):
            d = parse_date(m.group(2))
            if d:
                closed_dates.append(d)
        artifacts.append({
            "path": str(path.relative_to(root)),
            "kind": artifact_kind(path, text),
            "dates": sorted(dates),
            "log_dates": sorted(timestamps),
            "opened_dates": sorted(opened_dates),
            "closed_dates": sorted(closed_dates),
            "span_eligible": span_eligible,
            "real_use_confirmed": real_use_confirmed,
            "disqualified_span": disqualified_span,
            "has_text": bool(text),
            "mentions_failure": bool(re.search(r"(?i)fail|failure|recover|repair|blocker|ready=false", text)),
        })

artifacts.sort(key=lambda a: (a["dates"][0] if a["dates"] else date.max, a["path"]))
dated = [a for a in artifacts if a["dates"]]
window_candidates = [a for a in dated if a["kind"] == "window_candidate"]
log_artifacts = [a for a in dated if a["kind"] == "log"]

calculated_windows = []
for a in window_candidates:
    if not a["span_eligible"] or not a["real_use_confirmed"] or a["disqualified_span"]:
        continue
    # Documentation window spans must have an explicit start and close/review/end;
    # an opened clock plus a future day-7 target is not treated as sustained.
    if a["opened_dates"] and a["closed_dates"]:
        start, end = min(a["opened_dates"]), max(a["closed_dates"])
        calculated_windows.append({
            "path": a["path"],
            "start": start,
            "end": end,
            "days": (end - start).days,
            "kind": a["kind"],
        })
for a in log_artifacts:
    # Logs can form a span from observed timestamps, because each timestamp is
    # an observed event rather than a planned milestone.
    ds = a["log_dates"]
    if len(ds) >= 2:
        start, end = min(ds), max(ds)
        calculated_windows.append({
            "path": a["path"],
            "start": start,
            "end": end,
            "days": (end - start).days,
            "kind": a["kind"],
        })

sustained = [w for w in calculated_windows if w["days"] >= 7]
status = "ad_hoc"
if sustained:
    status = "windowed_candidate"
if not dated:
    status = "none"

lines = []
lines.append("# Dogfood reliability windows local proof summary")
lines.append("")
lines.append(f"- scanned roots: {', '.join(str(p.relative_to(root)) if p.is_relative_to(root) else str(p) for p in SEARCH_ROOTS)}")
lines.append(f"- matching artifacts/logs found: {len(artifacts)}")
lines.append(f"- dated artifacts/logs found: {len(dated)}")
lines.append(f"- dated window candidates: {len(window_candidates)}")
lines.append(f"- dated log artifacts: {len(log_artifacts)}")
lines.append(f"- calculated window spans: {len(calculated_windows)}")
lines.append(f"- sustained spans >=7 days: {len(sustained)}")
lines.append(f"- honest dogfood conclusion: {status}")
lines.append("")
lines.append("## Dated artifacts inspected")
if dated:
    for a in dated[:200]:
        dates = ", ".join(d.isoformat() for d in a["dates"])
        failure = "; mentions failure/recovery" if a["mentions_failure"] else ""
        lines.append(f"- `{a['path']}` ({a['kind']}): {dates}{failure}")
else:
    lines.append("- none")
lines.append("")
lines.append("## Calculated window spans")
if calculated_windows:
    for w in calculated_windows:
        lines.append(f"- `{w['path']}`: {w['start']} to {w['end']} = {w['days']} days ({w['kind']})")
else:
    lines.append("- none calculable from the dated artifacts/logs in this checkout")
lines.append("")
lines.append("## Honest interpretation")
if sustained:
    lines.append("At least one local artifact contains a >=7-day dated span. This is still a local candidate until reviewed for actual continuous usage and failure/recovery completeness.")
elif dated:
    lines.append("Dated ad hoc dogfood/reliability evidence exists, but this proof found no explicit >=7-day closed reliability window with calculable duration. Do not claim sustained/continuous dogfood.")
else:
    lines.append("No dated dogfood/reliability artifacts or logs were found in this checkout. Do not claim dogfood reliability evidence.")
lines.append("")
lines.append("External validation: pending; this proof only inspects local repository/bundle artifacts available to the runner.")

summary_path.write_text("\n".join(lines) + "\n")
print("\n".join(lines))

if len(artifacts) == 0:
    print("proof: FAIL: no dogfood/reliability surfaces found to inspect", file=sys.stderr)
    sys.exit(1)
PY

[[ -s "$TMP/proof-summary.md" ]] || fail "proof summary was not written"
log "summary: $TMP/proof-summary.md"
log "feature-dogfood-reliability-windows-proof=pass"
