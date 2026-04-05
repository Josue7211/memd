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

The product bar is higher than “memory features exist.” `memd` should feel like
real memory:

- zero-friction memory
- epistemic memory
- short-term-first memory
- native multi-agent interoperability
- inspectable memory
- measured self-improvement
- harness-aware portable learning
- native dream/autodream lifecycle

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
- [x] Turn evaluation findings into concrete operator recommendations.
- [x] Keep short-term resume and handoff fast by making semantic fallback opt-in.
- [x] Add a dedicated short-term checkpoint command for current-task memory.
- [x] Refresh bundle memory files automatically after short-term checkpoints.
- [x] Bias default bundle launch flows toward current-task memory.
- [x] Align bundle status preview with the current-task hot path.
- [x] Align hook-context defaults with the current-task hot path.
- [x] Use memd-specific bundle memory filenames that do not collide with agent-native memory files.
- [x] Add Claude-native `CLAUDE.md` import bridging on top of bundle memory files.
- [x] Persist short-term resume deltas and surface them through prompt, bundle, and status views.
- [x] Refresh bundle memory files immediately after durable `remember` writes.
- [x] Make one-line `resume --summary` reflect the active hot lane instead of only counts.
- [x] Capture meaningful short-term coordination transitions automatically.
- [x] Make retrieval behavior prefer verified canonical evidence over unverified synthetic continuity.

### Next Product Priorities

- [x] Expose the brokered coordination substrate through a first-class peer MCP surface.
- [x] Expand peer coordination from MCP primitives into richer shared-task orchestration.
- [x] Add a compact coordination inbox that merges peer messages, task ownership, and session pressure.
- [x] Add safe stale-session recovery so blocked coworking lanes can be reclaimed without ownership drift.
- [x] Add explicit coordination policy so exclusive-write and collaborative lanes are distinguishable before conflict.
- [x] Add advisory branch and scope recommendations so simultaneous sessions split work more cleanly.
- [x] Add compact coordination receipts so coworking transitions stay inspectable over time.
- [x] Add cleaner dashboard/history views so operators can inspect live and recent coordination state faster.
- [x] Add coordination drilldown and filter views so operators can isolate the right sessions, requests, and receipts faster under load.
- [x] Add live coordination watch and alert views so operators can keep pressure visible during active coworking instead of polling manually.
- [x] Add coordination subscriptions and hook-friendly change feeds so other agent and operator surfaces can react to pressure without reimplementing polling.
- [x] Add UI-friendly coordination feed surfaces so richer operator tools can consume the same bounded change model without custom reshaping.
- [x] Add coordination action surfaces so richer operator tools can act on bounded coordination signals through the same model.
- [ ] Add policy-aware coordination action suggestions so richer operator tools can see the best bounded next move under current pressure.
- [ ] Add a measured research loop that can detect memory and coordination gaps on its own.
- [ ] Move dream, autodream, and autoresearch into native `memd` subsystems instead of leaving them as wrapper-only skills.
- [ ] Add scenario harnesses for real memory workflows so self-improvement has stable targets.
- [ ] Add a composite scorer for correctness, memory quality, coordination quality, latency, and bloat.
- [ ] Add a bounded experiment runner that accepts only winning changes and discards regressions.
- [ ] Consolidate accepted experiment learnings into durable memory and autodream inputs.
- [ ] Make autoresearch and autodream work as one loop: research first, consolidation second.
- [ ] Track strengths, weaknesses, compatibility, and portability class per harness when promoting learned skills, CLIs, and procedures.
- [ ] Keep short-term memory sharp without transcript bloat or stale carryover.
- [ ] Expand epistemic retrieval beyond verified vs synthetic toward explicit inferred, claimed, stale, and contested routing behavior.
- [ ] Make multi-agent switching feel like changing terminals, not losing the brain.
- [ ] Expand inspectability from bundle files into richer workspace and UI surfaces.

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
It now also emits concrete corrective recommendations derived from live resume
state, so weak memory signals can map directly to next actions.
Short-term memory is now also protected as the default hot path: bundle-backed
resume and handoff stay local and fast unless semantic fallback is explicitly
requested. Current-task state can also be captured through a dedicated
checkpoint flow instead of forcing operators to handcraft full memory writes,
and those short-term writes now refresh the visible bundle memory files
immediately. The default attach and agent launch surfaces now also resume with
`current_task` intent so the short-term lane is the default starting point, and
bundle status now previews that same lane. The installed hook-context flow now
uses that same current-task default. The shared memd root file is now
`MEMD_MEMORY.md` instead of a generic `MEMORY.md`, so it does not collide with
agents that already own that filename.
The deployment shape is now explicitly tiered:

- Tier 1: Obsidian-only
- Tier 2: shared sync
- Tier 3: LightRAG

LightRAG stays optional for larger-scale semantic recall.
The first peer MCP bridge is now also in place under `integrations/mcp-peer/`,
so brokered messages, claims, assignments, and inbox flows can be consumed as
agent-native coordination tools instead of only CLI calls.

The product direction is now explicit:

- short-term memory must feel instant
- long-term retrieval must stay evidence-backed
- dream and autodream should consolidate signal instead of creating bloat
- native agent memory surfaces should bridge cleanly into `memd` without ownership collisions
- the eventual UX should make memory inspectable enough to trust
- autoresearch should find quality gaps, run bounded experiments, and only keep measured wins
- autodream should consolidate accepted autoresearch outputs into durable memory instead of mixing wins with discarded experiments
- dream, autodream, and autoresearch should live in the `memd` lifecycle, with skills and wrappers acting only as entrypoints
- promoted learning should live in the substrate, but each abstraction must be marked portable, harness-native, or adapter-required so harness-specific capabilities stay honest

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
| `memd` must feel like real memory, not memory tooling | Product success depends on low-friction continuity, inspectability, and truthfulness | — In Progress |

---
*Last updated: 2026-04-05 after shipping the first `v5` peer MCP bridge*
