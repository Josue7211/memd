# memd

`memd` is a memory control plane and knowledge base for agents.

It turns raw work into a compact, visible, source-linked memory system that stays
usable across turns, tabs, machines, and projects. The goal is not just to store
more context. The goal is to make memory reliable: read once, compile once,
reuse forever, and always keep a path back to the evidence.

`memd` combines:

- harness packs for Codex, OpenClaw, and other agent surfaces
- compiled memory pages that stay visible on disk
- Obsidian wikilinks for human navigation
- LightRAG as the semantic recall backend
- session, tab, and project scope so live work stays separated
- quality scoring so gaps, stale facts, and contradictions are obvious

## What it does

- routes memory by intent, scope, and freshness
- compacts working state without transcript dumps
- keeps compiled memory pages visible on disk
- uses Obsidian wikilinks for the graph
- uses LightRAG as the semantic recall backend
- tracks session and tab scope for live coordination
- scores memory quality so gaps are explicit

## Quickstart

```bash
cargo run -p memd-server
cargo run -p memd-client --bin memd -- init --agent codex
memd status --output .memd
memd resume --output .memd --intent current_task
```

If you are using Codex, `memd` can load or reload the current bundle for you.

## Architecture

<img src="./docs/architecture-preview.png" alt="memd architecture" width="100%" />

See the editable source at [docs/architecture.excalidraw](./docs/architecture.excalidraw).

## Docs

- [Setup](./docs/setup.md)
- [API](./docs/api.md)
- [Architecture](./docs/architecture.md)
- [Obsidian Bridge](./docs/obsidian.md)
- [RAG](./docs/rag.md)
- [Efficiency](./docs/efficiency.md)
- [OSS Positioning](./docs/oss-positioning.md)

## Integrations

- Codex
- Claude Code
- OpenClaw
- Obsidian
- shared hook kit

## License

AGPLv3. See [LICENSE](./LICENSE).
