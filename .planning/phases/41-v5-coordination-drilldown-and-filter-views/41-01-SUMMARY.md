# Summary 41-01: `v5` Coordination Drilldown and Filter Views

## Goal

Make coordination inspection faster under load by letting operators and peer
agents focus on one bounded coordination slice at a time.

## What Changed

1. Added `--view` support to the CLI coordination summary so operators can
   focus on `inbox`, `requests`, `recovery`, `policy`, or `history` instead of
   always reading the full dashboard.
2. Refactored the CLI coordination renderer into bounded section helpers so the
   dashboard and drilldown views stay aligned.
3. Extended the MCP `coordination_dashboard` tool with the same bounded view
   selector and matching recovery/policy/history drilldowns.
4. Updated the peer MCP documentation to describe the shared drilldown model.

## Verification

- `cargo test -q`
- `node --check integrations/mcp-peer/server.js`

## Result

Coordination inspection now supports overview plus bounded drilldowns across CLI
and MCP surfaces, so active coworking sessions can jump straight to the
pressure slice they need.
