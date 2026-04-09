# Centralized Runtime Spine Design

Date: 2026-04-09

## Goal

Turn `memd` into the centralized OSS memory and control plane for agent runtimes.

The system must optimize for all of these together:

- better retrieval
- lower token use
- maximum session continuity
- higher memory quality
- coworking without duplicate work

This is not a “more commands” project.
It is a runtime architecture project.

## Product Thesis

`memd` should have one compact runtime spine that every command, agent, and future UI reads from.

That spine must:

- prefer compact canonical memory before raw text
- preserve provenance and freshness
- keep session identity continuous across rebinds and restarts
- expose coworking state explicitly
- make automatic maintenance visible and overrideable

The core product rule is simple:

`memd` is not done when it can store memory.
`memd` is done when it can keep the right memory small, current, inspectable, and shared safely across sessions.

## Priority Order

1. Retrieval quality
2. Token efficiency
3. Continuity
4. Memory quality
5. Coworking

This order is intentional.

- retrieval quality without compactness becomes expensive noise
- compactness without continuity becomes brittle
- continuity without truth becomes durable confusion
- coworking without the first four becomes shared confusion at scale

## Architecture

The best architecture is a centralized runtime spine with five pillars.

### 1. Truth Spine

The truth spine is the canonical memory contract.

Each important runtime memory object must carry:

- provenance
- freshness
- confidence
- contradiction state
- superseded state
- canonical compact summary
- raw evidence pointer

Rules:

- retrieval prefers canonical compact memory first
- raw evidence is still accessible for drilldown
- stale and superseded state must be visible, not implied
- contradiction is a first-class state, not an ad hoc warning

### 2. Retrieval Spine

Retrieval must be tiered and budgeted.

The retrieval path should resolve in this order:

1. hot summary
2. working memory
3. canonical evidence
4. raw source fallback

Rules:

- token budget is an architectural constraint, not a later optimization
- retrieval should reuse compact state instead of regenerating it
- the same turn should not repeatedly recompute equivalent memory payloads
- retrieval outputs must explain why a memory item was selected

### 3. Continuity Spine

Session continuity is part of every read/write path.

Every relevant runtime surface must understand:

- bundle session
- live session
- rebased_from
- continuity events
- retirement/recovery history

Rules:

- repo-local bundles follow the live session identity when safe
- superseded stale sessions are retired explicitly
- continuity actions emit receipts
- no command should have continuity logic that only exists locally in that command

### 4. Coordination Spine

Coworking state must be explicit.

The coordination spine includes:

- session awareness
- claim ownership
- task ownership
- lifecycle state
- recoverability
- retirement state
- capability visibility

Rules:

- no coworking without explicit ownership
- stale sessions with owned work are recoverable
- stale sessions without owned work are retireable
- duplicate-work pressure must be visible

### 5. Maintenance Spine

Memory maintenance must be automatic underneath and explicit on the surface.

Maintenance includes:

- compaction
- freshness refresh
- contradiction cleanup
- stale-session retirement
- low-value memory pressure repair

Rules:

- every automatic mutation emits a receipt
- every maintenance pass can be inspected later
- maintenance is allowed to run automatically only if the outcome is visible and overrideable

## Runtime Surfaces

The runtime spine should be exposed through explicit surfaces.

### Existing surfaces to keep and strengthen

- `memd status`
- `memd awareness`
- `memd hive`
- `memd coordination`

### New or expanded surfaces

- `memd session`
  - `summary`
  - `resume`
  - `rebind`
  - `retire`
  - `reconcile`

- `memd maintain`
  - `scan`
  - `compact`
  - `refresh`
  - `repair`
  - `auto`

- `memd capabilities`
  - `summary`
  - `list`
  - `search`

- `memd tasks`
  - `summary`
  - `list`
  - `classify`

Rules:

- new commands are acceptable if they reduce surface overload
- commands must be thin views over the centralized runtime contracts
- no command gets its own private state model

## Deliverables For Today

Today’s finish line is a real shipped slice across five areas.

### Deliverable 1: Truth-first memory model

Done means:

- truth fields are explicit in persisted state
- stale/superseded/contradicting memory is surfaced in summaries
- retrieval uses truth-aware compact memory before raw fallback
- at least one CLI surface shows why a memory object is current or stale

### Deliverable 2: Memory maintenance loop

Done means:

- a real maintenance surface exists
- maintenance writes receipts and reports
- compaction/refresh/repair are visible operations
- memory gets smaller and cleaner, not just larger

### Deliverable 3: Session continuity overlay

Done means:

- live session identity is canonical
- rebinds and retirements are visible
- continuity state is shown consistently across session/status/hive paths
- continuity actions are persisted as events or receipts

### Deliverable 4: Live coordination view

Done means:

- current, active, stale, dead, and retired state are visible
- recoverable vs retireable stale sessions are differentiated
- coworking actions are explicit and bounded
- duplicate-work pressure is visible in summaries or gaps

### Deliverable 5: Capability and task runtime surfaces

Done means:

- the runtime can show what it can do right now
- tasks are visible with meaningful classification
- capabilities and tasks share the same runtime truth model
- the operator can inspect ownership and active work shape without reading raw files

## Persisted Contracts

Every one of the five deliverables must have:

- a persisted contract
- a command surface
- a summary/diagnostic rendering
- regression coverage

The persisted contracts should stay inside the existing runtime system where possible.

Preferred approach:

- extend existing bundle/runtime/session/task structures
- reuse awareness, heartbeat, receipts, and summary artifacts
- avoid adding parallel stores unless there is no credible extension path

## Anti-Drift Rules

These are hard rules.

1. One runtime truth model only
2. Retrieval must prefer compact canonical memory before raw text
3. Every automatic mutation writes a receipt
4. Every surface must be token-budget aware
5. No command gets its own private state model
6. No coworking without explicit ownership
7. No continuity logic that only exists in one command
8. No memory-quality feature without inspectability
9. No background maintenance that cannot be explained later
10. No integration-first work that bypasses the runtime spine

## What This Spec Explicitly Defers

These are important, but not required for today’s finish line:

- deep IDE-native integration
- full worktree orchestration model
- richer operator UI beyond CLI/runtime surfaces
- broad bridge/session-runner expansion beyond what the runtime spine needs today
- analytics or feature-flag layers not directly required by retrieval, continuity, memory quality, or coworking

They are not rejected.
They are deferred until the centralized runtime spine is locked.

## Verification

A feature is not complete unless all of the following are true:

- persisted contract exists
- command output exists
- automatic behavior is visible through receipts or summaries
- focused regression tests pass
- full relevant suite passes
- one real CLI path demonstrates the behavior

## Success Criteria

This slice is successful if, by the end of today:

- retrieval is more compact and more explainable
- memory quality is more explicit and less noisy
- continuity state is more reliable and more visible
- stale work is recoverable and stale noise is retireable
- operators can inspect active capabilities and task shape from runtime surfaces

## Recommendation

Implementation should proceed in this order:

1. memory maintenance loop
2. truth-first retrieval and memory model
3. continuity overlay everywhere
4. coordination and task surfaces
5. capability runtime surface

This order gives the highest leverage for token efficiency and retrieval quality while still landing the centralized coworking model on top of clean runtime truth.
