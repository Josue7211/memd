# Claude Code Integration

Claude Code should treat `memd` as the shared memory control plane.

Recommended flow:

1. read compact context before starting work
2. store candidate memories after notable decisions
3. promote only stable takeaways

## Read Context

```bash
memd context --project <project> --agent claude-code --compact
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
