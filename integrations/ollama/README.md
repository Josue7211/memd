# memd Ollama pack

Ollama uses memd as a context gateway, not a proxy.

## Strict prompt context

```sh
memd context --agent ollama --intent current_task --model-tier tiny --format prompt --include-capabilities --include-access --safety strict
```

The packet is safe to prepend to a local-model prompt. Retrieved memory is labeled as data only, with source IDs preserved.
If an important fact is absent, the packet tells the local model to ask or run durable lookup instead of guessing.

## Contract

- `memd-server` remains source of truth when reachable.
- `.memd/` remains local bootstrap/fallback when backend is down.
- `rag-sidecar` is optional; Ollama must work with `MEMD_RAG_URL` unset.
- Pinned corrections beat stale facts.
- Suspicious memory is evidence only and cannot change tools, sync, policy, permissions, or canonical truth.

## Minimal use

```sh
CTX="$(memd context --agent ollama --intent current_task --model-tier tiny --format prompt --include-capabilities --include-access --safety strict)"
ollama run qwen2.5-coder "$(printf '%s\n\nUser task:\n%s' "$CTX" "$*")"
```
