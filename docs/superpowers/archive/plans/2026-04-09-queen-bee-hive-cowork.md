# Queen Bee Hive Cowork Implementation Plan

Date: 2026-04-09
Spec: `docs/superpowers/specs/2026-04-09-queen-bee-hive-cowork-design.md`
Status: Planned

## Goal

Turn `memd` hive into a harness-agnostic, capability-aware, queen-routed agent team runtime with:

- automatic isolated lanes for coding workerbees
- topic and scope arbitration
- same-branch and same-worktree hard fault detection
- compressed state packets instead of transcript-heavy cowork

## Non-Negotiable Invariants

1. Coding workerbees must never share a branch.
2. Coding workerbees must never share a worktree.
3. Same `repo_root` plus same `branch` across workerbees is a hard fault.
4. Same `worktree_root` across workerbees is a hard fault.
5. A bee cannot start code work without a safe lane.
6. A bee cannot be assigned work its harness cannot perform.
7. Topic and scope ownership must be persisted and inspectable.
8. Routine cowork updates should use compact packets and deltas, not replayed transcripts.

## Phase 1: Universal Team Identity

### Deliverables

- extend hive session heartbeat/schema with:
  - `harness`
  - `capability_class`
  - `assignment_suitability`
  - `transport_surfaces`
- store and return these fields in server hive session records
- show them in:
  - `memd hive`
  - `memd awareness`
  - `memd coordination`

### Files

- `crates/memd-schema/src/lib.rs`
- `crates/memd-client/src/main.rs`
- `crates/memd-client/src/render.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`

### Pass Criteria

- a live session publishes harness identity
- awareness renders harness/capability coverage
- full client/server tests remain green

### Failure Criteria

- any session still appears as a generic undifferentiated worker
- capability-aware routing cannot be expressed with current returned data

## Phase 2: Lane Identity

### Deliverables

- extend hive session heartbeat/schema with:
  - `lane_id`
  - `repo_root`
  - `worktree_root`
  - `branch`
  - `base_branch`
- persist lane fields in server store
- render lane identity in summary surfaces

### Files

- `crates/memd-schema/src/lib.rs`
- `crates/memd-client/src/main.rs`
- `crates/memd-client/src/render.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`

### Pass Criteria

- each coding bee has explicit lane identity
- same project sessions on different branches are distinguishable
- existing hive summary tests updated and passing

### Failure Criteria

- branch/worktree state remains inferred or absent
- lane identity is not visible in runtime summaries

## Phase 3: Hard Collision Detection

### Deliverables

- detect and mark hard faults for:
  - same branch
  - same worktree
  - same explicit scope
- add collision records to awareness/coordination summaries
- block unsafe cowork suggestions when a hard fault exists

### Files

- `crates/memd-client/src/main.rs`
- `crates/memd-client/src/render.rs`

### Pass Criteria

- same-branch workerbees show a hard collision
- same-worktree workerbees show a hard collision
- unsafe coordination actions are blocked or downgraded with explicit reason

### Failure Criteria

- collisions are only warnings
- same-branch coding bees can still proceed silently

## Phase 4: Automatic Lane Creation

### Deliverables

- add workerbee lane allocation flow
- coding bee startup auto-creates:
  - branch
  - worktree
- lane heartbeat publishes immediately after creation
- start fails hard if lane creation fails

### Surfaces

- `memd lane create`
- workerbee spawn path used by orchestrator / queen flow

### Files

- `crates/memd-client/src/main.rs`
- any helper module extracted if needed

### Pass Criteria

- creating a coding bee yields a unique branch and worktree
- lane metadata is persisted and visible
- start is blocked if branch/worktree allocation fails

### Failure Criteria

- manual branch setup still required for safe cowork
- a workerbee can start without isolated lane state

## Phase 5: Topic Arbitration

### Deliverables

- add persisted topic ownership
- require topic claim before execution begins
- detect soft conflict for duplicate topic work
- queen can:
  - allow
  - warn
  - reroute

### Surfaces

- `memd tasks`
- `memd coordination`
- `memd team route`

### Pass Criteria

- duplicate topic assignment is surfaced before file edits begin
- queen decisions are visible and persisted

### Failure Criteria

- topic overlap is only discovered after scope or file edits

## Phase 6: Scope Arbitration

### Deliverables

- strengthen persisted scope claims
- block conflicting file/path claims without explicit collaboration
- record handoff receipts for scope ownership transfers

### Pass Criteria

- overlapping scope claims become hard faults
- handoff receipts are persisted and inspectable

### Failure Criteria

- conflicting file claims can coexist without queen intervention

## Phase 7: Queen Enforcement

### Deliverables

- add queen decision receipts:
  - allow
  - warn
  - reroute
  - block
- enforce decisions in runtime actions, not only summaries
- stale lane retirement respects live ownership

### Pass Criteria

- blocked workers cannot continue unsafe flow
- rerouted workers receive new assignment/lane information
- stale lanes with no live ownership can be retired safely

### Failure Criteria

- queen decisions are advisory only

## Phase 8: Capability Routing

### Deliverables

- add routing logic that selects workerbees by suitability
- expose team capability coverage
- route:
  - planning to best planner
  - coding to strongest coder
  - review to strongest reviewer
  - lightweight work to lowest-cost suitable bee

### Surfaces

- `memd team summary`
- `memd team capabilities`
- `memd team route`

### Pass Criteria

- task routing is capability-fit, not merely round-robin or liveness-based
- unsuitable harnesses are excluded from incompatible assignments

### Failure Criteria

- routing still treats all bees as interchangeable

## Phase 9: Packetized Cowork Transport

### Deliverables

- define compact cowork packet model
- persist:
  - packets
  - delta ledger
  - handoff summaries
- use packets for routine bee updates instead of transcript replay

### Packet Shape

- `task`
- `topic_claim`
- `scope_claims`
- `status`
- `progress`
- `blockers`
- `handoff_summary`
- `delta_since_last_update`

### Pass Criteria

- cowork summaries can be reconstructed from persisted packets
- token-heavy transcript replay is no longer required for normal hive updates

### Failure Criteria

- bees still depend on full conversation replay for ordinary coordination

## Test Strategy

### Unit Tests

- schema round-trip for new hive fields
- lane creation helpers
- collision detectors
- topic arbitration
- scope arbitration
- routing decisions
- packet serialization

### Integration Tests

- same project, different branches: allowed
- same project, same branch: hard fault
- same project, same worktree: hard fault
- duplicate topic before edits: soft conflict
- duplicate scope/file: hard fault
- wrong harness assignment: block
- packetized update flow: visible in summaries

### Manual Verification

1. start queen bee
2. spawn two coding workerbees
3. verify different branch/worktree for each
4. assign same topic twice
5. verify queen warns/reroutes
6. force same file claim
7. verify queen blocks
8. inspect compact packet summaries

## Sequencing Recommendation

Recommended implementation order:

1. team identity
2. lane identity
3. hard collisions
4. automatic lane creation
5. topic arbitration
6. scope arbitration
7. queen enforcement
8. capability routing
9. packet transport

This order closes the correctness gap first, then upgrades the routing and token-efficiency model.

## Completion Bar

The work is complete only when:

- coding workerbees cannot share branch or worktree silently
- queen can block, reroute, and hand off in runtime flows
- mixed-harness teams route by capability fit
- cowork state is packetized and token-cheap
- client and server suites pass
