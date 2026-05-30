#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SUMMARY_JSON="$ROOT/docs/verification/artifacts/dogfood-reliability-windows-local-summary.json"
SUMMARY_MD="$ROOT/docs/verification/artifacts/dogfood-reliability-windows-local-summary.md"

log() { printf 'proof: %s\n' "$*"; }
fail() { printf 'proof: FAIL: %s\n' "$*" >&2; exit 1; }

mkdir -p "$(dirname "$SUMMARY_JSON")"

python3 - "$ROOT" "$SUMMARY_JSON" "$SUMMARY_MD" <<'PYPROOF'
import json
import re
import sys
from datetime import date, datetime
from pathlib import Path

root = Path(sys.argv[1])
summary_json = Path(sys.argv[2])
summary_md = Path(sys.argv[3])

SEARCH_ROOTS = ["docs", "dogfood", "reliability", "artifacts", "logs", ".memd/logs"]
TEXT_SUFFIXES = {".log", ".jsonl", ".ndjson", ".json", ".txt", ".md", ".toml", ".yaml", ".yml"}
KEYWORDS = ("dogfood", "reliability", "evidence clock", "dogfood clock", "wake-budget", "wake-cost", "window")
EXCLUDED = {
    "docs/verification/features.registry.json",
    "docs/verification/feature-coverage-report.md",
    "docs/verification/FEATURES.md",
    "docs/verification/feature-dogfood-reliability-windows-25.md",
    "docs/verification/artifacts/dogfood-reliability-windows-local-summary.json",
    "docs/verification/artifacts/dogfood-reliability-windows-local-summary.md",
}
IGNORE_PARTS = {".git", "target", "node_modules", ".claude"}
DATE_RE = re.compile(r"(20\d{2})[-_/](\d{2})[-_/](\d{2})")
ISO_RE = re.compile(r"20\d{2}-\d{2}-\d{2}(?:[T ][0-2]\d:[0-5]\d(?::[0-5]\d)?(?:Z|[+-]\d{2}:?\d{2})?)?")
LABEL_RE = re.compile(r"(?im)^\s*(opened|started|start|closed|ended|reviewed|review|date|day7_earliest)\s*:\s*(.+)$")
REAL_USE_RE = re.compile(r"(?i)\b(real[- ]?(use|user|session|device|workflow)|weekly evidence review|actual use|dogfood(er|ing)? used)\b")
FAILURE_RE = re.compile(r"(?i)\b(fail(?:ed|ure)?|recover(?:y|ed)?|repair|blocker|incident|ready=false|regression|outage)\b")
PLANNING_RE = re.compile(r"(?i)\b(next actions?|next steps?|pending|planned|deferred|earliest|target|todo|proposal|in lieu of real[- ]session|not yet|needs? real users?)\b")
CLOSED_RE = re.compile(r"(?i)\b(closed|ended|reviewed|retrospective|postmortem|complete[d]?|window review)\b")
CONTINUITY_RE = re.compile(r"(?i)\b(daily|continuous|sustained|consecutive|uptime|window|duration|from .* to )\b")


def parse_date(value):
    if not value:
        return None
    value = str(value).strip().strip('"\'')
    m = ISO_RE.search(value)
    if not m:
        return None
    token = m.group(0).replace(" ", "T")
    try:
        if "T" in token:
            token = token.replace("Z", "+00:00")
            if re.search(r"[+-]\d{4}$", token):
                token = token[:-2] + ":" + token[-2:]
            return datetime.fromisoformat(token).date()
        return date.fromisoformat(token[:10])
    except ValueError:
        return None


def iso(d):
    return d.isoformat() if isinstance(d, date) else d


def read_text(path):
    if path.suffix.lower() not in TEXT_SUFFIXES:
        return ""
    try:
        data = path.read_bytes()
    except OSError:
        return ""
    if b"\0" in data[:4096]:
        return ""
    return data.decode("utf-8", errors="ignore")


def date_from_filename(rel):
    m = DATE_RE.search(rel)
    if not m:
        return None
    try:
        return date(int(m.group(1)), int(m.group(2)), int(m.group(3)))
    except ValueError:
        return None


def collect_json_dates(line):
    out = []
    s = line.strip()
    if not s.startswith("{"):
        return out
    try:
        obj = json.loads(s)
    except Exception:
        return out
    if not isinstance(obj, dict):
        return out
    for key in ("ts", "timestamp", "time", "date", "opened", "started", "closed", "ended", "reviewed", "created_at", "updated_at"):
        d = parse_date(obj.get(key))
        if d:
            out.append({"date": d, "source": f"json:{key}"})
    return out


def classify(rel, text):
    low = f"{rel}\n{text[:8000]}".lower()
    if "dogfood" in low or "evidence clock" in low or "dogfood clock" in low:
        return "dogfood_evidence"
    if "reliability" in low or "window" in low:
        return "reliability_evidence"
    if rel.endswith((".log", ".jsonl", ".ndjson")) or "wake-budget" in low or "wake-cost" in low:
        return "log_evidence"
    return "related_document"

artifacts = []
for root_name in SEARCH_ROOTS:
    base = root / root_name
    if not base.exists():
        continue
    for path in sorted(base.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(root).as_posix()
        if rel in EXCLUDED:
            continue
        if any(part in IGNORE_PARTS for part in path.relative_to(root).parts):
            continue
        text = read_text(path)
        haystack = f"{rel}\n{text[:12000]}".lower()
        if not any(k in haystack for k in KEYWORDS):
            continue
        observations = []
        fn_date = date_from_filename(rel)
        if fn_date:
            observations.append({"date": fn_date, "source": "filename"})
        for m in LABEL_RE.finditer(text):
            d = parse_date(m.group(2))
            if d:
                observations.append({"date": d, "source": f"label:{m.group(1).lower()}"})
        for line in text.splitlines():
            observations.extend(collect_json_dates(line))
            for token in ISO_RE.findall(line):
                d = parse_date(token)
                if d:
                    observations.append({"date": d, "source": "inline_iso"})
        seen = set(); dedup = []
        for o in observations:
            key = (o["date"], o["source"])
            if key not in seen:
                seen.add(key); dedup.append(o)
        dedup.sort(key=lambda o: (o["date"], o["source"]))
        dates = sorted({o["date"] for o in dedup})
        labels = {m.group(1).lower(): parse_date(m.group(2)) for m in LABEL_RE.finditer(text) if parse_date(m.group(2))}
        artifacts.append({
            "path": rel,
            "kind": classify(rel, text),
            "date_observations": [{"date": iso(o["date"]), "source": o["source"]} for o in dedup],
            "dates": [iso(d) for d in dates],
            "has_dated_evidence": bool(dates),
            "signals": {
                "real_use": bool(REAL_USE_RE.search(text)),
                "failure_or_recovery": bool(FAILURE_RE.search(text)),
                "closed_or_reviewed": bool(CLOSED_RE.search(text)),
                "continuity_language": bool(CONTINUITY_RE.search(text)),
                "planning_or_future_only_risk": bool(PLANNING_RE.search(text)),
            },
            "labels": {k: iso(v) for k, v in sorted(labels.items())},
        })

artifacts.sort(key=lambda a: (a["dates"][0] if a["dates"] else "9999-99-99", a["path"]))
dated = [a for a in artifacts if a["has_dated_evidence"]]

windows = []
for a in dated:
    ds = [date.fromisoformat(d) for d in a["dates"]]
    labels = {k: date.fromisoformat(v) for k, v in a["labels"].items()}
    sig = a["signals"]
    reasons = []
    if not sig["real_use"]:
        reasons.append("no explicit real-use/session/device signal")
    if not sig["closed_or_reviewed"]:
        reasons.append("no explicit close/end/review signal")
    if sig["planning_or_future_only_risk"]:
        reasons.append("planning/future-only language present")
    if not sig["failure_or_recovery"]:
        reasons.append("no failure/recovery/incident signal")
    start = labels.get("opened") or labels.get("started") or labels.get("start") or (min(ds) if len(ds) >= 2 else None)
    end = labels.get("closed") or labels.get("ended") or labels.get("reviewed") or labels.get("review") or (max(ds) if len(ds) >= 2 else None)
    days = (end - start).days if start and end and end >= start else None
    if days is None:
        reasons.append("insufficient start/end dates for duration")
    elif days < 7:
        reasons.append("duration under 7 days")
    sustained = bool(days is not None and days >= 7 and sig["real_use"] and sig["closed_or_reviewed"] and sig["failure_or_recovery"] and not sig["planning_or_future_only_risk"])
    windows.append({
        "path": a["path"],
        "start": iso(start) if start else None,
        "end": iso(end) if end else None,
        "duration_days": days,
        "sustained_window_present": sustained,
        "absence_reasons": [] if sustained else reasons,
    })

sustained = [w for w in windows if w["sustained_window_present"]]
result = {
    "feature_id": "feature.dogfood_reliability_windows",
    "proof_level": "strong_local",
    "generated_by": "scripts/verify/feature-dogfood-reliability-windows-proof.sh",
    "scanned_roots": SEARCH_ROOTS,
    "inventory": {
        "matching_artifact_count": len(artifacts),
        "dated_artifact_count": len(dated),
        "window_evaluation_count": len(windows),
        "sustained_window_count": len(sustained),
    },
    "dogfood_status_conclusion": "windowed" if sustained else ("ad_hoc" if dated else "none"),
    "sustained_window_present": bool(sustained),
    "sustained_window_absent": not bool(sustained),
    "no_false_positive_policy": "A sustained window requires >=7 calculable days plus explicit real-use, close/review, failure/recovery, and no planning/future-only risk signals in the same artifact.",
    "artifacts": artifacts,
    "window_evaluations": windows,
}
summary_json.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")

lines = [
    "# Dogfood reliability windows local proof summary",
    "",
    "Deterministic artifact generated by `bash scripts/verify/feature-dogfood-reliability-windows-proof.sh`.",
    "",
    "## Inventory",
]
for k, v in result["inventory"].items():
    lines.append(f"- {k}: {v}")
lines.extend([
    f"- sustained_window_present: {str(result['sustained_window_present']).lower()}",
    f"- sustained_window_absent: {str(result['sustained_window_absent']).lower()}",
    f"- dogfood_status_conclusion: {result['dogfood_status_conclusion']}",
    "",
    "## No-false-positive rule",
    result["no_false_positive_policy"],
    "",
    "## Dated evidence inventory",
])
if dated:
    for a in dated:
        lines.append(f"- `{a['path']}` ({a['kind']}): {', '.join(a['dates'])}")
else:
    lines.append("- none")
lines.extend(["", "## Sustained-window evaluations"])
if windows:
    for w in windows:
        verdict = "present" if w["sustained_window_present"] else "absent"
        span = f"{w['start']} to {w['end']} = {w['duration_days']} days" if w["duration_days"] is not None else "not calculable"
        reasons = "" if w["sustained_window_present"] else "; reasons: " + "; ".join(w["absence_reasons"])
        lines.append(f"- `{w['path']}`: {verdict}; {span}{reasons}")
else:
    lines.append("- none")
lines.extend(["", "## Honest conclusion"])
if sustained:
    lines.append("Local proof found at least one sustained-window candidate. This is not external validation.")
elif dated:
    lines.append("Dated local dogfood/reliability evidence exists, but no artifact passes the sustained-window rule. Dogfood status remains ad_hoc.")
else:
    lines.append("No dated dogfood/reliability evidence was found. Dogfood status is none.")
summary_md.write_text("\n".join(lines) + "\n")

print("\n".join(lines))
if len(artifacts) == 0:
    print("proof: FAIL: no dogfood/reliability surfaces found to inspect", file=sys.stderr)
    sys.exit(1)
if not summary_json.exists() or not summary_md.exists():
    print("proof: FAIL: deterministic summary artifacts missing", file=sys.stderr)
    sys.exit(1)
PYPROOF

[[ -s "$SUMMARY_JSON" ]] || fail "summary json was not written"
[[ -s "$SUMMARY_MD" ]] || fail "summary markdown was not written"
log "summary-json: ${SUMMARY_JSON#$ROOT/}"
log "summary-md: ${SUMMARY_MD#$ROOT/}"
log "feature-dogfood-reliability-windows-proof=pass"
