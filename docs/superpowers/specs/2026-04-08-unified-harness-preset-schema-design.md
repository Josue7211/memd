# Unified Harness Preset Schema Design

## Background
Memd now has multiple harness packs: Codex, OpenClaw, Hermes, OpenCode, and Agent Zero. They already share the same underlying memory plane, compiled pages, cache model, and routing primitives. The remaining problem is drift: each pack has grown its own wording and surface files, even though the behavior should come from the same core loop.

The goal of this slice is to collapse pack-specific behavior into a single preset schema so the harnesses become thin policy layers instead of forked systems.

## Goal
Create one shared harness preset schema that generates the pack surfaces, scripts, and docs for all harnesses from the same source of truth.

## Non-Goals

- Do not redesign the memory control plane.
- Do not change the compiled memory model.
- Do not introduce a second cache or another truth source.
- Do not flatten meaningful pack differences into one generic blob.

## Core Idea
Each harness should be defined by a data preset that describes:

- pack ID
- human name
- primary verbs
- startup/wake path
- resume/handoff language
- capture/spill policy
- cache policy
- generated surfaces
- test expectations

The preset is the behavior contract. The codebase then renders docs, readmes, entrypoints, and tests from that contract.

## Proposed Structure

### Shared Core
The core memd engine keeps ownership of:

- scope routing
- turn cache
- compiled memory pages
- handoff / resume / capture plumbing
- semantic backend sync
- visible truth surfaces

### Harness Presets
Each pack becomes a small preset with only behavior defaults and copy:

- Codex: strongest recall/capture automation, turn-scoped cache, wake/resume/checkpoint emphasis
- OpenClaw: compact context, spill-first, low-noise flow
- Hermes: onboarding-first wake surface, friendly defaults, easy activation
- OpenCode: explicit continuity verbs and shared-lane handoff behavior
- Agent Zero: minimal ceremony and zero-friction first-run path

## Preset Schema

The preset should minimally describe:

- `pack_id`
- `display_name`
- `entrypoint`
- `surface_set`
- `default_verbs`
- `cache_policy`
- `copy_tone`
- `generated_files`
- `test_cases`

The schema should be readable as data, not code. The schema may live as Rust structs first, but the long-term shape should be serializable and generator-friendly.

## Generated Surfaces

The following files should be rendered from the shared schema:

- `integrations/<pack>/README.md`
- `crates/memd-client/src/harness/*.rs` pack metadata
- `docs/setup.md` pack sections
- `docs/api.md` pack sections
- `docs/oss-positioning.md`
- command/help text for pack entrypoints
- pack index output

The generated content should keep pack names and defaults aligned everywhere.

## Behavior Boundaries

What is shared:

- memory state
- routing
- event capture
- cache behavior
- artifact generation

What is preset-specific:

- command wording
- startup emphasis
- default flow order
- pack-specific surface names

This keeps the system one product while still letting each harness feel native.

## Quality Rules

The preset schema is good if:

- pack docs and runtime behavior stop drifting
- adding a new harness means adding data, not copy-pasting a new subsystem
- all packs reuse the same core loop
- changes to the core fan out to every pack automatically
- pack differences stay limited to defaults and messaging

The schema is bad if:

- each pack still needs its own custom flow logic
- generated files diverge from runtime behavior
- pack descriptions become hand-maintained duplicates
- the schema is too weak to express entrypoint and surface differences

## Acceptance Criteria

This design is complete when:

- there is one shared preset contract for all harnesses
- Codex, OpenClaw, Hermes, OpenCode, and Agent Zero can all be described by it
- pack docs and runtime pack metadata are generated from the same source
- no new pack needs bespoke duplicated logic for its normal surface set

## Notes

This is a consolidation move, not a behavior rewrite. The point is to keep the harnesses thin and let memd own the actual memory system.
