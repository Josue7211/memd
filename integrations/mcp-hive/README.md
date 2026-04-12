# memd Hive MCP

This MCP server exposes the brokered `memd` hive coordination substrate as
agent-facing tools.

It reuses the existing `memd-server` coordination backend:

- hive messages
- hive inbox and acknowledgement
- brokered claims
- claim transfer
- assignment-friendly work handoff

It does not create a second coordination store.

## Tools

- `list_hives`
- `check_inbox`
- `coordination_inbox`
- `coordination_dashboard`
  - optional `view`: `all`, `inbox`, `requests`, `recovery`, `policy`, `suggestions`, `history`
- `coordination_changes`
  - optional `view`: `all`, `inbox`, `requests`, `recovery`, `policy`, `suggestions`, `history`
- `coordination_suggestions`
  - optional `view`: `all`, `policy`, `suggestions`, `history`
- `coordination_action`
  - `action`: `ack_message`, `assign_scope`, `recover_session`, `request_help`, `request_review`
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
- `bash_exec`
  - isolated `just-bash` shell rooted at `project`, `bundle`, or `integration`
  - default overlay mode keeps writes in memory only
  - set `allow_write: true` only when you want disk writes inside that root

## Environment

- `MEMD_BUNDLE_ROOT`
  Path to the active bundle root. Defaults to `./.memd`.
- `MEMD_BIN`
  Optional path to the `memd` CLI binary when `coordination_changes` should
  use a non-default executable.

The server reads the current bundle identity from `config.json` and uses the
bundle's configured `base_url` for coordination calls.
If a project hive is enabled, the bundle should already be opted into the shared
hive URL with `memd hive-project --enable`; `memd hive` is still the live session
join, and `memd hive-link` stays the safe cross-project pairing path.

## Install

```bash
cd integrations/mcp-hive
npm install
```

This package now installs `just-bash` and exposes it through `bash_exec`.

## Example MCP entry

```json
{
    "memd-hive": {
    "command": "node",
    "args": ["./integrations/mcp-hive/server.js"],
    "env": {
      "MEMD_BUNDLE_ROOT": "/absolute/path/to/project/.memd"
    }
  }
}
```

## Example `bash_exec`

Read-only repo inspection:

```json
{
  "name": "bash_exec",
  "arguments": {
    "script": "pwd && rg -n \"voice_mode\" .memd/config.json",
    "root": "project"
  }
}
```

Writable bundle update:

```json
{
  "name": "bash_exec",
  "arguments": {
    "script": "echo test >> notes.txt",
    "root": "bundle",
    "allow_write": true
  }
}
```

## Notes

- hive discovery is bundle-aware: it scans sibling project bundles and reads
  their session identity and heartbeat state
- message and claim operations use the shared `memd-server` backend routes
- claims, tasks, and assignments preserve session-qualified ownership instead
  of collapsing everyone into one lane
- dashboard drilldowns reuse the same bounded coordination categories as the
  CLI summary surface
