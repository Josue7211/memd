# Summary 44-01: `v5` UI-Friendly Coordination Feed Surfaces

## Goal

Make the bounded coordination delta model easier for richer operator surfaces to
consume without custom adapter glue.

## What Changed

1. Added a `coordination_changes` MCP tool that exposes the reusable
   coordination delta feed through the peer MCP surface.
2. Reused the same bounded `view` categories as dashboard, drilldown, watch,
   and hook surfaces.
3. Added a configurable `MEMD_BIN` path so operator environments can point the
   MCP bridge at the correct CLI binary when exposing the change feed.
4. Updated MCP documentation to describe the new UI-friendly coordination feed
   surface.

## Verification

- `cargo test -q`
- `node --check integrations/mcp-peer/server.js`

## Result

Richer operator surfaces can now consume the bounded coordination delta model
through the MCP layer without inventing a second feed shape.
