# Summary 45-01: `v5` Coordination Action Surfaces

## Goal

Let richer operator surfaces trigger bounded coordination actions through the
same shared model they already use for inspection.

## What Changed

1. Added a unified `coordination_action` MCP tool for bounded operator-facing
   coordination actions.
2. Kept the first slice focused on explicit actions:
   - `ack_message`
   - `assign_scope`
   - `recover_session`
   - `request_help`
   - `request_review`
3. Reused the existing coordination backend routes instead of inventing a
   separate UI-only action contract.
4. Updated MCP documentation to describe the new bounded action surface.

## Verification

- `cargo test -q`
- `node --check integrations/mcp-peer/server.js`

## Result

Richer operator surfaces can now act on bounded coordination pressure through
the same shared model they already use for inspection and feed consumption.
