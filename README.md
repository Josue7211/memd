# memd

`memd` is an open-source memory manager and retrieval control plane for agents.

It gives AI systems one place to store, route, compact, explain, and verify memory without turning every session into a transcript dump.

The direction is not just “store more memory.” `memd` is also being shaped to
support measured self-improvement: detect quality gaps, replay real memory and
coordination scenarios, run bounded experiments, and keep only changes that win
against explicit gates. Autoresearch now coordinates eight narrow research
loops—prompt surface compression, live truth freshness, capability-contract
detection, event-spine compaction, correction learning, long-context avoidance,
cross-harness portability, and controlled self-evolution—so every iteration
ends with a percent-improvement report, tracks expected token savings, and
stops once the metric budget for that loop is satisfied. The goal is a 90%
token-cost reduction for high-value tasks while maintaining evidence-driven
quality; accepted outcomes feed into autodream for durable consolidation.
See [docs/research-loops.md](./docs/research-loops.md) for the loop manifest,
execution guidance, and percent-improvement telemetry expectations. Use
`memd loops` (default list view), `memd loops --summary`, or
`memd loops --loop <slug>` to inspect the recorded loop artifacts inside
`.memd/loops/`. When you need an aggregated telemetry snapshot (text or JSON),
run `memd telemetry` (or `memd telemetry --json`) to read `loops.summary.json`
and surface percent-improvement and token-saving metrics directly. Drive the
loops with `memd autoresearch --manifest` (lists available loops),
`memd autoresearch --loop <slug>` (reruns a single loop), or
`memd autoresearch --auto` (sweeps every loop in manifest order); each run
writes telemetry back into `.memd/loops/`.

That loop should work together with autodream:

- autoresearch finds gaps, runs bounded experiments, and records accepted wins
- autodream consolidates those accepted learnings into durable memory and future procedure

If you are using Codex or another agent day to day, `memd` should be the
default memory substrate: retrieve working state from `memd`, inspect evidence
with `memd explain`, and write compiled knowledge back into the workspace when
the task is worth keeping.

The shortest default loop is:

- `memd resume --output .memd`
- `memd remember --output .memd --kind <kind> --content <text>`
- `memd handoff --output .memd --prompt`
- read `.memd/MEMD_MEMORY.md` or `.memd/agents/CODEX_MEMORY.md`

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

## Environment Facts

If you are making claims about tunnels, domains, VMs, or public reachability,
use [Infrastructure Facts](./docs/infra-facts.md) as the local truth source and
verify locally before stating anything as true.

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

Minimal local setup:

```bash
cargo run -p memd-server
cargo run -p memd-client --bin memd -- init --agent codex
memd status --output .memd
memd resume --output .memd --intent current_task
```

If you are initializing inside a repo, `memd init` now seeds `.memd/` from the
repo's existing docs, planning files, and memory files when it can infer a
project root. That includes sources like `AGENTS.md`, `CLAUDE.md`,
`MEMORY.md`, `memory/*.md`, and `.planning/*` when they exist. Use `--global`
when you want `~/.memd` instead. If you need to seed a different repo, pass
`--project-root <path>`.

Inside Codex, the default entrypoint is:

- `$memd`

It automatically decides whether to initialize or reload. The explicit
`$memd-init` and `$memd-reload` skills are there if you want to force one path.

If you are using the shared OpenClaw server instead of a local server, you
must reach it over Tailscale or another private VPN/private network and export
`MEMD_BASE_URL=http://100.104.154.24:8787` before running the same commands.

What that gives you:

- a server at `MEMD_BASE_URL` (default shared Tailscale endpoint for the bundled
  OpenClaw deployment, or a local explicit override for dev)
- a bundle at `.memd/`
- a readiness check through `status`
- a compact current-task resume path through `resume`
- a dedicated heartbeat lane in `.memd/config.json` and `.memd/env` so shared presence stays on a lightweight model

Use `status --output .memd` first when setup feels wrong. It reports:

- `setup_ready`
- missing bundle files
- backend reachability
- a lightweight hot-lane resume preview

`memd` also pins a separate `heartbeat_model` in the bundle. That lane is meant
for the low-cost presence/coordination loop, not your main reasoning model.

Optional next steps:

- store a durable memory:
  - `cargo run -p memd-client --bin memd -- remember --output .memd --kind decision --content "Prefer memd resume for Codex startup."`
- capture short-term task state:
  - `cargo run -p memd-client --bin memd -- checkpoint --output .memd --content "Current blocker: workspace handoff still needs better ranking."`
- launch a generated agent profile:
  - `.memd/agents/codex.sh`
  - `.memd/agents/claude-code.sh`
  - `.memd/agents/openclaw.sh`
  - `.memd/agents/opencode.sh`

If you want the longer bundle, Obsidian, eval, gap, and improve workflow, use
[docs/setup.md](./docs/setup.md).

## Project Docs

- [Architecture](./docs/architecture.md)
- [API](./docs/api.md)
- [Config Guide](./docs/config.md)
- [Setup Guide](./docs/setup.md)
- [Verification Feature Registry](./docs/verification/FEATURES.md)
- [Verification Runbook](./docs/verification/RUNBOOK.md)
- [Milestone v1 Audit](./docs/verification/milestones/MILESTONE-v1.md)
- [Milestone v2 Audit](./docs/verification/milestones/MILESTONE-v2.md)
- [Milestone v3 Audit](./docs/verification/milestones/MILESTONE-v3.md)
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

The server defaults to `127.0.0.1:8787` when you run a local instance.
Shared deployments are expected to live behind Tailscale or an equivalent
private VPN/private network, with `MEMD_BASE_URL` pointed at the service
endpoint. Set `MEMD_DB_PATH` to change the SQLite database location.

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
