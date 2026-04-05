# Summary 40-01: `v5` Coordination Dashboard and History Views

## Goal

Make live coordination pressure and recent coworking history faster to inspect
than raw JSON or one-line counters.

## What Changed

1. Refactored the CLI coordination summary into a compact dashboard layout with
   explicit sections for inbox, requests, recovery, policy, and recent history.
2. Added a `coordination_dashboard` MCP tool so agent peers can inspect the same
   pressure and history surface without reimplementing their own view logic.
3. Updated the peer MCP documentation to include the new dashboard surface.

## Verification

- `cargo test -q`
- `node --check integrations/mcp-peer/server.js`

## Result

Operators and peer agents can inspect current coworking pressure and recent
coordination receipts from a cleaner dashboard-style surface instead of
reconstructing state from raw counters and receipts.
