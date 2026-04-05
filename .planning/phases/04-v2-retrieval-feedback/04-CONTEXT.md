# Phase 4: `v2` Retrieval Feedback - Context

**Gathered:** 2026-04-04
**Status:** Ready for planning
**Mode:** Auto-generated from `v2` branchable-belief completion and roadmap goals

<domain>
## Phase Boundary

This phase begins the first explicit feedback loop for retrieval quality.
The target is not adaptive ranking yet, but a durable substrate that records
which retrieval surfaces were used and which policy hooks fired.

</domain>

<decisions>
## Implementation Decisions

### Feedback shape
- The first retrieval feedback slice should be explicit and bounded.
- Feedback should reuse existing explain and policy vocabulary where possible.
- The hot path should record lightweight counters or events, not full transcripts.

### Learnable surface
- Retrieval feedback should be visible enough that later phases can tune ranking.
- Branchable beliefs should remain part of the feedback context so conflicts are
  learnable instead of flattened away.

</decisions>

<code_context>
## Existing Code Insights

- `crates/memd-server/src/inspection.rs` already emits explicit policy hooks.
- `crates/memd-server/src/working.rs` already exposes a deterministic working-memory policy state.
- `crates/memd-server/src/store.rs` already stores durable event records that can carry lightweight retrieval outcomes.

</code_context>

<specifics>
## Specific Ideas

- Record lightweight retrieval outcome events tied to item ids and policy hooks.
- Add compact counters to explain or policy responses.
- Keep feedback deterministic and bounded before any adaptive ranking.

</specifics>

---

*Phase: 04-v2-retrieval-feedback*
*Context gathered: 2026-04-04*
