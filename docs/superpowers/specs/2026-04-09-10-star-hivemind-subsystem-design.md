# 10-Star Hivemind Subsystem Design

Date: 2026-04-09
Status: Proposed
Owner: memd

## Why

`memd` is the main product. `hive` is one of its flagship subsystems.

Today the hive already has strong technical pieces:

- shared awareness
- task, claim, and message coordination
- lane identity
- branch/worktree safety
- overlap blocking

But the subsystem still feels too mechanical and too low-level. It behaves like a powerful coordination backend, not yet like a premium hivemind for live agent teams.

The product gap is not missing raw capability. The gap is product shape:

- the queen is not first-class enough
- bees are still too session-shaped
- following another bee is not first-class
- the team board is spread across summaries instead of feeling like one runtime
- stale hive clutter still needs too much manual cleanup
- bee-to-bee coordination works, but does not yet feel like a natural classroom workflow

The 10-star goal is to turn hive into a clear, powerful team operating layer inside `memd`.

## Product Thesis

`memd hive` should feel like a live classroom for agent teams:

- the queen sees the whole room
- every bee has a clear identity
- every coding bee has a safe execution lane
- every bee continuously publishes compact live intent
- bees can coordinate directly without stepping on each other
- the queen intervenes before overlap becomes damage

This is not a separate product from `memd`.

The right framing is:

- `memd` = memory and control plane
- `hive` = the team orchestration subsystem inside `memd`

The 10-star hivemind promise is:

- live teamwork
- strong queen control
- seamless safety automation
- excellent operator UX

## Design Principles

1. Hive is team-shaped, not session-shaped

- sessions remain the backend truth
- humans should interact with bees, lanes, tasks, and roles

2. The queen is the authority

- queen decisions are first-class runtime actions
- unsafe cowork should be blocked, rerouted, or explicitly approved

3. Followability is mandatory

- if a bee may overlap with another bee, the user should be able to follow that bee directly
- broad awareness is not enough

4. Intent beats inference

- bees should publish structured task and scope intent
- the system should not rely on reverse-parsing ad hoc text when stronger fields exist

5. Automation should remove footguns

- worker lanes should be allocated automatically
- stale clutter should retire automatically
- overlap should be detected automatically

6. UX should surface team truth, not backend internals

- the default operator experience should look like a team board
- not a list of raw coordination primitives

7. Tokens are precious

- hive communication should prefer compact state, receipts, and handoff packets
- not transcript replay

## Core Objects

### Queen

The orchestrator and runtime authority for hive safety.

The queen can:

- assign work
- reroute bees
- deny unsafe work
- approve explicit collaboration
- request handoff
- retire stale bees
- manage the team roster

### Bee

A live hive participant.

A bee has:

- machine identity
- human identity
- team role
- lane identity
- current task
- current scopes
- current status
- capabilities

### Lane

The execution context for a coding bee.

A lane includes:

- `lane_id`
- `repo_root`
- `worktree_root`
- `branch`
- `base_branch`

Lane safety is non-negotiable for coding bees.

### Team Board

The live summary of the hive.

It shows:

- active bees
- queen decisions
- overlap risk
- review/help queues
- stale bees
- blocked bees
- recommended interventions

### Follow View

A focused live view of one bee.

It answers:

- what is this bee doing now
- what does it intend to touch
- what messages or receipts matter
- am I at risk of overlap
- what should the queen or user do next

## Identity Model

10-star hive identity must be dual:

- backend-safe
- human-usable

### Canonical Identity

- `session_id`

This remains immutable and is the backend key.

### Human Identity

- `worker_name`

This should default to the real subagent or worker name when available.

Examples:

- `Lorentz`
- `Avicenna`
- `Anscombe`

### Optional User Metadata

- `display_name`

This is an optional human-facing override, not the canonical key.

### Identity Display

Default human-readable format:

- `Lorentz (session-6d422e56)`

But hive surfaces should also show:

- `role`
- `lane`
- `task`
- `capabilities`

Example:

- `Lorentz (session-6d422e56)`
- `role=reviewer`
- `lane=render-polish`
- `task=review parser handoff`

### Identity Rules

1. `session_id` is immutable.
2. `worker_name` is the primary human label.
3. If a real subagent name exists, use it.
4. `display_name` is optional metadata only.
5. The backend stores all identity fields separately, not as one concatenated label.

## Runtime Contract

Each live bee should publish:

- `session_id`
- `worker_name`
- `display_name`
- `harness`
- `role`
- `capabilities`
- `lane_id`
- `repo_root`
- `worktree_root`
- `branch`
- `base_branch`
- `task_id`
- `topic_claim`
- `scope_claims`
- `next_action`
- `status`
- `needs_help`
- `needs_review`
- `handoff_state`
- `confidence`
- `risk`
- `updated_at`

The current system already has part of this. The 10-star design requires hive to complete and center this model.

## Subsystem Surface Model

The hivemind should feel coherent. The command model should reflect the team model directly.

### Primary Entry

- `memd hive`

This is the home of the subsystem.

### Main Subcommands

- `memd hive`
  - live team board
- `memd hive roster`
  - identity and membership
- `memd hive follow`
  - watch one bee closely
- `memd hive queen`
  - orchestration actions
- `memd hive lane`
  - lane inspection and repair
- `memd hive handoff`
  - structured handoff flow

### Existing Supporting Surfaces

These remain valid subsystem primitives:

- `memd tasks`
- `memd messages`
- `memd claims`
- `memd coordination`
- `memd awareness`

But the hivemind experience should not require users to mentally compose all of these from scratch every time.

## 10-Star Surface Details

### 1. Hive Board

`memd hive`

Should show:

- queen identity
- active bees
- blocked bees
- stale bees
- review/help queue
- overlap risk
- lane faults
- recommended queen actions

The board should answer:

- who is here
- who owns what
- who is blocked
- where overlap risk exists
- what to do next

### 2. Roster View

`memd hive roster`

Should show for each bee:

- `worker_name`
- `session_id`
- `role`
- `lane`
- `task`
- `capabilities`
- `status`

This is the stable human team map.

### 3. Follow View

`memd hive follow --worker <name>`

or

`memd hive follow --session <id>`

Optional flags:

- `--watch`
- `--summary`
- `--json`
- `--show messages,tasks,claims,receipts`
- `--overlap-with current`

Should show:

- identity
- current task
- work summary
- touched scopes/files
- recent messages
- recent receipts
- recent task transitions
- overlap risk with current bee
- recommended next action

### 4. Queen View

`memd hive queen`

Should support:

- assign
- reroute
- deny
- approve collaboration
- request handoff
- retire

Every queen action should write a receipt.

### 5. Lane View

`memd hive lane`

Should surface:

- lane identity
- branch/worktree state
- lane faults
- lane creation and reroute receipts
- stale lane status

### 6. Handoff View

`memd hive handoff`

Should support compact structured handoff packets between bees:

- current task
- claimed scopes
- status
- blocker
- next action
- requested receiver

## Queen Model

The queen must be explicit and first-class.

### Queen Responsibilities

- maintain the roster
- maintain ownership tables
- assign roles
- allocate lanes
- prevent unsafe overlap
- approve explicit collaboration
- retire stale bees
- manage handoff flow

### Queen Receipts

At minimum:

- `queen_assign`
- `queen_reroute`
- `queen_deny`
- `queen_handoff`
- `queen_retire`

### Queen Policy

Hard block:

- same branch
- same worktree
- same concrete scope without explicit collaboration

Soft intervention:

- same topic
- rising overlap risk
- stale bee with pending ownership

## Bee-to-Bee Coordination

Bees need lightweight direct teamwork without losing queen authority.

### Supported Bee-to-Bee Flows

- direct note
- help request
- review request
- handoff request
- collaboration acknowledgement

### Rules

1. Bee-to-bee messages are allowed.
2. Unsafe ownership changes are still queen-governed.
3. Requests and replies should remain compact.
4. The team board should surface pending help/review state.

## Lane Safety Model

The current lane work moves in the right direction. The 10-star subsystem keeps that and elevates it.

### Invariants

1. Coding bees never share a branch.
2. Coding bees never share a worktree.
3. Same explicit file/scope without collaboration approval is blocked.
4. A coding bee gets a lane automatically.
5. Lane faults are visible on the hive board.

### Automation

- auto-create branch/worktree lanes for coding bees
- auto-reroute on detected unsafe collision when safe to do so
- otherwise hard block

## Automatic Hygiene

The hivemind should not accumulate junk states forever.

### Automatic Retirement

Remote or local stale bees with:

- no active claims
- no active tasks
- no pending handoff

should auto-retire.

### Default View Hygiene

The default hive board should prioritize:

- live bees
- actionable stale bees

Historical dead clutter should not dominate the operator view.

## Overlap Model

Overlap must be high-signal.

### Real overlap signals

- same branch
- same worktree
- same file/path
- same task scope
- same confirmed topic when work would collide

### Weak signals that should not dominate

- generic `project`
- generic `workspace`
- vague shared presence

The hivemind should surface high-value overlap warnings, not spam the operator with obvious shared-team context.

## Operator UX

The hivemind should feel premium in both CLI and dashboard.

### CLI

Needs:

- short board summary
- worker names first
- session ids second
- clear recommendations
- obvious queen actions

### Dashboard

Should eventually mirror the CLI team board with:

- active bee cards
- queue panels
- lane fault panel
- queen action panel
- follow panel

## Non-Goals

This spec does not attempt to:

- turn hive into the entire `memd` product
- replace memory/retrieval as separate pillars
- replace raw coordination primitives with opaque automation
- make random codenames the default identity system

## Success Criteria

The hivemind is 10-star when all of the following are true:

1. A user can identify every active bee by name, role, lane, and task.
2. The queen can see and intervene in unsafe work before edits collide.
3. A user can follow a bee directly without digging through low-level surfaces.
4. Bee-to-bee communication feels natural and visible.
5. Lane safety is automatic and reliable.
6. Stale clutter is cleaned up automatically.
7. The default hive board is actionable and calm.
8. The subsystem feels like one coherent team runtime, not a bag of endpoints.

## Recommended Rollout Order

1. Roster and identity model
2. `memd hive follow`
3. queen-first action surface
4. team board polish
5. automatic stale-bee retirement
6. structured handoff flows
7. dashboard parity
