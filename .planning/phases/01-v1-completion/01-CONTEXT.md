# Phase 1: `v1` Completion - Context

**Gathered:** 2026-04-04
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase closes the remaining `v1` quality gaps in `memd` without drifting
into `v2` superhuman-memory work. The phase covers provenance drilldown,
practical repair actions, harder working-memory control, and the minimum
explicit source-trust / procedural / self-model surfaces required to call `v1`
 complete.

</domain>

<decisions>
## Implementation Decisions

### Provenance drilldown
- **D-01:** Provenance drilldown should start from existing explain and memory surfaces, not a new standalone subsystem.
- **D-02:** The first implementation target is summary-to-source traversal for memory items, including source path, source system, source agent, quality, and any linked raw artifact metadata already present in the store.
- **D-03:** Provenance should be exposed through API and CLI first; richer UI affordances can follow after the contract is stable.

### Repair actions
- **D-04:** Repair work should add concrete actions for stale, contested, and malformed memory state rather than only showing diagnostics.
- **D-05:** The first repair actions should map onto existing lifecycle semantics where possible: verify, expire, supersede, contest resolution, and safe metadata correction.
- **D-06:** Repair flows should prefer bounded, auditable mutations over broad automatic rewrites.

### Working-memory control
- **D-07:** Working memory should behave as a managed buffer with explicit admission, eviction, and rehydration semantics.
- **D-08:** Eviction reasons should become policy-driven, not just budget-driven; freshness, confidence, contradiction state, trust, and recent use should all matter.
- **D-09:** This phase should stop short of learned policy and keep the controller deterministic enough to test and explain.

### Source-trust, procedural, and self-model minimums
- **D-10:** `v1` only needs the minimum explicit surfaces necessary to stop these domains from being implicit or hand-wavy.
- **D-11:** Source-trust should begin as inspectable scoring and policy-visible metadata, not as a full branchable trust system.
- **D-12:** Procedural and self-model support in this phase should focus on clear schema and retrieval affordances for runbooks, operator preferences, and agent profile state already present in the repo.

### the agent's Discretion
- Exact endpoint names and CLI verbs for repair actions
- Internal scoring weights for working-memory eviction so long as they remain explainable and testable
- Whether provenance drilldown is best surfaced as extensions to explain responses or adjacent focused endpoints

</decisions>

<specifics>
## Specific Ideas

- The current product bar is to finish `v1` honestly before claiming `v2` work.
- `memd` should keep acting like the memory OS for future systems such as `braind`, not absorb the entire cognition stack.
- The recent working-memory controller work is valid, but it should be treated as `v1` completion work unless it crosses into learned policy.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Product and roadmap
- `ROADMAP.md` — strategic versioned roadmap and the distinction between `v1` completion and later `v2` work
- `.planning/PROJECT.md` — current project framing, active requirements, and architectural boundaries
- `.planning/REQUIREMENTS.md` — explicit `v1` and `v2` requirement split for this repo
- `.planning/ROADMAP.md` — execution-facing phase list for GSD

### Memory architecture and policy
- `docs/core/architecture.md` — memory layer model and control-plane ownership
- `docs/core/api.md` — current API surfaces including working memory and policy inspection
- `docs/policy/promotion-policy.md` — promotion gates, contradiction, freshness, and scope expectations
- `docs/policy/source-policy.md` — source-quality and source-material constraints
- `docs/policy/efficiency.md` — compact retrieval, bounded hot path, and budget rules

### Key implementation files
- `crates/memd-schema/src/lib.rs` — source of truth for memory, working-memory, policy, and repair-related schemas
- `crates/memd-server/src/main.rs` — current API handlers and working-memory orchestration
- `crates/memd-server/src/store.rs` — persistence, salience, decay, and consolidation behavior
- `crates/memd-client/src/main.rs` — CLI command surface and summary rendering
- `crates/memd-client/src/lib.rs` — client bindings to server routes

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/memd-server/src/main.rs`: existing explain, inbox, maintenance, working-memory, and policy endpoints provide the base extension points for this phase
- `crates/memd-server/src/store.rs`: existing salience, decay, rehearsal, and consolidation logic can inform repair and eviction policies
- `crates/memd-schema/src/lib.rs`: existing request/response types make it straightforward to extend contracts without inventing a parallel model layer
- `crates/memd-client/src/main.rs`: existing CLI command routing and summary rendering can absorb repair and provenance flows quickly

### Established Patterns
- API and CLI are tightly mirrored through shared schema contracts
- policy is encoded in explicit server logic rather than hidden prompting
- bounded outputs and explainability are first-class concerns
- external systems stay behind documented contracts instead of leaking into core memory semantics

### Integration Points
- provenance drilldown should connect to current explain and source-related paths
- repair actions should connect to lifecycle and maintenance endpoints
- smarter working-memory control should stay in the server-side working-memory path and surface through the existing client command

</code_context>

<deferred>
## Deferred Ideas

- Full learned retrieval policy belongs to `v2`, not this phase
- Branchable world models and competing-belief infrastructure belong to `v2`
- Deep collective/federated trust semantics belong to later versions

</deferred>

---

*Phase: 01-v1-completion*
*Context gathered: 2026-04-04*
