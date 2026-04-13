# Claude Code Integration

Claude Code should treat `memd` as the shared memory control plane and load it
through Claude's native memory system.

The bundle also exposes a command catalog, so the local `$` and `/` surfaces
stay documented alongside the memory files:

```bash
memd commands --root .memd --summary
```

Recommended flow:

1. import `memd` into project `CLAUDE.md`
2. verify the import chain with `/memory`
3. refresh wake-up state with `memd wake`
4. use `memd lookup` before answering about prior decisions, preferences, or history
5. write durable takeaways back through `memd`
6. let dream/autodream flow back into the same imported surface

Claude Code uses the same shared hook kit and bundle truth as the other
harness packs, but its native bridge is the primary path.

If you want a shell-level integration, reuse the shared hook kit in
[`../hooks`](../hooks).

## Native Claude Memory Bridge

If you are using a bundle, `memd` generates:

- `.memd/wake.md`
- `.memd/mem.md`
- `.memd/events.md`
- `.memd/agents/CLAUDE_IMPORTS.md`
- `.memd/agents/CLAUDE.md.example`

Use Claude's native imports from your project `CLAUDE.md`:

```md
@.memd/agents/CLAUDE_IMPORTS.md
```

Then verify the loaded files inside Claude Code:

```text
/memory
```

If you need the command list on disk, check `.memd/COMMANDS.md`.
The catalog also keeps external skill commands like `$gsd-autonomous` and
`$gsd-map-codebase` visible during migration, even though memd does not own
those commands.

The generated import chain pulls in only the hot-path wake surface:

- `.memd/wake.md`

`mem.md` and `events.md` stay cold by default. Reach for them
through `memd resume` or `memd lookup` when wake is not enough.

And use the Claude-specific entrypoint:

```bash
.memd/agents/claude-code.sh
```

## Short-Term Memory

Refresh Claude's loaded `memd` surface on the hot path:

```bash
memd wake --output .memd --intent current_task --write
memd checkpoint --output .memd --content "Current blocker: ..."
```

Use deeper recall only when you want it:

```bash
memd resume --output .memd --intent current_task --semantic
```

Before answering memory-dependent questions, run bundle-aware recall:

```bash
memd lookup --output .memd --query "what did we already decide about this?"
```

Generated bundle shortcuts:

```bash
.memd/agents/lookup.sh --query "what did we already decide?"
.memd/agents/recall-decisions.sh --query "memory recall"
.memd/agents/recall-preferences.sh --query "design taste"
.memd/agents/recall-design.sh --query "design memory"
.memd/agents/recall-history.sh --query "what happened last session?"
```

## Dream / Autodream

Dream and autodream are part of the memory system, not an extra.

- consolidate through `memd`
- write the distilled result back into the bundle memory files
- let Claude load that result through the import chain above

That keeps Claude's native memory surface and `memd`'s source of truth aligned.

## Read Context

```bash
memd context --project <project> --agent claude-code --compact
```

## Hook Context

```bash
memd hook context --project <project> --agent claude-code
```

## Shell Hook Example

```bash
MEMD_PROJECT=my-project \
MEMD_AGENT=claude-code \
./integrations/hooks/memd-context.sh
```

## Store Candidate Memory

```bash
cat <<'JSON' | memd candidate --stdin
{
  "content": "stable decision text",
  "kind": "decision",
  "scope": "project",
  "project": "my-project",
  "namespace": "claude-code",
  "source_agent": "claude-code",
  "source_system": "claude-code",
  "source_path": null,
  "confidence": 0.8,
  "ttl_seconds": null,
  "last_verified_at": null,
  "supersedes": [],
  "tags": ["decision"]
}
JSON
```

## Promote Candidate

```bash
cat <<'JSON' | memd promote --stdin
{
  "id": "uuid-from-candidate-response",
  "scope": "project",
  "project": "my-project",
  "namespace": "claude-code",
  "confidence": 0.9,
  "ttl_seconds": null,
  "tags": ["decision"],
  "status": "active"
}
JSON
```

## Spill Compaction Packet

```bash
cat <<'JSON' | memd hook spill --stdin --apply
{
  "session": {
    "project": "my-project",
    "agent": "claude-code",
    "task": "build memory manager"
  },
  "goal": "Preserve memory without token waste",
  "hard_constraints": ["compact retrieval only"],
  "active_work": ["verification worker scans stale canonical items"],
  "decisions": [],
  "open_loops": [],
  "exact_refs": [],
  "next_actions": [],
  "do_not_drop": [],
  "memory": {
    "route": "auto",
    "intent": "general",
    "retrieval_order": ["local", "synced", "project", "global"],
    "records": []
  }
}
JSON
```

## Shell Spill Example

```bash
MEMD_BASE_URL=http://100.104.154.24:8787 \
./integrations/hooks/memd-spill.sh --stdin --apply < compaction.json
```
