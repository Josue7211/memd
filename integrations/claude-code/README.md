# Claude Code Integration

Claude Code should treat `memd` as the shared memory control plane.

Recommended flow:

1. read compact context before starting work
2. store candidate memories after notable decisions
3. promote only stable takeaways
4. use the hook command at compaction boundaries

If you want a shell-level integration, reuse the shared hook kit in
[`../hooks`](../hooks).

If you are using a bundle, read:

- `.memd/MEMORY.md`
- `.memd/agents/CLAUDE_CODE_MEMORY.md`

And use the Claude-specific entrypoint:

```bash
.memd/agents/claude-code.sh
```

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
