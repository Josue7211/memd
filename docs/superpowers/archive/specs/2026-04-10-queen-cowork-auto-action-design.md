# Queen Cowork Auto-Action Design

## Background
The live hive now has canonical cowork state:
- `working`
- normalized `touches`
- `blocked_by`
- `cowork_with`
- `handoff_target`
- `offered_to`
- relationship states such as `near`, `blocked`, `cowork_active`, and `handoff_ready`

The CLI also has an explicit cowork protocol:
- `memd hive cowork request`
- `memd hive cowork ack`
- `memd hive cowork decline`

The remaining gap is queen behavior. Right now queen can summarize coordination, but it does not yet turn live cowork state into first-class operator guidance or safe auto-actions.

## Goal
Make queen the live cowork dispatcher for the hive while preserving operator control.

Queen should:
- detect when bees are `near`, `blocked`, or already `cowork_active`
- surface explicit cowork actions
- prepare the exact cowork packet or command
- optionally auto-dispatch only when the policy allows it

The default must be transparent and low-noise. Auto-send is an opt-in escalation, not the baseline.

## Non-Goals
- Do not replace the explicit `hive cowork` CLI
- Do not auto-send every detected overlap
- Do not change the underlying cowork transport format
- Do not add a second, queen-only cowork protocol
- Do not infer cowork intent from stale memory when canonical live fields exist

## Current State
Queen already:
- reads coordination suggestions
- renders action cards
- can recommend `deny`, `reroute`, `handoff`, `retire`, `request_review`, and `request_help`

The new cowork protocol already:
- sends `cowork_request`, `cowork_ack`, and `cowork_decline`
- records coordination receipts
- shows up on follow surfaces

What is missing:
- queen does not yet promote live relationship state into cowork-specific action cards
- queen does not yet have an explicit policy for suggest vs auto-send
- queen does not yet distinguish safe cowork dispatch from borderline overlap

## Proposed Shape
Use a hybrid queen model:

1. Suggest by default
- queen surfaces a cowork action card with the exact command that would be sent
- this is the default for all `near` and `blocked` cases

2. Auto-send only with policy
- queen can dispatch cowork packets automatically only when:
  - the session opts in, or
  - the relation is high-confidence and low-risk

3. Preserve explicit verbs
- `request` for new coordination
- `ack` for active cowork links
- `decline` for rejecting or rerouting an unsafe or wrong-lane request

This keeps the protocol honest:
- operator-visible by default
- automatic only when clearly safe

## Interaction Model
Queen should treat the cowork relationship states like this:

- `near`
  - create a `request_cowork` suggestion
  - priority: low or medium depending on scope overlap
  - command should target the peer bee and include the live task/scope context

- `blocked`
  - create a high-priority `request_cowork` suggestion
  - if the peer is already the blocker, the packet should clearly name the blocker relationship

- `cowork_active`
  - create an `ack_cowork` suggestion
  - queen should prefer keep-alive or acknowledgement semantics instead of opening a new handoff

- `handoff_ready`
  - keep the existing handoff behavior available
  - cowork should not replace handoff where the task boundary is already clear

Auto-send rules:
- off by default
- explicit opt-in can enable auto-send for selected repositories or sessions
- the first implementation slice should use a queen CLI flag, not hidden heuristics, for example `--cowork-auto-send`
- only `near` and `cowork_active` are eligible for automatic dispatch by default policy
- `blocked` may auto-send only if the policy explicitly allows it

## Data Flow
1. Queen builds its action list from the current coordination snapshot
2. The suggestion engine emits cowork-related suggestions when peers are near/blocked/active
3. Queen renders those suggestions as action cards
4. If auto-send is enabled and the policy allows it, queen dispatches the cowork packet through the same `hive cowork` transport used by the CLI
5. Queen records the resulting receipt and surfaces it in follow/board summaries

Important boundary:
- queen is a coordinator, not a second transport layer
- the same packet and receipt kinds must be used everywhere

## Safety And Guardrails
Queen auto-send must be conservative:
- never send to self
- never send without a resolved target bee
- never send if the target cannot be resolved in awareness
- never auto-send across an ambiguous or conflicting target match
- never auto-send when the command would obviously collide with an exclusive-write lane

When queen refuses to auto-send, it should still surface the exact packet as a suggestion so the operator can send it manually.

## Error Handling
If a cowork action cannot be resolved:
- explain why in the action card
- include the target lookup failure
- do not silently downgrade to a generic queen action

If the policy blocks auto-send:
- keep the suggestion visible
- mark the action as manual-only
- preserve the command payload for copy/use

If the peer is already in `cowork_active`:
- prefer `ack_cowork`
- avoid duplicating request packets unless the operator explicitly asks for it

## Testing
Add coverage for:
- queen suggestion generation for `near`
- queen suggestion generation for `blocked`
- queen suggestion generation for `cowork_active`
- action-card rendering for cowork packets
- policy gating for suggest-only vs auto-send
- live request/ack visibility on follow surfaces after queen dispatch

The tests should prove:
- queen emits a cowork action card when live relationship state justifies it
- the command shown to the operator matches the actual packet sent
- the packet kind and receipt kind are consistent across CLI and queen

## Acceptance Criteria
This slice is done when:
- queen surfaces cowork-specific action cards for `near`, `blocked`, and `cowork_active`
- queen can render the exact `memd hive cowork ...` command for each suggested action
- queen can auto-send only when policy explicitly allows it
- live follow surfaces show the resulting cowork receipt
- no existing handoff or deny behavior regresses

## Implementation Order
1. Add queen-side cowork suggestion cards from the existing coordination graph
2. Render cowork-specific action card commands
3. Add an explicit auto-send policy gate
4. Verify live dispatch in a shared-hive dogfood run
