# memd

`memd` is an open-source memory manager and retrieval control plane for agents.

It gives AI systems one place to store, route, compact, explain, and verify memory without turning every session into a transcript dump.

Supported platforms:

- Linux
- macOS
- Windows

## What It Does

- keeps short-term working context small
- stores durable memory as typed records
- routes retrieval by intent and scope
- collapses duplicate and near-duplicate facts
- preserves source quality and provenance
- exposes inbox and explain views for review
- includes a built-in dashboard for inspection
- keeps compaction separate from durable memory

## Why It Exists

Most memory systems fail in one of two ways.

They are either:

- too forgetful, because summaries cut away the important bits
- too expensive, because they reread everything and burn tokens

`memd` is built to avoid both.

## Core Pieces

- `crates/memd-schema`: shared types and request/response schema
- `crates/memd-core`: compaction and spill logic
- `crates/memd-server`: SQLite-backed memory manager API
- `crates/memd-client`: Rust SDK and CLI
- `crates/memd-worker`: background verification worker

The core binaries are cross-platform. Linux-only deploy helpers live under `deploy/systemd/`.

## Key Features

- typed memory records
- source quality enforcement
- redundancy collapse
- route and intent based retrieval
- compact context output
- hook command for agent context and spill
- compaction spill into durable memory
- verification and expiry lifecycle
- memory inbox for review
- explain view for provenance and key inspection

## Quickstart

Run the server:

```bash
cargo run -p memd-server
```

Check health:

```bash
cargo run -p memd-client --bin memd -- healthz
```

Request compact context:

```bash
cargo run -p memd-client --bin memd -- context --project demo --agent codex --compact
```

Open the built-in dashboard:

```bash
cargo run -p memd-server
# then open http://127.0.0.1:8787/
```

Inspect the inbox:

```bash
cargo run -p memd-client --bin memd -- inbox --project demo
```

Explain one memory item:

```bash
cargo run -p memd-client --bin memd -- explain --id <uuid>
```

## Project Docs

- [Architecture](./docs/architecture.md)
- [API](./docs/api.md)
- [Compaction](./docs/compaction.md)
- [Efficiency](./docs/efficiency.md)
- [Routing](./docs/routing.md)
- [Schema](./docs/schema.md)
- [Promotion Policy](./docs/promotion-policy.md)
- [Source Policy](./docs/source-policy.md)
- [Redundancy Policy](./docs/redundancy.md)
- [Platform Support](./docs/platforms.md)
- [Hook Kit](./integrations/hooks/README.md)
- [OSS Positioning](./docs/oss-positioning.md)
- [Roadmap](./ROADMAP.md)

## Repository Layout

```text
crates/
  memd-schema/
  memd-core/
  memd-server/
  memd-client/
  memd-worker/
docs/
deploy/
integrations/
```

## Development

```bash
cargo fmt --all
cargo test
```

The server defaults to `127.0.0.1:8787`.
Set `MEMD_DB_PATH` to change the SQLite database location.

## Integrations

- Claude Code
- Codex
- Mission Control
- OpenClaw
- Shared hook kit for shell integration

## Status

The project is usable and under active development.

## License

Apache-2.0. See [LICENSE](./LICENSE).
