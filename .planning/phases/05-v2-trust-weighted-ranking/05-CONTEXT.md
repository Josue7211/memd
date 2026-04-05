# Phase 5: `v2` Trust-Weighted Ranking - Context

**Gathered:** 2026-04-04
**Status:** Ready for planning
**Mode:** Auto-generated from retrieval-feedback completion and roadmap goals

<domain>
## Phase Boundary

This phase makes source-trust floors operational in ranking.
The goal is deterministic demotion of weak source lanes, not learned ranking.

</domain>

<decisions>
## Implementation Decisions

- Trust-aware ranking should affect ordering, not hard-delete low-trust memories.
- Contested and synthetic/weak lanes should lose score predictably.
- Explain and working-memory reasons should surface trust demotion explicitly.

</decisions>

<code_context>
## Existing Code Insights

- `crates/memd-server/src/store.rs` already computes source trust aggregates.
- `crates/memd-server/src/main.rs` and `crates/memd-server/src/working.rs` own ranking.
- `crates/memd-server/src/inspection.rs` already exposes policy hooks and retrieval feedback.

</code_context>

---

*Phase: 05-v2-trust-weighted-ranking*
*Context gathered: 2026-04-04*
