# Queen Bee Hive Cowork Design

Date: 2026-04-09
Status: Proposed
Owner: memd

## Why

`memd` hive is supposed to be the agent-team runtime for any harness, not only a local Codex cowork helper. Right now that model is incomplete. Multiple live sessions can share the same project hive without a first-class team contract or execution lane contract. That allows unsafe states:

- the hive behaving like a local session list instead of a real team
- workerbees sharing a branch
- workerbees sharing a worktree
- duplicate topic work starting without arbitration
- overlapping file edits starting before the hive intervenes
- wrong harnesses getting assigned work they should never own

That is not a coordination detail. It is a correctness failure. A 10-star hive cannot rely on humans to remember branch hygiene.

The product model must change from:

- "who is alive in the project hive"

to:

- "which agent team exists, which harnesses are in it, what each member can do, and who owns which execution lane, topic, and scopes right now"

## Product Thesis

`memd` hive should feel like a live classroom for agent teams across any harness:

- the `queen bee` orchestrates
- each `workerbee` is a team member with capabilities
- each coding workerbee gets its own lane
- every bee continuously broadcasts what it is doing
- the hive intervenes before overlap turns into damage

The queen bee is not only a viewer. The queen is the runtime authority for safe cowork.

The stronger product framing is:

- `memd hive = a compressed, capability-aware, queen-routed agent team runtime`

The goal is not to merely match agent teams. The goal is to beat them on:

- coordination quality
- execution speed
- routing intelligence
- token efficiency

## Core Principles

1. Isolation first

- Every coding workerbee must have its own branch.
- Every coding workerbee should have its own worktree.
- Shared branch or shared worktree between workerbees is unsafe by default.

2. Harness-agnostic teams

- Hive is not Codex-specific.
- Claude, Codex, OpenCode, Claw, OpenClaw, and future harnesses should all join the same team model.
- Queen decisions must consider harness capability, not just session liveness.

3. Intent must be explicit

- A bee must publish what it is doing before it starts doing it.
- Topic ownership and scope ownership are separate signals and both matter.

4. The queen bee arbitrates

- The queen can approve, deny, reroute, or retire work.
- Unsafe overlap is blocked, not merely logged.

5. Hive truth is runtime truth

- Coordination state must be persisted and inspectable.
- The surfaced runtime model must match what the queen uses for decisions.

6. Seamless by default

- Bees should not need a manual "remember to branch first" step.
- If the hive can safely allocate an isolated lane, it should do so automatically.

7. Compression over chatter

- Bees should exchange compact state packets, not long transcripts.
- The hive should share deltas, receipts, and handoff packets instead of replaying full conversation history.

## Roles

### Queen Bee

The queen bee is the orchestrator session. The queen:

- creates workerbee lanes
- assigns work by harness suitability
- assigns topics
- approves or denies potential overlap
- requests handoff
- retires stale lanes
- maintains the ownership table

### Workerbee

A workerbee is an execution session with a dedicated lane. A workerbee:

- belongs to a harness-specific runtime
- may own one execution lane when its work touches code
- owns one branch when acting as a coding bee
- should own one worktree when acting as a coding bee
- publishes topic and scope claims
- reports progress and blockers
- cannot override queen arbitration

### Hive

The hive is the coordination plane that stores:

- team membership
- harness and capability identity
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
7. A coding bee without a safe lane cannot start work.
8. A bee cannot be assigned work its harness cannot actually perform.

## Runtime Contract

Each live hive session must publish:

- `lane_id`
- `harness`
- `role`
- `capability_class`
- `assignment_suitability`
- `transport_surfaces`
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

These fields divide into two layers:

- universal team fields:
  - `harness`
  - `role`
  - `capability_class`
  - `assignment_suitability`
  - `transport_surfaces`
  - `topic_claim`
  - `status`
  - `progress`
  - `blockers`
- execution lane fields:
  - `lane_id`
  - `repo_root`
  - `worktree_root`
  - `branch`
  - `base_branch`
  - `scope_claims`

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

### Harness Identity

Every bee must publish:

- `harness`
- `role`
- `capability_class`
- `assignment_suitability`

This lets the queen route work correctly across mixed-harness teams.

Examples:

- a Claude bee may be excellent at planning or review
- a Codex bee may be excellent at direct code execution
- a lightweight harness may be suitable for search, triage, or watchdog tasks

The hive should know this directly instead of pretending all bees are interchangeable.

### Compressed State Packets

The hive should not be a transcript bus.

Each bee should publish compact packets containing only the runtime state needed for coordination:

- `task`
- `topic_claim`
- `scope_claims`
- `status`
- `progress`
- `blockers`
- `handoff_summary`
- `delta_since_last_update`

The queen and other bees should consume these packets instead of full raw transcripts whenever possible.

This is the token-efficiency advantage of the system.

## Conflict Model

### Soft Conflict

A soft conflict means work may overlap soon, but has not yet crossed a hard safety boundary.

Examples:

- same topic, different candidate files
- same task goal, no concrete scope claim yet
- same request routed to two suitable bees before a queen decision is recorded

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
- same work assigned to a harness that lacks the required capability

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

For non-coding bees, lane creation may omit branch/worktree but must still allocate a durable team identity and assignment record.

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

### Capability Ownership

The hive must also track which bees are suitable for which kinds of work.

This prevents:

- giving coding work to a non-coding bee
- giving review work to an execution-only bee when a better reviewer exists
- assigning the same kind of work redundantly across harnesses without reason

### Routing Intelligence

The queen should route work using capability fit, not only availability.

Examples:

- planning goes to the strongest planner
- code execution goes to the strongest coder
- review goes to the strongest reviewer
- search or triage goes to the lightest capable bee

The team should be optimized for correctness-per-token, not just concurrency.

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
- harness mix
- capability coverage
- lane health
- hard faults
- soft conflicts

### `memd coordination`

Shows:

- topic ownership
- scope ownership
- assignment suitability
- overlap table
- arbitration decisions
- packet-level handoff summaries

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

### `memd team`

New command surface for harness-agnostic teams:

- `summary`
- `members`
- `capabilities`
- `coverage`
- `route`
- `packets`

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
- clearly unsuitable harness assignment

## Persistence

This must not live only in summaries.

Persisted artifacts are required for:

- team membership
- capability coverage
- lane state
- ownership table
- overlap table
- arbitration receipts
- handoff receipts
- retirement receipts
- compressed state packets
- delta ledgers

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

### Failure: wrong harness assignment

Cause:

- hive does not model capability suitability strongly enough

Fix:

- capability and assignment suitability become first-class runtime fields
- queen routes by suitability, not just presence

### Failure: transcript-heavy cowork

Cause:

- bees communicate with repeated full-context summaries instead of compact deltas

Fix:

- compressed state packets become the default cowork transport
- handoffs and updates publish deltas and receipts, not replayed history

## Migration Plan

### Phase 1: Universal team identity

Add to hive heartbeat/schema:

- `harness`
- `capability_class`
- `assignment_suitability`
- `transport_surfaces`

### Phase 2: Lane identity

Add to hive heartbeat/schema:

- `lane_id`
- `repo_root`
- `worktree_root`
- `branch`
- `base_branch`

### Phase 3: Collision detection

Add hard-fault detection for:

- same branch
- same worktree
- same scope

### Phase 4: Auto lane creation

Workerbee spawn path auto-creates:

- branch
- worktree

### Phase 5: Topic arbitration

Add topic ownership and soft-conflict intervention.

### Phase 6: Queen enforcement

Add block/reroute/handoff receipts and enforce them at runtime.

### Phase 7: Capability routing

Add harness-aware routing so the queen chooses the right bee, not merely an available bee.

### Phase 8: Packetized cowork transport

Add compact state packets and delta-ledger transport so bee-to-bee communication stays token-cheap.

## Success Criteria

This design is successful when:

1. A workerbee cannot silently start on another workerbee's branch.
2. A workerbee cannot silently start in another workerbee's worktree.
3. The queen always knows which harnesses are in the team and what they can do.
4. The queen always knows who owns each topic.
5. The queen always knows who owns each scope.
6. The hive intervenes before duplicate work turns into overlapping edits.
7. The hive routes work to the right harness instead of treating all bees as interchangeable.
8. The hive uses compressed state packets instead of transcript replay for routine cowork.
9. Live cowork feels seamless because isolation, routing, and arbitration are automatic.

## Non-Goals

- This spec does not define UI polish for the visible memories dashboard.
- This spec does not require full autonomous planning behavior by itself.
- This spec does not replace existing memory truth work; it extends cowork correctness.
