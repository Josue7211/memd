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

Success:

- long-term memory is searchable and evidence-backed without becoming the hot path

### Phase 6: Client Integrations

Deliver:

- Codex adapter
- Claude Code adapter
- Mission Control integration
- OpenClaw integration

Success:

- every client can request compact context and submit durable memory candidates

### Phase 7: Freshness and Contradictions

Deliver:

- verification jobs
- background verification worker
- staleness decay
- supersession chains
- contradiction resolution

Success:

- stale or contradicted memories stop poisoning retrieval

### Phase 8: Graph and Learning

Deliver:

- entity and relationship layer
- retrieval feedback loops
- adaptive ranking by agent and task

Success:

- memory becomes graph-aware and self-improving

## Immediate Next Steps

1. Lock architecture and schema.
2. Scaffold the Rust workspace.
3. Implement `store`, `search`, and `context` APIs.
4. Add short-term sync support.
5. Add dream-to-candidate promotion.

## Non-Goals

- transcript dumping
- vendor lock-in
- using RAG as the only memory layer
- mixing project-local truth with global truth
