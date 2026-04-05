---
phase: 05-v2-trust-weighted-ranking
plan: 01
subsystem: ranking
tags: [memory, trust, ranking, working-memory, search]
requires:
  - phase: 04-v2-retrieval-feedback
    provides: explicit retrieval feedback and policy surfaces
provides:
  - trust-aware search ranking
  - trust-aware working-memory ranking
  - explicit trust demotion hooks in explain
affects:
  - future contradiction resolution
  - future adaptive ranking
requirements-completed: [SUPR-02]
completed: 2026-04-04
---

# Phase 5: `v2` Trust-Weighted Ranking Summary

`memd` now uses source trust as a live ranking input instead of leaving it as inspection-only metadata.

## Accomplishments
- Added per-item source trust lookup from the existing source aggregate surface.
- Demoted low-trust lanes and boosted strong-trust lanes in search and working-memory ranking.
- Added trust-floor and trust-boost hooks to explain output.
- Updated API docs so trust-aware ranking is part of the contract.

## Verification
- `cargo test -q`

## Next Phase Readiness

Phase 6 can now build contradiction resolution on top of branchable belief lanes and trust-aware ranking.

---
*Phase: 05-v2-trust-weighted-ranking*
*Completed: 2026-04-04*
