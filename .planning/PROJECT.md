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
- ✓ `v1` provenance, repair, and working-memory control are complete enough to move to `v2`.

### Active

- [x] Start `v2` foundations: explicit working-memory controller semantics.
- [x] Add trust-weighted source memory and reversible compression.
- [x] Keep branchable belief lanes explicit and inspectable.
- [x] Add retrieval feedback surfaces so ranking can learn from outcomes.
- [x] Make source-trust floors affect ranking, not just inspection.
- [x] Add contradiction resolution on top of branchable belief lanes.
- [ ] Make procedural and self-model memory explicit.
- [ ] Keep procedural, self-model, and source-trust surfaces explicit as `v2` grows.

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
policy inspection, explicit repair, provenance drilldown, managed
working-memory state, and branchable belief lanes. The repo is usable now, and
`v2` is active with explicit trust, artifact, sibling-branch, retrieval-feedback,
and trust-weighted ranking surfaces.

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
*Last updated: 2026-04-04 after GSD phase 1 completion*
