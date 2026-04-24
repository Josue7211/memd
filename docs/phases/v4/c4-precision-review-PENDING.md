---
phase: C4
task: C4.8
status: pending-evidence
opened: 2026-04-24
gate_threshold:
  precision: 0.85
  recall: 0.75
  fp_rate: 0.10
sample_set: shared/corrections/c4-sample-40.jsonl
---

# C4.8 — Precision review (PENDING)

This document is a placeholder. The C4.8 gate requires 7 calendar days of
live dogfood with `MEMD_C4_CORRECTION_DETECT=1` against real Claude Code
sessions. It cannot be satisfied in a single agent session.

## What needs to happen

1. Set `MEMD_C4_CORRECTION_DETECT=1` in the user's shell rc.
2. Use Claude Code normally for ≥ 7 days. The hook capture flow will
   write to `.memd/logs/corrections.ndjson` whenever a correction
   pattern triggers.
3. After 7 days:
   - Sample 40 captures from the NDJSON log (20 aligned + 20 adversarial
     "no, wait, actually" phrasings).
   - Save them as `shared/corrections/c4-sample-40.jsonl`.
   - Human-label each row: was this an actual correction (true positive)
     or a false positive?
4. Compute precision, recall, false-positive rate.
5. If all three thresholds clear, replace this file with
   `c4-precision-review-YYYY-MM-DD.md` containing the numbers and
   sample evidence.

## Gate thresholds (per phase-c4-plan revision 2026-04-22)

| Metric    | Threshold |
|-----------|-----------|
| Precision | ≥ 0.85    |
| Recall    | ≥ 0.75    |
| FP rate   | ≤ 0.10    |

A failed gate blocks C4 close. Threshold tuning is allowed; silencing
the gate is forbidden.

## Why this is deferred

C4.2-C4.7 + C4.10 ship the substrate. C4.8 is a measurement gate. It
requires real production traffic over real time — the only way to get
honest precision/recall numbers. Faking it with synthetic fixtures
would defeat the gate.

## Dependent tasks

- **C4.9** (`MEMD_C4_CORRECTION_DETECT` defaults to `1` + 10-STAR
  rescore) cannot proceed until this file is replaced with passing
  evidence.
