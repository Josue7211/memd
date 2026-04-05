# Phase 8: `v2` Reversible Compression and Rehydration - Context

**Gathered:** 2026-04-04
**Status:** Ready for planning
**Mode:** Auto-generated from remaining `v2` roadmap and requirements gaps

<domain>
## Phase Boundary

This phase adds the first explicit reversible-compression layer to `memd`.
The target is summary-first retrieval that can still rehydrate into bounded
artifact evidence when an operator or agent needs to go deeper.

</domain>

<decisions>
## Implementation Decisions

- Rehydration should build on existing explain artifact trails instead of inventing a parallel evidence model.
- The first slice should stay bounded and inspectable rather than trying to reconstruct arbitrary raw state.
- Summary-first retrieval must remain cheap on the hot path while exposing deeper evidence on demand.

</decisions>

<code_context>
## Existing Code Insights

- Explain already returns artifact trails, source memory, and recent events.
- Working memory already has admission, eviction, and rehydration queue state.
- Retrieval feedback and trust-weighted ranking are already available to bias what gets rehydrated first.

</code_context>

---

*Phase: 08-v2-reversible-compression-and-rehydration*
*Context gathered: 2026-04-04*
