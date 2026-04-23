---
phase: G7
name: Rollback on Bad Correction + V7 Completion Gate
version: v7
status: planned
opened: 2026-04-22
depends_on: [A7, B7, C7, D7, E7, F7]
axis: correction_retention, trust_provenance, V7 completion gate
plan_spec: docs/phases/v7/phase-g7-plan.md
---

# Phase G7: Rollback on Bad Correction + V7 Completion Gate

## Goal

Two jobs: (1) rollback path — user can undo a correction, memd restores prior canonical with provenance-preserving forward pointer. (2) V7 gate — 100% V5 B5 propagation, 3-session dogfood clean, rollback tested, 10-STAR composite ≥ 7.8.

## Why this phase exists

Corrections can be wrong. Without rollback, memd becomes an unreliable memory that can be poisoned by a bad user utterance. G7 ships the undo + gates the V7 milestone.

## Deliver

1. **Rollback CLI.** `memd correction rollback <correction-id> [--reason "..."]`. Restores prior canonical to `stage=canonical`, demotes corrector to `stage=retracted-by-rollback`, appends reverse-link to chain.
2. **Rollback scenarios.** 10 test cases: rollback recent, rollback chain (A→B→rollback-B), rollback with intervening correction.
3. **V7 dogfood harness.** 3 sessions × 5 corrections + 1 rollback. Must run clean in CI.
4. **V7 aggregator.** Regenerates `docs/verification/V7_CORRECTION_AUDIT.md` with A7 miss-rate, B7 promotion count, C7 respected-rate, D7 contradiction count, E7 chain completeness, F7 surface snapshot, G7 rollback success.
5. **10-STAR axis writer.** Bumps correction_retention 7→9, trust_provenance 7→8, session_continuity 6→7.
6. **MILESTONE-v7 close.** Fill audit doc; flip ROADMAP.

## Pass Gate

- pre: no rollback; V7 axes unmoved
- post canonical:
  - V5 B5 CorrectionPropagation ≥ 1.00 (100%)
  - C7 respected-rate@session=5 ≥ 0.90 (held from C7)
  - 10 rollback scenarios green
  - D7 contradiction resolution tested end-to-end
  - 10-STAR composite ≥ 7.8
- evidence: V7 audit doc, CI harness 10/10 runs, 10-STAR regen

## Product Win

"memd can be corrected, memd can be un-corrected, all with provenance" — trust floor for V8.

## Evidence

- V7 audit doc
- rollback tests (10)
- CI 10/10
- 10-STAR diff

## Fail Conditions

- Any rollback test fails: correction chain bug; root-cause.
- B5 suite <1.00: promotion regression.
- Composite <7.8: axis writer refuses to publish; root-cause upstream.

## Non-Goals

- multi-user rollback coordination (V9)
- auto-rollback from user ignore signal (V10 B10)
