# Codex Integration

Codex should use the same `memd` surface as every other agent.

Recommended flow:

1. fetch compact context at task start
2. write candidate memories for durable findings
3. verify or expire stale items during maintenance runs
4. use the hook command at compaction boundaries

If you want a shell-level integration, reuse the shared hook kit in
[`../hooks`](../hooks).

## Read Context

```bash
memd context --project <project> --agent codex --compact
```

## Hook Context

```bash
memd hook context --project <project> --agent codex
```

## Shell Hook Example

```bash
MEMD_PROJECT=my-project \
MEMD_AGENT=codex \
./integrations/hooks/memd-context.sh
```

## Search Memory

```bash
cat <<'JSON' | memd search --stdin
{
  "query": "postgres",
  "scopes": ["project", "global"],
  "kinds": ["fact", "topology", "runbook"],
  "statuses": ["active", "stale"],
  "project": "my-project",
  "namespace": "codex",
  "source_agent": "codex",
  "tags": ["infra"],
  "stages": ["canonical"],
  "limit": 10,
  "max_chars_per_item": 240
}
JSON
```

## Verification

```bash
cat <<'JSON' | memd verify --stdin
{
  "id": "uuid-from-search-or-context",
  "confidence": 0.95,
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
    "agent": "codex",
    "task": "fix retrieval routing"
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
