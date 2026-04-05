# Phase 3: `v2` Branchable Beliefs - Context

**Gathered:** 2026-04-04
**Status:** Ready for planning
**Mode:** Auto-generated from `v2` foundations and roadmap goals

<domain>
## Phase Boundary

This phase adds the first explicit contradiction lane to `memd`.
Competing durable beliefs should remain separately queryable and inspectable
instead of collapsing into one canonical record.

</domain>

<decisions>
## Implementation Decisions

### Belief branching
- The first branchable-belief slice should be payload-level and API-visible.
- Belief branches should be explicit on memory items rather than inferred from
  repo branch or entity context.
- Duplicate control must separate records across belief branches so competing
  beliefs can coexist.

### Inspection
- Explain should show sibling belief branches so operators can inspect nearby
  competing records quickly.
- Search and inbox should allow branch filtering without changing the current
  route and intent model.

### the agent's Discretion
- Whether belief branches should also be surfaced in compact summaries
- How many sibling records explain should return by default
- Whether the first phase should include branch-specific promotion controls

</decisions>

<code_context>
## Existing Code Insights

- `crates/memd-server/src/keys.rs` controls duplicate and canonical separation.
- `crates/memd-server/src/inspection.rs` already builds artifact trails and
  policy hooks.
- `crates/memd-server/src/main.rs` already filters search and inbox by
  project/namespace and is the right place to add branch filters.

</code_context>

<specifics>
## Specific Ideas

- Add `belief_branch` to durable memory items and key request surfaces.
- Keep the first sibling-inspection path bounded and deterministic.
- Do not add new tables yet; keep the slice compatible with the existing
  SQLite payload model.

</specifics>

<deferred>
## Deferred Ideas

- Learned branch merge policies
- Multi-branch ranking and arbitration
- Shared or federated contradiction resolution

</deferred>

---

*Phase: 03-v2-branchable-beliefs*
*Context gathered: 2026-04-04*
