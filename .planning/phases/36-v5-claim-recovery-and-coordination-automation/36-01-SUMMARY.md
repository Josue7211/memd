# Summary 36-01: `v5` Claim Recovery and Coordination Automation

## Outcome

Added a bounded stale-session recovery path so blocked coworking lanes can be
reclaimed without silent ownership drift.

## What Shipped

- added backend claim recovery support under `/coordination/claims/recover`
- extended CLI coordination surfaces to:
  - detect stale/dead peer ownership
  - surface reclaimable claims and stalled tasks
  - recover stale-session work into the current or chosen live session
- extended the MCP bridge with `recover_stale_session`
- kept recovery explicit and session-qualified instead of hidden reassignment

## Verification

- `cargo test -q`
- `node --check integrations/mcp-peer/server.js`

## Result

Active sessions can now recover blocked shared work from stale or dead peers
using an inspectable recovery path.
