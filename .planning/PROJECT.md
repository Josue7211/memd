# memd

## What This Is

`memd` is an open-source memory manager and retrieval control plane for agents.
It gives agent systems a typed, inspectable place to store, route, compact,
verify, and explain memory without collapsing everything into transcript
history. The current direction is to turn `memd` into the memory substrate that
future systems like `braind` can treat as a real memory OS.

## Core Value

Give agents short-term and long-term memory that stays compact, durable,
inspectable, and useful under real task pressure.

## Requirements

### Validated

- ✓ Typed memory storage and retrieval exist in the Rust core and server.
- ✓ Agents can fetch compact context and working memory through one API.
- ✓ Optional long-term semantic backend support exists behind the control plane.
- ✓ Project bundles and attach flows exist for agent integrations.

### Active

- [ ] Finish `v1` repair actions for stale, contested, and malformed memory.
- [ ] Add provenance drilldown from compact summaries to raw source artifacts.
- [ ] Strengthen working-memory admission, eviction, and rehydration behavior.
- [ ] Make procedural, self-model, and source-trust memory more explicit.
- [ ] Prepare `v2` superhuman-memory primitives without pretending `v1` is done.

### Out of Scope

- Generic transcript dumping — it destroys signal and token efficiency.
- Vendor-locked semantic storage — `memd` stays portable above the backend.
- Treating RAG as the whole product — the control plane remains primary.
- Rebuilding the larger cognition stack inside this repo — that belongs in `braind`.

## Context

This is a brownfield Rust workspace with crates for schema, core compaction,
server, client, worker, multimodal ingest, sidecar contract, and RAG adapter.
The repo already has strong docs under `docs/`, integration assets under
`integrations/`, and an evolving top-level `ROADMAP.md` organized around
capability versions `v1` through `v5`.

Recent work tightened bundle-first backend wiring, sidecar metadata fidelity,
policy inspection, and managed working-memory state. The repo is usable now,
but the remaining `v1` gap is quality of control, repair, and provenance rather
than raw feature count.

## Constraints

- **Architecture**: `memd` remains the memory control plane — cognition and planning stay outside this repo.
- **Portability**: Core binaries must work across Linux, macOS, and Windows.
- **Evidence**: Durable memory must preserve provenance, trust, and contradiction state.
- **Efficiency**: Retrieval stays bounded by explicit token/character budgets.
- **Compatibility**: External backend stack changes must stay behind `rag-sidecar`.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Keep `memd` as the memory substrate, not the whole brain | Prevents cognition and storage concerns from smearing together | ✓ Good |
| Organize future work by capability versions `v1`-`v5` | Better matches the architecture jump than endless phase numbering | ✓ Good |
| Keep the external multimodal stack behind `rag-sidecar` | Preserves portability and clean backend boundaries | ✓ Good |
| Treat working memory as a managed buffer, not just compact retrieval | Needed for eventual superhuman short-term memory | — Pending |

---
*Last updated: 2026-04-04 after GSD brownfield initialization*
