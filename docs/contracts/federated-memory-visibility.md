---
doc: federated-memory-visibility-contract
status: active
opened: 2026-04-22
closes_gap: 25
depends_on: [../verification/0.1.0-CONTRACT.md, ../theory/locks/2026-04-11-memd-theory-lock-v1.md]
implemented_in: [V4 (contract), V9 (enforcement), V10 (adversarial suite)]
---

# Federated Memory Visibility Contract

> memd is a second-brain substrate. When multiple agents, harnesses, projects,
> or users share it, the substrate must never silently leak, collide, or
> overwrite beyond scope. This doc defines the rules. Enforcement lands in V9;
> contract-level violations (spec that contradicts this doc) block any
> milestone at review time.

## Closes Gap-25 ("no live memory contract")

COVERAGE-MATRIX.md lists Gap-25 as unowned across V4–V10. This contract is
the zero-code add that closes it: the rules are now a review artifact, and
any code that breaks them is a bug against this file.

## Scope axes

memd models federation along three orthogonal axes:

| Axis | Values | Storage field |
|---|---|---|
| `MemoryScope`      | Local, Synced, Project, Global | `memory_items.scope` |
| `MemoryVisibility` | Private, Workspace, Public     | `memory_items.visibility` |
| `Agent identity`   | harness preset + agent id      | `memory_items.agent_id` + preset |

A memory's federation posture is `(scope, visibility, agent_id)`. The
contract below defines what reads and writes are permissible given that
tuple.

## Read rules

A query from agent A in workspace W, project P observes a memory M iff:

1. **Scope**: `M.scope = Local`  → A must equal M.agent_id
   `M.scope = Synced`           → A and M.agent_id share the same harness machine bundle
   `M.scope = Project`          → P must equal M.project_id
   `M.scope = Global`           → always observable (visibility still applies)

2. **Visibility**: `M.visibility = Private`    → A must equal M.agent_id
   `M.visibility = Workspace`  → W must equal M.workspace_id
   `M.visibility = Public`     → always observable (scope still applies)

3. **Both must pass.** A `(Global, Private)` memory is visible only to its
   author regardless of scope. A `(Local, Public)` memory is visible only to
   its author regardless of visibility. The stricter axis wins.

## Write rules

Writes carry `(scope, visibility, agent_id)` at insertion. Mutating any of
these fields post-insert is a supersede-write (new row, old row superseded)
not a field update. Audit trail preserved in `memory_item_history`.

- Agents may not write on behalf of another agent. `memory_items.agent_id`
  MUST equal the caller's agent identity.
- Cross-scope promotion (Local → Project, Workspace → Public) requires
  explicit promotion API call with human confirmation or `--promote-trusted`
  flag — never automatic.
- Supersede respects source trust hierarchy: canonical > promoted > candidate.
  A candidate-tier source cannot supersede a canonical-tier belief even if it
  targets the same claim.

## Correction rules

Corrections are a privileged kind with the strongest trust tier. But they
still obey scope/visibility:

- A correction in workspace W1 does not automatically propagate to W2.
- A human correction crosses agent boundaries within the same workspace
  (all agents in W observe the superseded state) but does not cross
  workspaces.
- Cross-workspace propagation requires explicit operator action.

## Collision rules

Two memories with identical content hash inserted by different agents
within visible scope dedupe to the earlier one with both agent_ids
attributed. Content hash uses normalized text (trim + collapse whitespace +
lowercase) then SHA256 → first 16 hex chars.

Identity-level collisions (same agent, same content, different time) retain
the earlier record; later insert updates `last_seen_at` only.

## Enforcement surface

Milestone-by-milestone rollout:

| Milestone | Enforcement level |
|---|---|
| V4 | Contract published. Review artifact only. Scope/visibility fields present in schema but enforcement is "trust the caller". |
| V5 | Read-path enforcement: queries respect scope × visibility, covered by tests. |
| V9 | Multi-user adversarial suite. Every (scope, visibility) × (same-agent, cross-agent, cross-workspace, cross-project) matrix tested for read AND write. |
| V10 | Self-improvement must not violate. Overnight evolution can only supersede within its own trust tier and scope. |

## Negative tests required at V9

The V9 gate proves, with failing-if-leak assertions, that:

1. Local/Private memory by agent A is invisible to agent B in same workspace.
2. Workspace/Workspace memory does not leak across workspaces.
3. Project scope respects project_id boundary.
4. Canonical belief cannot be superseded by candidate-tier source.
5. Correction from W1 does not show up in W2 retrieval without operator action.
6. Content-hash dedup across agents attributes both, discards neither.
7. Supersede write preserves audit trail (old row + new row both query-able
   via history API).

## Non-goals

- End-to-end encryption at rest. V10+ scope; current bar is logical isolation.
- ACLs beyond the three axes. If we need role-based access, that's a V11+
  design.
- Cross-organization sharing. Out of scope for 0.1.0.

## Violation response

If a V9 adversarial test fails:
- File a P0 backlog entry tagged `axis: shared_federated`.
- Block the V9 milestone close.
- Do not "fix" the test to pass — fix the substrate.

If this contract is contradicted by a phase spec:
- Reject the spec at review.
- Spec author updates either the spec or this contract (the latter requires
  theory-lock review).

## Changelog

- 2026-04-22 — initial contract, closes Gap-25. Enforcement phased V4..V10.
