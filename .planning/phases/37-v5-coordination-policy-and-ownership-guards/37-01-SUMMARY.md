# Summary 37-01: `v5` Coordination Policy and Ownership Guards

## Outcome

Added explicit coordination modes and surfaced ownership-policy mismatches
before overlapping work turns into conflict.

## What Shipped

- added task coordination modes such as:
  - `exclusive_write`
  - `shared_review`
  - `help_only`
- threaded coordination mode through shared task persistence and MCP task upserts
- extended coordination summaries to surface policy conflicts where an
  exclusive-write task is backed by claims owned by another session

## Verification

- `cargo test -q`
- `node --check integrations/mcp-peer/server.js`

## Result

Sessions can now distinguish exclusive ownership from collaborative support
lanes before overlap becomes a coworking conflict.
