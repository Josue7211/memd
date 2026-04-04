# memd Roadmap

## Vision

`memd` is a universal memory substrate for agents and agent-powered applications.

It should solve memory as infrastructure:

- local working memory
- shared short-term state
- cross-project long-term knowledge
- durable retrieval with evidence
- lifecycle, dedupe, freshness, and contradiction handling

The target is not a feature. The target is an open-source platform.

## Current Status

`memd` is past the core platform build. The remaining roadmap is now about
making the platform feel inevitable to use:

- the core still works without RAG
- LightRAG is the intended long-term semantic backend
- project bundles make the long-term path configurable
- clients attach through the same control plane

Phase summary:

- Phase 0: done
- Phase 1: done
- Phase 2: done
- Phase 3: mostly done
- Phase 4: mostly done
- Phase 5: in progress
- Phase 6: in progress
- Phase 7: partially seeded
- Phase 8: not started

## Product Shape

`memd` is the control plane.

Backends and producers work behind it:

- local memory
- short-term sync
- auto-dream / consolidation
- semantic retrieval
- graph relationships
- verification workers

Clients consume it through one API:

- Codex
- Claude Code
- Mission Control
- OpenClaw
- generic HTTP/CLI users

## Phases

### Phase 0: Specs

Deliver:

- architecture
- schema
- promotion policy
- retrieval policy
- OSS positioning

Success:

- storage tiers are defined
- write authority is defined
- retrieval order is defined

### Phase 1: Rust Core

Deliver:

- Rust workspace
- core schema crate
- core policy crate
- basic server crate
- SQLite-backed local mode

Success:

- structured memory can be stored, searched, and expired

### Phase 2: Retrieval Layer

Deliver:

- compact context builder
- scope-aware ranking
- budgeted retrieval
- current-project first policy

Success:

- context packages are small, relevant, and deterministic

### Phase 3: Short-Term Sync

Deliver:

- synced manifests for active state
- client adapters for shared short-term state
- TTL-based short-term memory lifecycle

Success:

- active work can move across machines without becoming long-term sludge

### Phase 4: Dream Pipeline

Deliver:

- candidate-memory ingest
- project dream pass
- cross-project dream pass
- promotion gates

Success:

- dream output becomes candidate facts, not canonical truth

### Phase 5: Long-Term Memory Backend

Deliver:

- LightRAG adapter
- project namespace
- global namespace
- compact-summary plus raw-doc strategy
- optional backend configuration in the project bundle

Success:

- long-term memory is searchable and evidence-backed without becoming the hot path

### Phase 5.1: RAG Adapter Hardening

Deliver:

- stronger `memd rag` sync/search behavior
- bundle-level RAG configuration
- clear enabled/disabled/healthy state
- deterministic export of canonical memory into the semantic backend

Success:

- RAG feels like part of the product, not a sidecar script

### Phase 5.2: Backend Stack Contract

Deliver:

- explicit integration contract for `rag-sidecar`
- document `MinerU` as the document extraction dependency
- document `RAGAnything` as the multimodal retrieval dependency
- backend capability matrix for text, PDF, image, table, and equation flows

Success:

- the repo clearly distinguishes `memd` core from the external LightRAG stack

### Phase 6.1: Agent Attach Automation

Deliver:

- one-command project attach flow
- per-agent launch snippets for Claude Code, Codex, Mission Control, and OpenClaw
- automatic bundle loading for project defaults

Success:

- an agent can start a task with the right memory context without manual wiring

### Phase 7.1: Memory Quality Enforcement

Deliver:

- stronger freshness checks
- contradiction surfacing
- inbox triage for contested memories
- verification-based promotion and demotion

Success:

- memory stops drifting into stale truth

### Phase 8.1: Graph and Learning

Deliver:

- entity and relationship layer
- retrieval feedback loops
- adaptive ranking by agent and task

Success:

- memory becomes graph-aware and self-improving

## Immediate Next Steps

1. Finish bundle-first LightRAG defaults and adapter behavior.
2. Finish agent attach automation for Claude Code, Codex, Mission Control, and OpenClaw.
3. Tighten freshness and contradiction resolution.
4. Add graph-aware memory only after the core lifecycle is stable.

## Non-Goals

- transcript dumping
- vendor lock-in
- using RAG as the only memory layer
- mixing project-local truth with global truth
