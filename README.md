# memd

`memd` is a memory control plane for agents.

It keeps memory compact, visible, scoped, and reusable across turns, tabs, and projects.

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

![memd architecture](./docs/architecture.png)

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
