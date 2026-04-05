# RAG Adapter

`memd` keeps the memory control plane in Rust and treats long-term semantic
storage as an optional backend.

## Tier Position

RAG is the third deployment tier, not the starting requirement:

- Tier 1: Obsidian-only markdown-native knowledge
- Tier 2: shared sync and shared memory lanes
- Tier 3: LightRAG semantic retrieval

The markdown/file layer stays first-class even after the semantic backend is attached.

For daily work, the flow should be:

1. store and inspect state in `memd`
2. compile or write back useful evidence into Obsidian when it should stay visible
3. use LightRAG only when the vault or project memory is too large for markdown-native retrieval alone

The intended backend is LightRAG or a LightRAG-compatible service behind
`rag-sidecar`, but the core product does not require it to run.

That is important because `memd` also supports a markdown-native path through
Obsidian vault ingest and compiled wiki workflows. For smaller knowledge bases,
that can be enough without reaching for a semantic backend first.

The full backend stack, when configured, must support multimodal inputs:

- video
- PDF
- image
- table
- equation

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

If the client is running inside a `memd` project bundle, it will prefer the
bundle's `config.json` `backend.rag.url` setting before falling back to
`MEMD_RAG_URL`.

`memd status --output .memd` also reports whether the bundle has RAG enabled
and whether the configured backend is reachable.

## Sync Behavior

`memd rag sync` exports canonical project and global memory items from `memd`
into the configured semantic backend.

The sync path is intentionally explicit:

- `memd` remains the source of truth for typed memory and policy
- the backend receives compact records
- duplicate and near-duplicate suppression still happens in `memd`
- the semantic layer augments the markdown/file layer instead of replacing it

## Product Positioning

RAG is optional for runtime deployment, but it is a core part of the long-term
product story.

Use `memd` alone for compact structured memory.
Use Obsidian plus `memd` for markdown-native raw-source and compiled-wiki workflows.
Add RAG when you want cross-project, cross-session semantic recall.
Keep `memd` as the control plane even after RAG is added so retrieval, provenance,
and branchable belief state stay in one place.

See also:

- [Backend Stack Contract](./backend-stack.md)
- [Backend Ownership Split](./backend-ownership.md)
- [Backend Implementation Plan](./backend-implementation-plan.md)
- [Credits](./credits.md)
