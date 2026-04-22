---
phase: G4
name: Session-Continuity Proof Harness
version: v4
status: planned
opened: 2026-04-22
depends_on: [A4, B4, C4, D4, E4, F4]
backlog_items: [continuity-unverified]
axis: session_continuity
---

# Phase G4: Session-Continuity Proof Harness

## Goal

Ship the automated proof harness that gates V4 completion. 3-session dogfood scenario, scripted, reproducible, run in CI nightly. Passes or V4 doesn't close.

## Why this phase exists

A4-F4 each ship a piece. G4 is the glue that proves they work together under a real session pattern. Without G4 the milestone gate is "seems fine"; with G4 it's "10/10 runs pass this canonical trace."

## Deliver

1. **Dogfood script.** 3-session scripted trace:
   - **Session 1:** 10 turns. Asserts 5 facts, sets 2 preferences, makes 1 correction.
   - Session end: compaction.
   - **Session 2:** 10 turns. Agent picks up task, touches 3 of the 5 facts. User makes 1 additional correction.
   - Session end: normal exit.
   - **Session 3:** 5 turns. Agent must retrieve correction from session 1, honor it in session 3.
2. **Assertions (automated).**
   - All 5 facts retrievable in session 2 via `memd lookup`.
   - Both preferences replayed in session 2 wake (D4 compiler output).
   - Correction from session 1 honored in session 3 (retrieval returns corrected value).
   - Wake token count in sessions 2 and 3 <2k.
   - No `continuity-breach.log` entries across all 3 sessions.
   - Hook trace shows correct fire order, no silent failures.
3. **10-STAR scorecard regeneration.** Script writes updated composite to `docs/verification/MEMD-10-STAR.md` with per-axis delta.
4. **CI integration.** Nightly run; failure blocks main merge.

## Pass Gate

- pre: no automated continuity proof
- post: harness passes 10/10 CI runs over 7 days; 10-STAR composite moves 2.15 → ≥4.0
- evidence: CI run logs + regenerated 10-STAR composite + session-trace artifacts
- regression budget: none — this is the gate

## Product Win

V4 ships with a number the team stands behind, measured daily. A dropped phase surfaces in CI, not in a user complaint.

## Evidence

- Dogfood script (committed)
- 10 CI run logs
- Regenerated `docs/verification/MEMD-10-STAR.md`
- 3-session trace artifacts (anonymized)

## Fail Conditions

- Harness flakes: stabilize (retry logic only for infra flakes, not for memd).
- Composite misses 4.0: do not close V4. File per-axis recovery phase, rerun.

## Rollback

N/A — this phase is a gate. If it fails, V4 stays open.
