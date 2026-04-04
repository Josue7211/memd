# RAG Adapter

`memd` keeps the memory control plane in Rust and treats long-term semantic
storage as an optional backend.

The intended backend is LightRAG or a LightRAG-compatible service, but the
core product does not require it to run.

## Why It Exists

`memd` already handles:

- routing
- compaction
- spill
- verification
- inbox and explain
- short-term synced state

The RAG adapter exists to extend that system into long-term semantic retrieval
without forcing the core manager to depend on one specific vector store.

## CLI

The client exposes a dedicated `rag` subcommand:

```bash
memd rag healthz --rag-url http://127.0.0.1:9000
memd rag sync --rag-url http://127.0.0.1:9000 --project demo
memd rag search --rag-url http://127.0.0.1:9000 --query "decision cache"
```

If `--rag-url` is omitted, `MEMD_RAG_URL` is used.

`memd status --output .memd` also reports whether the bundle has RAG enabled
and whether the configured backend is reachable.

## Sync Behavior

`memd rag sync` exports canonical project and global memory items from `memd`
into the configured semantic backend.

The sync path is intentionally explicit:

- `memd` remains the source of truth for typed memory and policy
- the backend receives compact records
- duplicate and near-duplicate suppression still happens in `memd`

## Product Positioning

RAG is optional for runtime deployment, but it is a core part of the long-term
product story.

Use `memd` alone for compact structured memory.
Add RAG when you want cross-project, cross-session semantic recall.
