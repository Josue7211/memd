---
phase: B5
name: CorrectionPropagation Bench
version: v5
status: planned
opened: 2026-04-22
depends_on: [V4]
axis: correction_retention
---

# Phase B5: CorrectionPropagation Bench

## Goal

Measure: user corrects a fact in session 1; does session N retrieval use the corrected value? Includes provenance check — the retrieved record must cite the correction turn.

## Why this phase exists

V4 C4 proves corrections store. B5 proves they propagate — that the correction lane actually changes retrieval output across sessions, not just storage.

## Deliver

1. **Scenario generator.** For each of 20 seeded facts, assert in session 1, correct in session 2, query in sessions {3,5,8}.
2. **Metrics.** `correction-propagation-rate`, `provenance-correctness` (did retrieved record cite correction turn), `stale-answer-rate`.
3. **Runner.** `memd bench substrate --suite correction-propagation`.
4. **Scorer.** Exact-match on canonical value + provenance turn-id match.
5. **Rollback variant.** User re-asserts original value in session 5; assert memd accepts, preserves chain.

## Pass Gate

- pre: no metric
- post: propagation-rate ≥ 0.85 at session 3, ≥ 0.80 at session 8; provenance-correctness ≥ 0.95
- evidence: results NDJSON + report
- regression budget: no regression on V4 C4 precision floor

## Product Win

"memd honors corrections" becomes measurable, not aspirational.

## Evidence

- scorer + report
- 10 runs stable ±0.03
- V7 uses these fixtures as its smoke tests

## Fail Conditions

- Propagation under 0.85: C4 detector under-capturing, or lookup not preferring corrected version.
- Provenance missing: C4 provenance fields not wired at lookup layer.

## Rollback

Bench-only.
