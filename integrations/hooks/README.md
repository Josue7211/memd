# memd Hook Kit

These scripts are the default agent loop integration for `memd`.

Use them when a client wants:

- a bundle-backed wake-up surface before work starts
- a stable live-capture path while task state changes
- durable spill at a compaction boundary
- a single stable path into the memory manager

For per-project bootstrap, use:

```bash
memd setup --output .memd --project <project> --namespace <namespace> --agent <agent>
```

Check bundle health with:

```bash
memd status --output .memd
```

Resume the default memory snapshot from the bundle:

```bash
memd resume --output .memd
```

Refresh the startup wake-up surface and write it into the bundle:

```bash
memd wake --output .memd --intent current_task --write
```

Force a manual refresh of the same bootstrap path in an existing session:

```bash
memd refresh --output .memd
```

That also refreshes:

- `.memd/MEMD_MEMORY.md`
- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_EVENTS.md`
- `.memd/agents/CODEX_WAKEUP.md`
- `.memd/agents/CODEX_MEMORY.md`
- `.memd/agents/CODEX_EVENTS.md`
- `.memd/agents/CLAUDE_CODE_WAKEUP.md`
- `.memd/agents/CLAUDE_CODE_MEMORY.md`
- `.memd/agents/CLAUDE_CODE_EVENTS.md`
- `.memd/agents/CLAUDE_IMPORTS.md`
- `.memd/agents/AGENT_ZERO_WAKEUP.md`
- `.memd/agents/AGENT_ZERO_MEMORY.md`
- `.memd/agents/OPENCLAW_WAKEUP.md`
- `.memd/agents/OPENCLAW_MEMORY.md`
- `.memd/agents/OPENCLAW_EVENTS.md`
- `.memd/agents/OPENCODE_WAKEUP.md`
- `.memd/agents/OPENCODE_MEMORY.md`
- `.memd/agents/OPENCODE_EVENTS.md`
- `.memd/agents/HERMES_WAKEUP.md`
- `.memd/agents/HERMES_MEMORY.md`

For Codex, that wake path is the pre-turn read step in the harness pack flow.
It pulls compiled memory first, then refreshes the visible wakeup files after a
successful backend read.

Persist a memory into the same bundle lane:

```bash
memd remember --output .memd --kind decision --content "Store the outcome worth keeping."
```

Emit a shared handoff and refresh the same markdown memory files with shared
lane/source information:

```bash
memd handoff --output .memd
```

Agent-specific bundle entrypoints are generated under `.memd/agents/`:

- `codex.sh`
- `claude-code.sh`
- `agent-zero.sh`
- `openclaw.sh`
- `opencode.sh`
- `hermes.sh`

For Claude Code, import `.memd/agents/CLAUDE_IMPORTS.md` from project
`CLAUDE.md` and verify it with `/memory`.

The same bundle also writes `.memd/COMMANDS.md`, and you can inspect the
catalog at any time with:

```bash
memd commands --output .memd --summary
```

OpenClaw is the second harness pack after Codex and uses the same shared hook
kit, but its primary flow is context + spill instead of wake + capture.

Hermes is the adoption-focused harness pack after OpenClaw and uses the same
shared hook kit, but its primary flow is onboarding-friendly wake + capture +
spill with cloud-first reach and self-host later.

Agent Zero is the zero-friction harness pack after Hermes and uses the same
shared hook kit, but its primary flow is fast resume + durable remember +
clean handoff + spill for fresh sessions.

OpenCode is the shared-lane harness pack after Agent Zero and uses the same
shared hook kit, but its primary flow is resume + remember + handoff + spill
for explicit continuity clients.

## Environment

Set:

- `MEMD_BASE_URL` - defaults to the bundle's exported value; if no bundle env is loaded it falls back to the shared Tailscale endpoint for the hosted deployment
- `MEMD_PROJECT` - required for context fetches
- `MEMD_NAMESPACE` - optional namespace lane inside the project
- `MEMD_AGENT` - required for context fetches
- `MEMD_ROUTE` - defaults to `auto`
- `MEMD_INTENT` - defaults to `current_task`
- `MEMD_WORKSPACE` - optional shared workspace lane
- `MEMD_VISIBILITY` - optional `private|workspace|public`
- `MEMD_LIMIT` - defaults to `8`
- `MEMD_MAX_CHARS` - defaults to `280`
- `MEMD_RAG_URL` - optional; bundle backend config can supply this when present

## Context Hook

```bash
./memd-context.sh
```

This now calls `memd resume --prompt` under the bundle defaults and defaults the
intent to `current_task`. It now routes through `memd wake --write` so the same
startup call both renders the live wake-up view and refreshes the generated
memory files in the bundle.

For Codex bundles, the wake path also refreshes `.memd/MEMD_WAKEUP.md`,
`.memd/MEMD_MEMORY.md`, and the Codex agent copies after a successful backend
read. If the backend read is unavailable, the existing local bundle markdown is
used instead of dropping the turn.

The installed `memd-hook-context` shim now routes through this script, so the
default installed hook path also gets the richer wake-up surface.

## Capture Hook

```bash
printf 'changed auth flow: keep optimistic UI disabled for now\n' | ./memd-capture.sh
```

This routes through `memd hook capture --stdin --summary` under the active
bundle defaults and writes an episodic live-memory update back into the hosted
backend. Use it whenever task state changes and you want the live backend to
stay ahead of transcript loss.

For Codex bundles, a successful capture also refreshes the local wake/memory
files so the visible bundle stays in sync. If capture or recall fails, the
script keeps the existing local bundle truth and preserves the turn result
instead of overwriting it with partial state.

If captured line starts with typed prefix like `decision:`, `preference:`,
`constraint:`, `fact:`, `runbook:`, `procedural:`, or `status:`, `memd hook
capture` now auto-promotes durable memory even without explicit
`--promote-kind`.

When auto-promotion fires, `memd` also auto-tags durable memory from content:
- kind tag like `decision` or `preference`
- `correction` when superseding stale memory
- `design-memory` for UX/UI/design preferences
- `product-direction` for memory-loop/startup-surface style product truth

If a captured event is durable truth instead of transient task state, promote it
in the same call:

```bash
printf 'decision: keep wake as the universal startup surface\n' | memd hook capture --output .memd --stdin --promote-kind decision --promote-tag 10-star --promote-tag product-direction
```

That records both the live episodic update and a durable typed project memory.

If the new durable memory corrects a stale belief, supersede the stale memory in
the same call:

```bash
printf 'corrected fact: hosted backend health does not prove usable agent memory\n' | memd hook capture --output .memd --stdin --promote-kind fact --promote-tag correction --promote-supersede <stale-memory-uuid>
```

Or let `memd` find likely stale targets first:

```bash
printf 'corrected fact: hosted backend health does not prove usable agent memory\n' | memd hook capture --output .memd --stdin --promote-kind fact --promote-supersede-query "hosted backend health"
```

For `corrected fact:` / `corrected decision:` / `corrected preference:` /
`corrected constraint:` / `correction:` payloads, `memd` now infers that
supersede query automatically when no explicit supersede target is provided.

## Stop Save Hook

```bash
./memd-stop-save.sh
```

This is the parity hook against MemPalace's periodic save checkpoint. It reads
the session transcript metadata from stdin and blocks every `MEMD_SAVE_INTERVAL`
user messages. The block reason forces the agent to persist state into `memd`
before ending the turn instead of relying on manual memory discipline.

Use it for harnesses that support a `Stop` hook.

## PreCompact Save Hook

```bash
./memd-precompact-save.sh
```

This hook always blocks right before context compaction and tells the agent to
checkpoint, write durable truth, and spill any available compaction packet
before compaction proceeds.

Use it for harnesses that support a `PreCompact` hook.

## Install on Unix

```bash
./install.sh
```

Optional:

- `MEMD_BIN=/path/to/memd ./install.sh`

## Spill Hook

```bash
./memd-spill.sh --stdin --apply < compaction.json
```

## Example Hook Wiring

For Codex-compatible hook runners:

```json
{
  "Stop": [{
    "type": "command",
    "command": "/absolute/path/to/memd-hook-stop-save",
    "timeout": 30
  }],
  "PreCompact": [{
    "type": "command",
    "command": "/absolute/path/to/memd-hook-precompact-save",
    "timeout": 30
  }]
}
```

## Install on Windows

```powershell
./install.ps1
```

Optional:

- `$env:MEMD_BIN = "C:\\path\\to\\memd.exe"`
