# Claude Code Integration

Claude Code should treat `memd` as the shared memory control plane and load it
through Claude's native memory system.

Recommended flow:

1. import `memd` into project `CLAUDE.md`
2. verify the import chain with `/memory`
3. refresh short-term state with `memd resume` and `memd checkpoint`
4. write durable takeaways back through `memd`
5. let dream/autodream flow back into the same imported surface

If you want a shell-level integration, reuse the shared hook kit in
[`../hooks`](../hooks).

## Native Claude Memory Bridge

If you are using a bundle, `memd` generates:

- `.memd/MEMD_MEMORY.md`
- `.memd/agents/CLAUDE_CODE_MEMORY.md`
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

The generated import chain pulls in both:

- `.memd/MEMD_MEMORY.md`
- `.memd/agents/CLAUDE_CODE_MEMORY.md`

And use the Claude-specific entrypoint:

```bash
.memd/agents/claude-code.sh
```

## Short-Term Memory

Refresh Claude's loaded `memd` surface on the hot path:

```bash
memd resume --output .memd --intent current_task
memd checkpoint --output .memd --content "Current blocker: ..."
```

Use semantic fallback only when you want deeper recall:

```bash
memd resume --output .memd --intent current_task --semantic
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
MEMD_BASE_URL=http://127.0.0.1:8787 \
./integrations/hooks/memd-spill.sh --stdin --apply < compaction.json
```
