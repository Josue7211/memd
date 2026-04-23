---
phase: B7
name: Correction → Canonical Promotion
version: v7
status: planned
opened: 2026-04-22
depends_on: [A7, V6 C6]
axis: correction_retention, trust_provenance
plan_spec: docs/phases/v7/phase-b7-plan.md
---

# Phase B7: Correction → Canonical Promotion

## Goal

When does a correction replace the prior canonical record? B7 ships the rule: on confidence ≥ 0.85 judge confirmation + prior canonical existing + no conflicting correction within window → promote. Prior canonical retires to `stage=retracted` with pointer to corrector.

## Why this phase exists

A7 verifies capture. C7 tests behavior change. B7 is the rule surface between them: without explicit promotion rules, corrections sit as candidates and never beat canonical in retrieval. B7 is also where we reuse V6 C6 promotion engine — one rule card, two sources (distillation + correction).

## Deliver

1. **Promotion rule extension.** V6 C6 `promotion.rs` learns a `source: correction` branch; thresholds distinct from distillation.
2. **Retraction lane.** `stage: retracted` added; retracted records still retrievable under explicit `--include-retracted`, never surface by default.
3. **Rule card.** `docs/contracts/correction-promotion.md` — thresholds, conflict handling, retraction semantics.
4. **Test scenarios.** 20 planted correction chains (A claims X → B corrects to Y → promotes, retracts A) with expected end-state.

## Pass Gate

- pre: corrections captured but never replace canonical
- post: 20 planted chains resolve correctly; retraction lane populated; rule card committed; V5 B5 CorrectionPropagation suite lifts ≥ 0.05
- evidence: promotion NDJSON, retraction log, B5 delta

## Product Win

"the last word wins" becomes an enforced contract, not a hope.

## Evidence

- rule card
- 20 chain test fixtures
- retraction log
- B5 suite delta

## Fail Conditions

- Any chain mis-resolves: rule thresholds wrong; do not tune fixtures.
- Retracted records leaking to default retrieval: hard fail.

## Non-Goals

- contradiction detection when no prior canonical (D7)
- UI for corrections (V8 B8)
