# Summary 34-01: `v5` Shared Task Orchestration

## Outcome

Turned peer coordination from raw messages and scope claims into an explicit
shared-task layer across the backend, CLI, and MCP bridge.

## What Shipped

- added shared peer task records and backend routes for:
  - task upsert
  - task assignment
  - task listing
- added CLI task orchestration with:
  - `memd tasks --upsert`
  - `memd tasks --assign-to-session`
  - `memd tasks --request-help`
  - `memd tasks --request-review`
- preserved session-qualified ownership while transferring linked claims during
  task assignment
- extended `integrations/mcp-peer` with task-native coworking tools instead of
  forcing agents through claim/message choreography only

## Verification

- `cargo test -q`
- `node --check integrations/mcp-peer/server.js`

## Result

Simultaneous agent sessions can now coordinate around named shared tasks
instead of only loose scopes and direct messages.
