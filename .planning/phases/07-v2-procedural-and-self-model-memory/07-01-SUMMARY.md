---
phase: 07-v2-procedural-and-self-model-memory
plan: 01
subsystem: typed-memory
tags: [memory, procedural, self-model, retrieval]
requires:
  - phase: 06-v2-contradiction-resolution
    provides: explicit branch and resolution state for durable beliefs
provides:
  - first-class procedural memory kind
  - first-class self-model memory kind
  - retrieval and explain surfaces that understand both lanes
completed: 2026-04-04
---

# Phase 7: `v2` Procedural and Self Model Memory Summary

`memd` now treats procedural and self-model memory as explicit first-class lanes instead of implicit tag conventions.

## Accomplishments
- Added `procedural` and `self_model` memory kinds to the shared schema.
- Added matching retrieval intents so workflow recall and self-knowledge recall can be routed intentionally.
- Updated server routing, explain hooks, worker maintenance, dashboard controls, and CLI parsing/rendering to recognize the new lanes.
- Extended docs and tests so the new contract is explicit and verified.

## Verification
- `cargo test -q`

## Next Phase Readiness

Phase 8 can now focus on reversible compression and evidence rehydration so summary-first retrieval can zoom back into deeper artifacts without losing grounding.

---
*Phase: 07-v2-procedural-and-self-model-memory*
*Completed: 2026-04-04*
