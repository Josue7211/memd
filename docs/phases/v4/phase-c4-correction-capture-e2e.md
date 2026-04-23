---
phase: C4
name: Correction Capture E2E
version: v4
status: planned
opened: 2026-04-22
depends_on: [B4]
backlog_items: [corrections-not-e2e-verified]
axis: correction_retention
---

# Phase C4: Correction Capture E2E

## Goal

User says "no, X is Y" in a session — memd captures the correction, stamps provenance (turn, user, timestamp), stages it for canonical promotion per the correction lane rules. Today the capture path exists but is not end-to-end verified; V5's CorrectionPropagation bench needs C4 first.

## Why this phase exists

memd has `memd hook capture --summary` and a correction lane, but there is no automated test that (a) the user's correction is recognized as a correction, (b) it's stored with provenance, (c) it's retrievable. Correction-retention axis stuck at 2/10 because the feature is unverified.

## Deliver

1. **Correction detector.** Deterministic rule: user turn contains negation + prior-claim reference ("no, X is Y" / "wait, actually Y" / "I meant Y") → flag as correction candidate. LLM-judge confirms (cached per H3 pattern).
2. **Capture path.** Flagged turns go through `memd hook capture --summary --kind correction` with `--source-turn <ref>` pointer.
3. **Storage contract.** Correction records carry: `corrects_id: <prior_record>`, `source_turn`, `captured_by`, `confidence`. Schema change documented.
4. **E2E test.** Scripted session: assert fact X in turn 1, correct to Y in turn 5. Test asserts:
   - correction detected
   - record stored with correct provenance fields
   - `memd lookup --kind correction` returns it
5. **Turn-level telemetry.** Every correction detection + capture logged to `.memd/logs/corrections.ndjson`.

## Pass Gate

- pre: no E2E correction test; unknown capture rate in real sessions
- post: E2E test passes; 7-day dogfood shows ≥10 corrections captured with clean provenance
- evidence: test output + 7-day `corrections.ndjson` + sample human-review of 20 captures (precision ≥ 0.85)
- regression budget: no increase in false-positive capture rate (baseline measured during pre)

## Product Win

User feels heard. A correction isn't just acknowledged in-session — memd remembered that it was a correction, knows what it corrected, can show the user later. Foundation for V7.

## Evidence

- E2E test + output
- `.memd/logs/corrections.ndjson` from 7-day dogfood
- Human-review precision sample (20 captures graded)

## Fail Conditions

- LLM-judge cost blown (>$5 per 7-day dogfood): fall back to deterministic rule only, LLM-judge on opt-in.
- Precision <0.85: tighten detector, rerun.

## Rollback

Correction detection behind `MEMD_C4_CORRECTION_DETECT=1`. Manual `memd hook capture --kind correction` path always works.
