# Summary 35-01: `v5` Coordination Inbox and Task Presence

## Outcome

Added a compact coordination inbox that combines direct peer messages with
shared-task ownership and help/review pressure.

## What Shipped

- added backend coordination inbox support under `/coordination/inbox`
- added CLI coordination view for active bundle sessions
- exposed the same compact inbox through the MCP bridge
- kept the surface session-scoped and inspectable instead of hiding state
  behind opaque automation

## Verification

- `cargo test -q`
- `node --check integrations/mcp-peer/server.js`

## Result

Active sessions can now inspect coordination pressure from one compact surface
instead of manually stitching together messages, tasks, and liveness.
