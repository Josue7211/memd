# Claude Code hook wiring for memd A3

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

memd's A3 continuity foundation uses two Claude Code hooks:

1. **PostToolUse** on `Read|Edit|Write|NotebookEdit` — appends a file-interaction
   entry to the active session ledger.
2. **PreCompact** (no matcher) — blocks compaction long enough to flush durable
   state, then seals the session ledger so the continuation session can surface
   a `## Files Touched` block and `memd prime-reads` output.

Both hooks are idempotent: a missing `memd` binary, an unreachable memd server,
or an already-sealed ledger all degrade to a no-op without blocking tool calls.

## Install snippet

Merge the block below into `~/.claude/settings.json`. Adjust `$HOME` paths if
this repo lives outside `$HOME/Documents/projects/memd`.

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Read|Edit|Write|NotebookEdit",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"$HOME/Documents/projects/memd/.memd/hooks/memd-file-interaction.sh\"",
            "timeout": 10
          }
        ]
      }
    ],
    "PreCompact": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"$HOME/Documents/projects/memd/.memd/hooks/memd-precompact-save.sh\"",
            "timeout": 15
          }
        ]
      }
    ]
  }
}
```

## What the PostToolUse hook does

- Reads the Claude Code hook JSON from stdin.
- Invokes `memd hook file-interaction --output <bundle> --stdin`.
- Parses `tool_name` + `tool_input.file_path` (or `tool_input.notebook_path`
  for `NotebookEdit`), upserts an entry into
  `.memd/state/session-<session-id>/file_interactions.json`.
- `Read` / `Edit` / `Write` / `NotebookEdit` are recognised; all other tools
  are ignored.
- Upsert semantics: one entry per `(path, op)` pair; `count` increments and
  `last_ts_ms` updates on each subsequent call.

## What the PreCompact hook does

- Logs the trigger to `$MEMD_HOOK_STATE_DIR/hook.log` (defaults to
  `$HOME/.memd/hook_state/hook.log`).
- Calls `memd hook seal-ledger --session-id <id> --output <bundle>`, which
  copies the live session ledger to
  `.memd/state/session-<id>/sealed/<timestamp>.json`.
- Emits the existing `{"decision":"block","reason":"COMPACTION IMMINENT ..."}`
  JSON so Claude Code pauses to let the agent flush durable state via
  `memd hook spill` etc.

## Why this is Part 1, not Part 2

Part 1 is the **surfacing** half: the continuation session can see what the
prior session touched. Part 2 (not yet shipped) adds a cross-harness validator
that will block an Edit on a ledgered path without a prior Read in the new
session — that enforcement is what closes the "File has not been read yet"
loop. Part 1 gives Part 2 a reliable signal to gate on.

## Verifying the wiring end-to-end

After merging the snippet above:

```bash
# Trigger a Read/Edit from Claude Code on any file under this repo, then:
ls .memd/state/session-*/file_interactions.json
cat "$(ls -t .memd/state/session-*/file_interactions.json | head -1)"

# Trigger /compact in a fresh session, then inspect the sealed ledger:
ls .memd/state/session-*/sealed/

# The next wake packet should now contain a `## Files Touched` block, and:
memd prime-reads --output .memd
```

## PreToolUse continuity gate (Part 2)

Add a PreToolUse matcher alongside the PostToolUse/PreCompact hooks:

    ```json
    "PreToolUse": [
      {
        "matcher": "Edit|Write|NotebookEdit",
        "hooks": [
          { "type": "command", "command": "bash \"$HOME/Documents/projects/memd/.memd/hooks/memd-pretool-gate.sh\"", "timeout": 10 }
        ]
      }
    ]
    ```

Policy toggle: `.memd/config.json` key `continuity.enforcement` accepts `off|warn|block` (default `warn`).
