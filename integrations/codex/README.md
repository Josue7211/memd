# Codex Integration

Codex should use the same `memd` surface as every other agent.

Recommended flow:

1. fetch compact context at task start
2. write candidate memories for durable findings
3. verify or expire stale items during maintenance runs

## Read Context

```bash
memd context --project <project> --agent codex --compact
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
