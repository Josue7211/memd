---
phase: 01-v1-completion
plan: 01
subsystem: api
tags: [memory, provenance, repair, working-memory, docs]
requires:
  - phase: 00-branches-version-history-and-contribution-rules
    provides: OSS-ready branch and history discipline
provides:
  - provenance drilldown from explain to source-memory trail
  - explicit repair actions for verify, expire, supersede, contest, and metadata correction
  - policy-visible working-memory admission, eviction, and rehydration reasons
affects:
  - phase 02 v2 foundations
tech-stack:
  added: []
  patterns:
    - focused server helper modules for explain/repair/working memory
    - shared client render helpers for explain, repair, source, profile, and working-memory summaries
    - deterministic policy reasons for memory control paths
key-files:
  created:
    - .planning/phases/01-v1-completion/01-01-SUMMARY.md
  modified:
    - crates/memd-server/src/inspection.rs
    - crates/memd-server/src/repair.rs
    - crates/memd-server/src/working.rs
    - crates/memd-server/src/main.rs
    - crates/memd-client/src/main.rs
    - crates/memd-client/src/render.rs
    - crates/memd-client/src/commands.rs
    - crates/memd-client/src/lib.rs
    - crates/memd-schema/src/lib.rs
    - docs/api.md
    - docs/architecture.md
    - docs/source-policy.md
    - docs/promotion-policy.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/PROJECT.md
    - .planning/REQUIREMENTS.md
key-decisions:
  - "Repair is bounded and auditable rather than automatic rewriting."
  - "Explain keeps provenance visible by drilling from compact memory to source-memory metadata."
  - "Working-memory eviction and rehydration reasons are deterministic and policy-visible."
patterns-established:
  - "Pattern 1: route handlers delegate explain and repair behavior to focused server modules."
  - "Pattern 2: CLI summaries come from shared render helpers instead of inline formatting."
  - "Pattern 3: policy factors are exposed as explicit reasons instead of hidden heuristics."
requirements-completed: [QUAL-04, QUAL-05, WORK-02, WORK-03, INTG-04]
completed: 2026-04-04
---

# Phase 1: `v1` Completion Summary

`v1` now has explicit provenance drilldown, bounded repair actions, and a
working-memory controller that reports policy reasons instead of hiding them.

## Performance

- **Duration:** about 1h 20m
- **Started:** 2026-04-04T20:10:00Z
- **Completed:** 2026-04-04T21:33:09Z
- **Tasks:** 3
- **Files modified:** 28

## Accomplishments
- Added `POST /memory/repair` and the matching CLI repair surface for verify, expire, supersede, contest, and metadata correction.
- Extended explain to surface source-memory drilldown and added a CLI summary path for provenance inspection.
- Hardened working memory so admission, eviction, and rehydration carry explicit policy reasons for freshness, trust, contradiction, and recent use.
- Updated architecture, source-policy, and promotion-policy docs so the `v1` finish line matches the actual behavior.

## Task Commits

Each task was completed inside the phase boundary commit:

1. **Task 1: Extract provenance and repair server helpers** - `0552a88` (`feat(v1): complete provenance, repair, and working memory`)
2. **Task 2: Split client rendering and command plumbing** - `0552a88` (`feat(v1): complete provenance, repair, and working memory`)
3. **Task 3: Harden working memory and document the v1 finish line** - `0552a88` (`feat(v1): complete provenance, repair, and working memory`)

**Plan metadata:** `0552a88` (`feat(v1): complete provenance, repair, and working memory`)

## Files Created/Modified
- `crates/memd-server/src/inspection.rs` - provenance explain helper and source drilldown
- `crates/memd-server/src/repair.rs` - bounded repair lifecycle helpers
- `crates/memd-server/src/working.rs` - deterministic working-memory policy helper and reasons
- `crates/memd-client/src/render.rs` - shared explain/repair/working-memory renderers
- `crates/memd-client/src/main.rs` - CLI dispatch for repair and explain summaries
- `docs/architecture.md` - working-memory controller contract
- `docs/source-policy.md` - bounded repair expectations for source material
- `docs/promotion-policy.md` - promotion and inbox policy visibility

## Decisions Made
- Kept public route names stable while moving logic into smaller helpers.
- Modeled repair as a bounded lifecycle action rather than a new memory subsystem.
- Kept working-memory policy deterministic so it can be inspected and tested.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

One compile break surfaced while plumbing repair and working-memory reasons. It was fixed by tightening imports and refactoring the working-memory ranking helper to accept optional entity metadata instead of a fake default state.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

`v1` is complete enough to move to `v2` foundations. The next phase can focus on explicit working-memory controller semantics, trust-weighted source memory, reversible compression, and the first learned retrieval-policy hooks.

---
*Phase: 01-v1-completion*
*Completed: 2026-04-04*
