# Summary 38-01: `v5` Branch and Scope Recommendations

## Outcome

Added advisory branch and scope recommendations so simultaneous sessions can
split work more cleanly before implementation overlap begins.

## What Shipped

- extended coordination summaries with boundary recommendations derived from:
  - shared task coordination mode
  - task scope bindings
  - current ownership
- added MCP `recommend_boundaries` for agent-native access to the same advice
- kept the layer advisory instead of mutating git state automatically

## Verification

- `cargo test -q`
- `node --check integrations/mcp-peer/server.js`

## Result

Peer sessions can now see cleaner branch and scope suggestions before they step
into overlapping implementation lanes.
