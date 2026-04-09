# Queen Bee Hive Cowork Design

Date: 2026-04-09
Status: Proposed
Owner: memd

## Why

`memd` cowork is currently incomplete. Multiple live sessions can share the same project hive without a first-class execution lane contract. That allows unsafe states:

- workerbees sharing a branch
- workerbees sharing a worktree
- duplicate topic work starting without arbitration
- overlapping file edits starting before the hive intervenes

That is not a coordination detail. It is a correctness failure. A 10-star hive cannot rely on humans to remember branch hygiene.

The product model must change from:

- "who is alive in the project hive"

to:

- "who owns which execution lane, topic, and scopes right now"

## Product Thesis

`memd` hive cowork should feel like a live classroom:

- the `queen bee` orchestrates
- each `workerbee` gets its own lane
- every bee continuously broadcasts what it is doing
- the hive intervenes before overlap turns into damage

The queen bee is not only a viewer. The queen is the runtime authority for safe cowork.

## Core Principles

1. Isolation first

- Every workerbee must have its own branch.
- Every workerbee should have its own worktree.
- Shared branch or shared worktree between workerbees is unsafe by default.

2. Intent must be explicit

- A bee must publish what it is doing before it starts doing it.
- Topic ownership and scope ownership are separate signals and both matter.

3. The queen bee arbitrates

- The queen can approve, deny, reroute, or retire work.
- Unsafe overlap is blocked, not merely logged.

4. Hive truth is runtime truth

- Coordination state must be persisted and inspectable.
- The surfaced runtime model must match what the queen uses for decisions.

5. Seamless by default

- Bees should not need a manual "remember to branch first" step.
- If the hive can safely allocate an isolated lane, it should do so automatically.

## Roles

### Queen Bee

The queen bee is the orchestrator session. The queen:

- creates workerbee lanes
- assigns topics
- approves or denies potential overlap
- requests handoff
- retires stale lanes
- maintains the ownership table

### Workerbee

A workerbee is an execution session with a dedicated lane. A workerbee:

- owns one branch
- should own one worktree
- publishes topic and scope claims
- reports progress and blockers
- cannot override queen arbitration

### Hive

The hive is the coordination plane that stores:

- live lane identity
- ownership
- conflict state
- handoff receipts
- retirement state

## Hard Invariants

These are product invariants, not best practices.

1. Workerbees never share a branch.
2. Workerbees never share a worktree.
3. Same `repo_root` plus same `branch` across workerbees is a hard fault.
4. Same `worktree_root` across workerbees is a harder fault.
5. Same topic without queen approval is a soft conflict.
6. Same file or scope without explicit collaboration is a hard conflict.
7. A bee without a safe lane cannot start work.

## Runtime Contract

Each live hive session must publish:

- `lane_id`
- `role`
- `repo_root`
- `worktree_root`
- `branch`
- `base_branch`
- `topic_claim`
- `scope_claims`
- `status`
- `progress`
- `blockers`
- `updated_at`

### Lane Identity

`lane_id` is the durable identity for a workerbee execution lane.

It must be stable enough to:

- correlate heartbeats
- track ownership
- detect retirement
- anchor handoff receipts

It should be derived from:

- session identity
- branch
- worktree root

## Conflict Model

### Soft Conflict

A soft conflict means work may overlap soon, but has not yet crossed a hard safety boundary.

Examples:

- same topic, different candidate files
- same task goal, no concrete scope claim yet

Queen action:

- warn both bees
- require topic acknowledgement
- optionally reroute one bee

### Hard Conflict

A hard conflict means real collision risk already exists.

Examples:

- same branch
- same worktree
- same file
- same explicit scope

Queen action:

- block immediately
- deny new claim
- reroute or handoff

## Auto Lane Allocation

When the queen starts a workerbee, `memd` must allocate a safe lane automatically.

### Required behavior

1. Create a new branch for the workerbee.
2. Create a new worktree for the workerbee.
3. Publish the lane heartbeat.
4. Attach the assigned topic to the lane.
5. Refuse start if any of the above fails.

### Why worktree-first

Branch-only isolation is not enough in a shared checkout. Worktree-first workerbees give:

- better safety
- better continuity
- cleaner cleanup
- easier visibility

## Ownership Model

The hive must track two distinct ownership tables.

### Topic Ownership

Represents intent at the task level.

Use cases:

- stop duplicate work early
- show who already owns the idea
- support reassignment before code edits begin

### Scope Ownership

Represents actual file or path claims.

Use cases:

- block concrete overlap
- protect active edits
- support handoff of touched code

Both are required for a 10-star system.

## Handoff Protocol

A handoff is an explicit transfer of ownership, not an inferred social convention.

The queen must be able to:

- request handoff
- force handoff when a stale bee still owns a topic or scope
- record who yielded and who received ownership

A handoff receipt must persist:

- from lane
- to lane
- topic
- scopes
- reason
- timestamp

## Stale Lane Policy

The hive must distinguish:

- current active lane
- active workerbee lane
- stale lane
- dead lane
- retired lane

Retirement rules:

- stale lanes can be auto-retired if they no longer own active work
- stale lanes with owned scopes require queen arbitration before retirement

## Surfaces

### `memd hive`

Shows:

- queen lane
- workerbee lanes
- lane health
- hard faults
- soft conflicts

### `memd coordination`

Shows:

- topic ownership
- scope ownership
- overlap table
- arbitration decisions

### `memd tasks`

Shows:

- classroom board
- assigned work
- blocked work
- help requests
- review requests

### `memd session`

Shows and controls:

- rebind
- reconcile
- retire
- handoff

### `memd lane`

New command surface:

- `create`
- `assign`
- `handoff`
- `retire`
- `ack-conflict`

## Queen Decisions

The queen can respond to a new workerbee action in four ways:

1. `allow`
2. `warn`
3. `reroute`
4. `block`

`allow` is only valid when lane and ownership are safe.

`warn` is for soft conflict.

`reroute` means:

- reassign topic
- reassign lane
- or reassign scope

`block` is required for:

- same branch
- same worktree
- same scope/file without explicit collaboration

## Persistence

This must not live only in summaries.

Persisted artifacts are required for:

- lane state
- ownership table
- overlap table
- arbitration receipts
- handoff receipts
- retirement receipts

## Failure Modes

### Failure: shared branch workerbees

Cause:

- no branch field in hive session identity

Fix:

- branch becomes mandatory lane identity field
- same-branch workerbees become hard fault

### Failure: shared worktree workerbees

Cause:

- no worktree field in hive session identity

Fix:

- worktree root becomes mandatory lane identity field
- same-worktree workerbees become hard fault

### Failure: duplicate work before edits

Cause:

- no topic ownership arbitration

Fix:

- topic claim is required before execution

### Failure: duplicate file edits

Cause:

- no scope ownership arbitration

Fix:

- scope claims are required before touching files

## Migration Plan

### Phase 1: Lane identity

Add to hive heartbeat/schema:

- `lane_id`
- `repo_root`
- `worktree_root`
- `branch`
- `base_branch`

### Phase 2: Collision detection

Add hard-fault detection for:

- same branch
- same worktree
- same scope

### Phase 3: Auto lane creation

Workerbee spawn path auto-creates:

- branch
- worktree

### Phase 4: Topic arbitration

Add topic ownership and soft-conflict intervention.

### Phase 5: Queen enforcement

Add block/reroute/handoff receipts and enforce them at runtime.

## Success Criteria

This design is successful when:

1. A workerbee cannot silently start on another workerbee's branch.
2. A workerbee cannot silently start in another workerbee's worktree.
3. The queen always knows who owns each topic.
4. The queen always knows who owns each scope.
5. The hive intervenes before duplicate work turns into overlapping edits.
6. Live cowork feels seamless because isolation and arbitration are automatic.

## Non-Goals

- This spec does not define UI polish for the visible memories dashboard.
- This spec does not require full autonomous planning behavior by itself.
- This spec does not replace existing memory truth work; it extends cowork correctness.
