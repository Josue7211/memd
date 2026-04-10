# 10-Star Live Hive Cowork Design

## Goal

Make four live projects behave like one coordinated working hive instead of four isolated sessions with heartbeats.

The system should answer, in real time:

- who is alive
- what each bee is working on right now
- what each bee is touching or approaching
- who is blocked on whom
- when two bees should cowork before conflict
- when a handoff, help request, review request, or deny/reroute action is needed

This is not a heartbeat-interval problem. Fresh but semantically empty heartbeats do not produce live coworking.

## Product Standard

The 10-star standard is:

- every bee continuously publishes a short live work card
- every bee can see peers in the same hive group, even across projects
- the hive computes peer relationship state continuously
- the system surfaces suggested coordination actions before overlap becomes a collision
- roster, board, and follow all expose the same canonical live cowork truth

## Current Gaps

Current behavior already has pieces of the model:

- presence
- topic claims
- scope claims
- next action
- help/review flags
- overlap detection

But the current system is still below 10-star because:

- work blurbs are inconsistent and often derived from stale or noisy sources
- cross-project visibility has had project-scope leaks instead of true group-scoped truth
- coworking is implicit instead of first-class
- bees do not reliably advertise dependencies on other bees
- operators still have to infer what is happening instead of reading a live card

## Canonical Live Work Card

Each bee must publish one canonical live work card as part of the live hive state.

### Required fields

- `session`
- `project`
- `namespace`
- `workspace`
- `worker_name`
- `working`
- `next_action`
- `touches`
- `status`
- `updated_at`

### Coordination fields

- `needs_help`
- `needs_review`
- `blocked_by`
- `cowork_with`
- `handoff_target`
- `offered_to`

### Derived fields

- `relationship_state`
- `relationship_peer`
- `relationship_reason`
- `suggested_action`

## Semantic Rules

### `working`

`working` is the short live sentence for what the bee is doing now.

Requirements:

- human-readable
- one sentence or short phrase
- derived from real live state, not generic focus noise
- updated whenever the bee materially changes work

Examples:

- `fixing hive group visibility`
- `reviewing clawcontrol session merge behavior`
- `waiting on secret-broker token contract`
- `wiring agent-shell launcher worker names`

### `next_action`

`next_action` is the next concrete step, not a broad intention.

Examples:

- `patch shared awareness query scope`
- `ask clawcontrol for merge semantics`
- `wait for noether review ack`

### `touches`

`touches` is the normalized set of current nearby scopes.

Types:

- `task:<id>`
- `file:<path>`
- `topic:<name>`
- `area:<subsystem>`

Examples:

- `task:hive-awareness`
- `file:crates/memd-client/src/main.rs`
- `topic:session-merge`
- `area:coordination-runtime`

These values must be normalized so they can be compared reliably across bees.

## Relationship Engine

The hive computes a relationship between bees continuously.

### States

- `clear`
- `near`
- `conflict`
- `blocked`
- `cowork_active`
- `handoff_ready`

### Rules

#### `conflict`

Triggered by:

- exact task overlap
- exact file overlap
- exact exclusive-scope overlap

Effect:

- loud warning
- deny/reroute suggestion if exclusivity applies

#### `near`

Triggered by:

- same subsystem area
- related topic
- adjacent touched files
- shared workspace with meaningful touch overlap

Effect:

- suggest cowork before collision

#### `blocked`

Triggered by:

- bee explicitly depends on another bee
- bee requests a contract, answer, or review from another bee

Effect:

- suggest handoff/help/review route

#### `cowork_active`

Triggered by:

- both sides acknowledge active coordination

Effect:

- reduce false conflict noise
- show explicit cowork pairing

#### `handoff_ready`

Triggered by:

- one bee is at a clean boundary and another bee is the logical next owner

Effect:

- suggest or automate handoff

## Live Behavior Model

Bees should not silently drift into each other’s area.

The behavior loop is:

1. bee publishes live work card
2. hive computes relationship state against peers
3. system surfaces suggested action
4. bees can request cowork, acknowledge cowork, request help, request review, or hand off work
5. relationship state updates live

This turns hive from passive awareness into active coordination.

## Operator UX

All live surfaces should read from the same canonical live work card.

### Roster

One line per bee with:

- bee name
- project
- short work sentence
- relationship state
- top touches

Example:

- `Memd | fixing hive group visibility | near clawcontrol | touches: file:crates/memd-client/src/main.rs, task:hive-awareness`

### Board

Group cards by:

1. `conflicts`
2. `nearby`
3. `blocked`
4. `clear`

Each card shows:

- who
- what
- peer relation
- suggested action

Action examples:

- `cowork`
- `handoff`
- `request review`
- `wait`
- `continue`

### Follow

Follow must be a live delta stream, not just a periodic summary replay.

Events:

- `work_changed`
- `touches_changed`
- `near_detected`
- `conflict_detected`
- `blocked_on_peer`
- `cowork_requested`
- `cowork_acknowledged`
- `risk_cleared`
- `handoff_sent`
- `handoff_accepted`

## Cross-Project Hive Truth

If bees are in the same hive group, they must be visible to each other across project boundaries.

That means:

- awareness queries must support true hive-group scope
- roster views must not collapse back to project-only filtering when a hive group exists
- group peers must remain visible even when their `project` differs

This is mandatory for live cowork across `memd`, `clawcontrol`, `secret-broker`, and `agent-shell`.

## Data Source Policy

Canonical priority for live work card data:

1. explicit live runtime work state
2. active task state
3. normalized heartbeat fields
4. fallback derivation from current bundle/runtime state

The system should avoid showing noisy placeholder strings when better live data is available.

## Safety Policy

The hive should bias toward early coordination instead of late collision.

Policy:

- `near` should trigger suggestion, not panic
- `conflict` should trigger warning and, for exclusive scopes, deny/reroute recommendation
- `blocked` should trigger help/review/handoff routing
- `cowork_active` should suppress duplicate conflict noise where appropriate

## Implementation Plan Shape

This should be built in slices.

### Slice 1: Canonical work card and roster truth

- add canonical `working` field
- normalize `touches`
- compute `relationship_state`
- surface work sentence plus relation in roster

### Slice 2: Cross-project live cowork detection

- group-scoped awareness and roster behavior
- `near` versus `conflict` severity
- peer reasoning and suggested actions

### Slice 3: First-class cowork actions

- cowork request
- cowork acknowledge
- blocked-by signaling
- handoff-ready signaling

### Slice 4: Follow and board parity

- live delta events
- operator board grouped by action priority
- clear/near/conflict/blocked visibility across all surfaces

## Testing Strategy

Required verification:

- cross-project hive-group sessions are visible together
- every active bee shows a short live work sentence
- relationship engine distinguishes `clear`, `near`, `conflict`, and `blocked`
- cowork requests and acknowledgements transition state correctly
- follow emits live relationship deltas
- roster, board, and follow agree on the same canonical truth

## Recommendation

Start with the smallest high-value slice:

- canonical `working`
- normalized `touches`
- `relationship_state`
- roster rendering

That gives immediate live clarity across multiple projects and creates the base model the rest of the hive can build on.
