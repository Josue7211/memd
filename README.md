# memd

`memd` is a multiharness second-brain memory substrate for humans and agents.

It turns raw work into compact, source-linked memory that stays usable across
sessions, harnesses, machines, and projects. The point is not to store more
context. The point is to make memory reliable: read once, remember once, reuse
everywhere, and always keep a path back to the evidence.

## Start Here

- canonical entrypoint: [[START-HERE]]
- current project state: [[ROADMAP]]
- fresh-session recovery: [[docs/WHERE-AM-I.md|WHERE-AM-I]]
- current milestone: [[docs/verification/milestones/MILESTONE-v1.md|MILESTONE-v1]]
- current priority harnesses: Codex, OpenCode, Hermes, OpenClaw

## Architecture

Canonical board:

- [docs/assets/composite/memd-10-star-board-v1.png](./docs/assets/composite/memd-10-star-board-v1.png)
- [docs/assets/composite/memd-10-star-board-v1@2x.png](./docs/assets/composite/memd-10-star-board-v1@2x.png)

Supporting graphs:

- [topology](./docs/assets/memd-10-star-topology-v2.svg)
- [live loop](./docs/assets/memd-10-star-live-loop-v2.svg)
- [capability map](./docs/assets/memd-10-star-capability-map-v2.svg)
- [overnight loop](./docs/assets/memd-10-star-overnight-v2.svg)
- [lanes](./docs/assets/memd-10-star-lanes-v1.svg)

<img src="./docs/assets/composite/memd-10-star-board-v1.png" alt="memd 10-star architecture board" width="100%" />

## At A Glance

```mermaid
flowchart LR
  subgraph Harnesses[Harness packs]
    H1[Codex]
    H2[Claude Code]
    H3[OpenClaw / Hermes]
    H4[Future harnesses]
  end

  subgraph Ingest[Read-once ingest]
    I1[Turns, docs, artifacts, corrections]
    I2[Hooks, checkpoints, spill]
    I3[Resume and handoff packets]
  end

  subgraph Plane[memd control plane]
    P1[Working context compiler]
    P2[Session continuity]
    P3[Typed retrieval and promotion]
    P4[Correction, provenance, and authority]
  end

  subgraph Memory[Typed memory]
    M1[Working context]
    M2[Session continuity]
    M3[Episodic memory]
    M4[Semantic memory]
    M5[Procedural memory]
    M6[Candidate memory]
    M7[Canonical memory]
  end

  subgraph Surfaces[Recall surfaces]
    S1[Wake packet]
    S2[Memory atlas]
    S3[Canonical deep dive]
    S4[Raw evidence]
    S5[Obsidian workspace]
    S6[Latency briefing]
  end

  subgraph Optional[Optional semantic expansion]
    O1[Semantic recall backend]
  end

  H1 --> I1
  H2 --> I1
  H3 --> I1
  H4 --> I1
  I1 --> I2 --> I3 --> P1 --> P2 --> P3 --> P4
  P1 --> M1
  P2 --> M2
  P3 --> M3
  P3 --> M4
  P3 --> M5
  P3 --> M6
  P4 --> M7
  M7 --> S1
  M7 --> S2
  M7 --> S3
  M7 --> S4
  M7 --> S5
  P2 --> S6
  P3 -. optional .-> O1
```

## What It Is

- raw truth that stays source-linked
- typed memory kinds instead of one flat store
- session continuity that resumes real work fast
- corrections that replace stale beliefs
- provenance that lets every claim be inspected and traced
- portability across Codex, Claude Code, OpenClaw, Hermes, OpenCode, and future harnesses
- memory atlas navigation over canonical memory
- optional semantic expansion behind the control plane

## Core Loops

- capture raw work once
- compile small wake packets
- retrieve by type before giant search
- revise stale beliefs when better evidence arrives
- dream, autoresearch, and autoevolve in the background

## What It Connects

- Codex
- Claude Code
- OpenClaw
- Hermes
- OpenCode
- Obsidian
- optional semantic recall backend

## Quickstart

```bash
cargo run -p memd-server
cargo run -p memd-client --bin memd -- setup --agent codex
memd status --output .memd
memd doctor --output .memd --summary
memd commands --output .memd --summary
memd resume --output .memd --intent current_task
```

If you are using Codex, `memd` can load or reload the current bundle for you.
For an opt-in project hive, use `memd hive-project --output .memd --enable --summary`
to turn the repo on, `memd hive --output .memd --publish-heartbeat --summary` to
join the live session, and `memd hive-link` only when you need a safe link
between different projects.

## Docs

- [Setup](./docs/core/setup.md)
- [API](./docs/core/api.md)
- [Architecture](./docs/core/architecture.md)
- [10-Star Model](./docs/theory/models/2026-04-11-memd-10-star-memory-model-v2.md)
- [Obsidian Bridge](./docs/core/obsidian.md)
- [RAG](./docs/core/rag.md)
- [Efficiency](./docs/policy/efficiency.md)
- [OSS Positioning](./docs/reference/oss-positioning.md)

## Integrations

- Codex, Claude Code, OpenClaw, Hermes, OpenCode, and future harnesses
- Obsidian
- shared hook kit

## License

AGPLv3. See [LICENSE](./LICENSE).
