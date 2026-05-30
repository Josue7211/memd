# Feature 25 proof: dogfood reliability windows

Reference doc: local verification artifact for the feature registry; see [[ROADMAP]] for product priority context.

## Scope

Feature id: `feature.dogfood_reliability_windows`

This proof slice covers the evidence question behind dogfood reliability windows:

- discover local dogfood/reliability artifacts and logs in the checkout or local bundle paths;
- identify dated artifacts where possible;
- calculate window counts and durations when an artifact/log contains enough dates;
- write deterministic summary artifacts under `docs/verification/artifacts/`;
- distinguish ad hoc evidence from sustained, closed reliability windows with an explicit no-false-positive rule;
- avoid claiming continuous dogfood, production reliability, or external validation from local evidence alone.

## Executable proof

Run:

```bash
bash scripts/verify/feature-dogfood-reliability-windows-proof.sh
```

The script scans `docs/`, `dogfood/`, `reliability/`, `artifacts/`, `logs/`, and `.memd/logs/` when those paths exist. It looks for dogfood/reliability/window/log terms, extracts dates from filenames, front matter, JSON/NDJSON timestamp fields, and ISO-like timestamps, then reports:

- matching artifact/log count;
- dated artifact/log count;
- window evaluation count;
- sustained-window count;
- calculated spans where at least two dates are present;
- sustained spans of at least seven days, if any;
- explicit sustained-window presence/absence and absence reasons for every dated candidate.

The deterministic summaries are:

- `docs/verification/artifacts/dogfood-reliability-windows-local-summary.json`
- `docs/verification/artifacts/dogfood-reliability-windows-local-summary.md`

## Local result on this slice

Current local proof result: **strong local proof passed**. It inventoried local dogfood/reliability artifacts, extracted dated evidence, evaluated every dated candidate against the no-false-positive sustained-window rule, and found **zero sustained windows** in this checkout. Dated ad hoc evidence exists, so dogfood status remains `ad_hoc`.

Evidence currently found includes dated handoff/planning artifacts such as:

- `docs/handoff/2026-04-24-d4-dogfood-clock-started.md`, which starts a dogfood clock and names `2026-05-01` as the earliest day-7 review point;
- `docs/handoff/2026-05-06-dogfood-installer-m0-m4-ready.md`, which states the evidence clock was open and lists next actions for real users/devices;
- other dated dogfood/reliability planning or gap notes under `docs/backlog/` and `docs/handoff/`.

Those are useful local artifacts, but they do not by themselves close a sustained reliability window with audited real-use logs, failures, recovery measurements, and an end-of-window review. The honest classification remains **ad hoc local evidence**, not sustained/continuous dogfood.

## Coverage matrix

| Requirement | Local check | Status |
| --- | --- | --- |
| Artifact discovery | Scan repository and local bundle paths for dogfood/reliability/window/log evidence | covered locally |
| Dated evidence | Extract dates from filenames, front matter, JSON/NDJSON fields, and ISO timestamps | covered locally |
| Window counts | Count dated window candidates and calculated spans | covered locally |
| Duration calculation | Calculate start/end/day duration when at least two dates are present | covered when data exists |
| Failure/recovery evidence | Flag artifacts mentioning failure, recovery, repair, blocker, or `ready=false` | partial local signal only |
| Sustained dogfood | Require >=7 calculable days plus explicit real-use, close/review, failure/recovery, and no planning/future-only risk signals | absent in current local inventory |
| External validation | Independent users/machines and external audit replay | pending external |

## Honest limitations

This proof is intentionally conservative. It does not prove:

- actual daily use occurred for every date inside an open clock;
- uptime, regression rate, or recovery time unless those values are present in dated logs;
- that an open dogfood clock was completed or reviewed;
- that multiple independent users or devices participated;
- external validation or production readiness.

The proof status is `strong` because the local absence/presence check is deterministic and conservative, not because sustained dogfood exists. A 25/25 dogfood-readiness claim still needs dated, closed reliability windows with reviewed logs, explicit failures/recoveries, calculated durations, and preferably independent replay or auditor review.

## Freshness

Re-run this proof after changes to dogfood enrollment/status, installer/setup flows, memory storage, server/runtime reliability, release process, or whenever a new dogfood window starts/closes.
