# supermemory Teardown for memd

## Pass-Gate Summary

- strongest idea: one memory API served through thin harness adapters plus a reusable graph surface
- wrong idea: collapsing truth, memory, RAG, connectors, and product claims into one flat ontology
- memd overlap:
  - harness-routable memory substrate
  - dashboard and graph ambitions
  - profile and recall surfaces
- direct lift targets:
  - `N2` integrations polish
  - `I2` human dashboard
  - `E2` atlas activation
- judgment: `reference only`

## Why This Exists

We need a direct answer to three questions:

- what `supermemory` productized well
- what `memd` should port as runtime surface
- what `memd` should reject because it weakens truth control

This is a donor analysis, not a product comparison.

## What We Inspected

Local repo inspected:

- `/home/josue/Documents/projects/supermemory`

Docs and code paths read:

- `README.md`
- `packages/ai-sdk/README.md`
- `packages/tools/src/tools-shared.ts`
- `packages/memory-graph/README.md`
- `packages/memory-graph/src/types.ts`
- `packages/memory-graph/src/hooks/use-graph-data.ts`

Relevant memd docs checked during comparison:

- `README.md`
- `ROADMAP.md`
- `docs/strategy/live-truth.md`
- `docs/backlog/2026-04-14-steal-from-inspiration-repos.md`

## Hard Findings

### 1. supermemory is strongest at packaging, not memory doctrine

The best part is not the marketing line.

The strongest implementation pattern is:

- one hosted memory API
- thin adapters for many harnesses
- shared tool descriptions and scoping rules
- a reusable UI graph package

This matters because `memd` already has deeper theory than `supermemory`.

What `supermemory` does better is:

- shipping one surface many clients can consume

### 2. The adapter layer is directly useful to memd

`packages/ai-sdk/README.md` and `packages/tools/src/tools-shared.ts` show a clean posture:

- one standard config surface
- common container scoping
- shared tool descriptions
- minimal wrapper code per harness

This is worth copying for `memd` because current integrations are still uneven.

The direct lesson is:

- keep one core API
- keep adapter packages thin
- centralize tool semantics once

### 3. Container tags are practical, but too flat for memd truth

`supermemory` uses:

- `containerTags`
- `projectId`
- static memories
- dynamic memories
- search results

This is useful for:

- scoping
- product ergonomics
- quick API adoption

It is not enough for `memd` as a truth system because `memd` still needs:

- typed memory kinds
- correction lineage
- canonical overwrite rules
- local truth precedence

Takeaway:

- steal scoping ergonomics
- reject ontology flattening

### 4. The graph package is a real donor for atlas activation

`packages/memory-graph` is implementation-specific in the right way:

- typed document and memory nodes
- explicit edge kinds
- deterministic initial layout
- canvas/WebGL-oriented rendering
- variants for console vs consumer embedding

That maps cleanly onto `memd`'s dormant atlas problem.

Important limit:

- the graph is a navigation surface
- it is not a truth engine

This matches `memd` doctrine well.

### 5. supermemory separates UX surface from backend complexity well

The graph package and SDK docs both show the same discipline:

- small public surface
- more backend complexity hidden behind it

`memd` needs more of that.

Today `memd` often exposes internal architecture names before it exposes one obvious operator path.

The donor lesson is:

- compile internal depth into one simple surface

### 6. supermemory over-centralizes around a hosted product posture

This is the main wrong idea for `memd`.

`supermemory` assumes:

- hosted API first
- one product ontology
- one vendor surface doing memory, RAG, connectors, and profiles together

That is fine for their business.

It is not right for `memd`, which should remain:

- human-owned
- inspectable
- local-first when truth matters
- multi-harness instead of vendor-anchored

### 7. The profile split is useful as a presentation layer

The `static` plus `dynamic` profile split is a good operator-facing view.

`memd` should copy this only as a projection layer for:

- current stable facts
- recent active context

Do not downgrade the internal memory model to just those buckets.

### 8. supermemory proves graph UX must be productized, not left dormant

Their graph package exists as a reusable package with:

- explicit types
- reusable rendering
- documented install path

`memd` atlas has theory, routes, and code, but still lacks this productized surface.

Takeaway:

- atlas must become an actively surfaced operator tool

## What memd Should Steal

### Steal 1: Thin harness adapter packages

Build one `memd` core memory API, then layer:

- Claude Code adapter
- Codex adapter
- OpenAI SDK adapter
- MCP adapter

Keep behavior centralized.

### Steal 2: Shared tool descriptions and scoping helpers

Have one source of truth for:

- tool semantics
- memory scope rules
- adapter defaults

This reduces drift across harnesses.

### Steal 3: Atlas as a reusable product surface

Promote atlas into:

- reusable UI module
- explicit node and edge types
- embed-ready dashboard surface

### Steal 4: Presentation-level stable vs recent profile view

Expose:

- stable facts
- recent context

But keep full `memd` internal typing underneath.

## What memd Should Reject

### Reject 1: Flat ontology as the source of truth

Do not reduce `memd` memory into:

- static
- dynamic
- search results

Those are views, not truth categories.

### Reject 2: Hosted-first dependency for canonical truth

Do not make `memd` require:

- vendor API round-trips
- opaque hosted state

for trust-critical memory.

### Reject 3: Product sprawl inside the truth layer

Do not merge:

- memory
- RAG
- connectors
- profiles
- dashboard concerns

into one inseparable substrate.

## Recommended Placement in memd Roadmap

### 1. `N2` Integrations Polish

Port:

- thin adapter packages
- shared tool contract
- scope helpers

### 2. `E2` Atlas Activation

Port:

- explicit graph node and edge contracts
- reusable graph surface
- operator-visible navigation

### 3. `I2` Human Dashboard

Port:

- stable vs recent profile projection
- embed-ready atlas surface

## Bottom Line

`supermemory` is a strong packaging donor.

It is not the right truth model donor.

`memd` should copy:

- adapter discipline
- graph productization
- simple public surfaces

`memd` should reject:

- ontology flattening
- hosted-first truth
- product sprawl in the core substrate
