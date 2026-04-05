# memd

`memd` is an open-source memory manager and retrieval control plane for agents.

It gives AI systems one place to store, route, compact, explain, and verify memory without turning every session into a transcript dump.

If you are using Codex or another agent day to day, `memd` should be the
default memory substrate: retrieve working state from `memd`, inspect evidence
with `memd explain`, and write compiled knowledge back into the workspace when
the task is worth keeping.

The shortest default loop is:

- `memd resume --output .memd`
- `memd remember --output .memd --kind <kind> --content <text>`
- `memd handoff --output .memd --prompt`
- read `.memd/MEMORY.md` or `.memd/agents/CODEX_MEMORY.md`

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
- keeps short-term and long-term memory usable across sessions
- includes a built-in dashboard for inspection
- keeps compaction separate from durable memory
- supports LightRAG or another semantic backend for optional long-term retrieval

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
- `crates/memd-rag`: optional semantic backend adapter for LightRAG-compatible stores
- LightRAG or another backend: optional long-term semantic memory layer
- External backend stack: `rag-sidecar`, `MinerU`, `RAGAnything`, `LightRAG`

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
- long-term semantic backend support via LightRAG or a compatible backend

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

Inspect working memory and evidence:

```bash
cargo run -p memd-client --bin memd -- working --project demo --agent codex --follow
cargo run -p memd-client --bin memd -- explain --id <uuid> --follow
```

Bootstrap a project bundle:

```bash
cargo run -p memd-client --bin memd -- init --project demo --namespace main --agent codex
```

Resume the default memory snapshot from that bundle:

```bash
cargo run -p memd-client --bin memd -- resume --output .memd
```

If the bundle has a LightRAG-compatible backend configured, `resume` also pulls
a bounded semantic recall lane and writes it into the memory files without
replacing typed `memd` working memory.

That also refreshes:

- `.memd/MEMORY.md`
- `.memd/agents/CODEX_MEMORY.md`
- `.memd/agents/CLAUDE_CODE_MEMORY.md`
- `.memd/agents/OPENCLAW_MEMORY.md`
- `.memd/agents/OPENCODE_MEMORY.md`

Persist a durable memory into the same bundle defaults:

```bash
cargo run -p memd-client --bin memd -- remember --output .memd --kind decision --content "Prefer memd resume for Codex startup."
```

Resume with a shared workspace lane:

```bash
cargo run -p memd-client --bin memd -- init --project demo --namespace main --agent codex --workspace team-alpha --visibility workspace
cargo run -p memd-client --bin memd -- resume --output .memd
```

Emit a compact shared handoff bundle for delegation or resume:

```bash
cargo run -p memd-client --bin memd -- handoff --output .memd --prompt
```

With RAG configured, `handoff` carries the same bounded semantic recall lane so
cross-agent resumes get both typed state and semantic fallback.

Write that handoff into the Obsidian workspace:

```bash
cargo run -p memd-client --bin memd -- obsidian handoff --vault ~/vault --project demo --workspace team-alpha --visibility workspace --apply --open
```

Switch between clients on the same bundle with the generated scripts:

```bash
.memd/agents/codex.sh
.memd/agents/claude-code.sh
.memd/agents/openclaw.sh
.memd/agents/opencode.sh
```

Bootstrap a project bundle with LightRAG configured:

```bash
cargo run -p memd-client --bin memd -- init --project demo --agent codex --rag-url http://127.0.0.1:9000
```

When you are still building the memory loop, keep `memd` as the source of
truth and treat LightRAG as an optional semantic backend behind the control
plane.

Check bundle health:

```bash
cargo run -p memd-client --bin memd -- status --output .memd
```

When the server is reachable, `status` also includes a lightweight resume
preview so you can see whether the default memory lane is actually returning
working records, inbox items, workspace lanes, and semantic hit count.

Print the attach snippet that points agents at the bundle-backed resume flow:

```bash
cargo run -p memd-client --bin memd -- attach --output .memd
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
- [RAG](./docs/rag.md)
- [Backend Stack Contract](./docs/backend-stack.md)
- [Backend API Contract](./docs/backend-api.md)
- [Backend Ownership Split](./docs/backend-ownership.md)
- [Backend Implementation Plan](./docs/backend-implementation-plan.md)
- [Credits](./docs/credits.md)
- [Schema](./docs/schema.md)
- [Promotion Policy](./docs/promotion-policy.md)
- [Source Policy](./docs/source-policy.md)
- [Redundancy Policy](./docs/redundancy.md)
- [Platform Support](./docs/platforms.md)
- [Branching Model](./docs/branching.md)
- [Maintainer Workflow](./docs/maintainer-workflow.md)
- [Release Process](./docs/release-process.md)
- [Changelog](./CHANGELOG.md)
- [Code of Conduct](./CODE_OF_CONDUCT.md)
- [Hook Kit](./integrations/hooks/README.md)
- [Hook Kit Installers](./integrations/hooks/README.md)
- [OpenClaw Integration](./integrations/openclaw/README.md)
- [Obsidian Vault Bridge](./docs/obsidian.md)
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
- Obsidian vault bridge with round-trip sync and watch mode
- Shared hook kit for shell integration
- Optional LightRAG adapter
- External backend stack contract

## Status

The project is usable and under active development.

## License

AGPLv3. See [LICENSE](./LICENSE).
