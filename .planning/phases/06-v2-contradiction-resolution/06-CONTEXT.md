# Phase 6: `v2` Contradiction Resolution - Context

**Gathered:** 2026-04-04
**Status:** Ready for planning
**Mode:** Auto-generated from trust-weighted ranking completion and roadmap goals

<domain>
## Phase Boundary

This phase adds an explicit operator-visible resolution layer over branchable
beliefs. The target is not automatic truth arbitration, but a durable way to
mark one branch preferred while preserving competing branches.

</domain>

<decisions>
## Implementation Decisions

- Contradiction resolution should preserve all branches.
- One branch may become preferred without deleting or merging away the others.
- Explain and inbox surfaces should reveal unresolved contradictions directly.

</decisions>

<code_context>
## Existing Code Insights

- Belief branches already exist on memory items.
- Explain already returns sibling branches.
- Repair and lifecycle routes already support contested and superseded state.

</code_context>

---

*Phase: 06-v2-contradiction-resolution*
*Context gathered: 2026-04-04*
