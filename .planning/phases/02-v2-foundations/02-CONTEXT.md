# Phase 2: `v2` Foundations - Context

**Gathered:** 2026-04-04
**Status:** Ready for planning
**Mode:** Auto-generated from `v1` completion and roadmap goals

<domain>
## Phase Boundary

This phase begins the move from brain-inspired memory toward machine-advantaged
memory. The near-term target is to make trust, compression, and retrieval policy
more explicit without breaking the stable `v1` contracts that already ship.

</domain>

<decisions>
## Implementation Decisions

### Trust and policy
- The policy snapshot should expose a default source-trust floor, because agent
  profiles already carry per-agent trust floors and the server needs a policy
  baseline to reason from.
- Working-memory policy should stay deterministic and explainable before any
  learned policy enters the picture.

### Compression and retrieval
- Reversible compression should preserve a compact hot path while keeping raw
  evidence reachable behind the summary.
- Retrieval policy hooks should be visible and measurable before they become
  adaptive.

### the agent's Discretion
- Whether the trust floor is surfaced only in the policy snapshot or also in
  working-memory responses
- How to expose reversible compression without turning `memd` into transcript
  storage
- Whether the first retrieval-feedback hook is an event, a metric, or a policy
  trace

</decisions>

<code_context>
## Existing Code Insights

- `crates/memd-server/src/working.rs` already has deterministic ranking and a
  policy snapshot.
- `crates/memd-schema/src/lib.rs` already has agent-profile trust floors and a
  memory policy response contract.
- `crates/memd-server/src/inspection.rs` already exposes explain and source
  drilldown paths.

</code_context>

<specifics>
## Specific Ideas

- Start with explicit policy surfaces rather than learned heuristics.
- Keep raw evidence available behind the summary so compression stays reversible.
- Use the current phase to make trust and retrieval policy more inspectable,
  not to solve the whole cognition stack.

</specifics>

<deferred>
## Deferred Ideas

- Branchable world models
- Fully learned retrieval policy
- Cross-agent trust federation

</deferred>

---

*Phase: 02-v2-foundations*
*Context gathered: 2026-04-04*
