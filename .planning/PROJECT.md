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

Priority 1 is Codex continuity: `memd` must let Codex persist state, recover
evidence, and inspect its own working memory across sessions.

## Requirements

### Validated

- ✓ Typed memory storage and retrieval exist in the Rust core and server.
- ✓ Agents can fetch compact context and working memory through one API.
- ✓ Optional long-term semantic backend support exists behind the control plane.
- ✓ Obsidian vault ingest exists as a markdown-native knowledge source path.
- ✓ Project bundles and attach flows exist for agent integrations.
- ✓ `v1` provenance, repair, and working-memory control are complete enough to move to `v2`.

### Active

- [x] Start `v2` foundations: explicit working-memory controller semantics.
- [x] Add trust-weighted source memory and reversible compression.
- [x] Keep branchable belief lanes explicit and inspectable.
- [x] Add retrieval feedback surfaces so ranking can learn from outcomes.
- [x] Make source-trust floors affect ranking, not just inspection.
- [x] Add contradiction resolution on top of branchable belief lanes.
- [x] Make procedural and self-model memory explicit.
- [x] Add reversible compression and evidence rehydration.
- [x] Keep procedural, self-model, and source-trust surfaces explicit as `v2` grows.
- [x] Establish shared workspace and visibility lanes for `v3`.
- [x] Add resumable workspace handoff bundles for shared memory.
- [x] Add audited workspace and visibility lane correction support.
- [x] Prefer active workspace lanes during shared-memory retrieval.
- [x] Add bundle memory evaluation snapshots and regression diffs.
- [x] Add automation failure gates for bundle memory evaluation.

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
working-memory state, shared workspace lanes, and branchable belief lanes. The
repo is usable now, with `v2` complete enough to support `v3` shared-memory
work and bundle-backed Codex startup/resume flows already in place.

The product direction now explicitly includes an Obsidian compiled-wiki mode:
raw sources and derived markdown pages can live in the same workspace, with
`memd` preserving typed memory, provenance, and policy around that markdown
surface. Compiled memory/evidence pages are now a first-class Obsidian lane.
Workspace handoff pages and lane-correction repair are now in place so shared
memory can be resumed and corrected inside both the CLI and the vault. Bundle
evaluation now supports saved baselines, regression diffs, and explicit failure
gates so memory quality can drive automation instead of only human inspection.
The deployment shape is now explicitly tiered:

- Tier 1: Obsidian-only
- Tier 2: shared sync
- Tier 3: LightRAG

LightRAG stays optional for larger-scale semantic recall.

## Constraints

- **Architecture**: `memd` remains the memory control plane — cognition and planning stay outside this repo.
- **Portability**: Core binaries must work across Linux, macOS, and Windows.
- **Evidence**: Durable memory must preserve provenance, trust, and contradiction state.
- **Efficiency**: Retrieval stays bounded by explicit token/character budgets.
- **Compatibility**: External backend stack changes must stay behind `rag-sidecar`.
- **Workflow**: milestone work should land on `work/<milestone>` branches and large slices should move on scoped `feat/<area>` branches instead of one long-lived catch-all branch.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Keep `memd` as the memory substrate, not the whole brain | Prevents cognition and storage concerns from smearing together | ✓ Good |
| Organize future work by capability versions `v1`-`v5` | Better matches the architecture jump than endless phase numbering | ✓ Good |
| Keep the external multimodal stack behind `rag-sidecar` | Preserves portability and clean backend boundaries | ✓ Good |
| Treat Obsidian as a first-class markdown workspace, not just an import source | Supports compiled-wiki workflows without forcing semantic backend dependency at small scale | ✓ Good |
| Treat working memory as a managed buffer, not just compact retrieval | Needed for eventual superhuman short-term memory | — Pending |

---
*Last updated: 2026-04-05 after GSD phase 17 completion*
