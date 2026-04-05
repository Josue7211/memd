# memd Peer MCP

This MCP server exposes the brokered `memd` peer coordination substrate as
agent-facing tools.

It reuses the existing `memd-server` coordination backend:

- peer messages
- peer inbox and acknowledgement
- brokered claims
- claim transfer
- assignment-friendly work handoff

It does not create a second coordination store.

## Tools

- `list_peers`
- `check_inbox`
- `coordination_inbox`
- `coordination_dashboard`
  - optional `view`: `all`, `inbox`, `requests`, `recovery`, `policy`, `history`
- `recover_stale_session`
- `recommend_boundaries`
- `send_message`
- `ack_message`
- `list_claims`
- `acquire_claim`
- `release_claim`
- `transfer_claim`
- `assign_work`
- `list_tasks`
- `upsert_task`
- `assign_task`
- `request_task_help`
- `request_task_review`

## Environment

- `MEMD_BUNDLE_ROOT`
  Path to the active bundle root. Defaults to `./.memd`.

The server reads the current bundle identity from `config.json` and uses the
bundle's configured `base_url` for coordination calls.

## Install

```bash
cd integrations/mcp-peer
npm install
```

## Example MCP entry

```json
{
  "memd-peer": {
    "command": "node",
    "args": ["./integrations/mcp-peer/server.js"],
    "env": {
      "MEMD_BUNDLE_ROOT": "/absolute/path/to/project/.memd"
    }
  }
}
```

## Notes

- peer discovery is bundle-aware: it scans sibling project bundles and reads
  their session identity and heartbeat state
- message and claim operations use the shared `memd-server` backend routes
- claims, tasks, and assignments preserve session-qualified ownership instead
  of collapsing everyone into one lane
- dashboard drilldowns reuse the same bounded coordination categories as the
  CLI summary surface
