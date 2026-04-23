# Omegon + Smriti Teardown for memd

## Pass-Gate Summary

- strongest idea: runtime DB plus tracked transport file, compiled startup/status surface, and compact claims
- wrong idea: flattening memd into either a generic harness memory system or a checkpoint-only backend
- memd overlap:
  - truth-first memory substrate
  - hive and multi-session coordination
  - wake packet and state surfaces
- direct lift targets:
  - `K2` observability
  - `L2` hive hardening
  - `J2` isolation + trust
- judgment: `steal now`

## Why This Exists

We need a clean answer to three questions:

- what `memd` already does
- what `Omegon` does better in implementation
- what `Smriti` does better in coordination

This is not a branding comparison.

It is a donor analysis for `memd`.

## What We Inspected

Remote repos inspected:

- `https://github.com/styrene-lab/omegon`
- `https://github.com/himanshudongre/smriti`

Omegon docs and code read:

- `README.md`
- `docs/project-memory.md`
- `docs/multi-instance-coordination.md`
- `.gitattributes`
- `core/crates/omegon/src/status.rs`
- `core/crates/omegon/src/setup.rs`

Smriti docs and code read:

- `README.md`
- `ARCHITECTURE.md`
- `backend/app/api/routes/claims.py`
- `backend/app/api/routes/chat.py`
- `cli/smriti_cli/formatters.py`

Relevant memd docs checked during comparison:

- `README.md`
- `docs/strategy/live-truth.md`
- `docs/backlog/2026-04-13-memd-no-cross-session-codebase-memory.md`
- `docs/backlog/2026-04-13-lane-architecture-gaps.md`

## Hard Findings

### 1. memd is stronger in theory than both donors

`memd` is ahead on:

- live truth precedence
- typed memory kinds
- correction as overwrite of stale belief
- provenance and evidence paths
- multiharness ownership model

This is real advantage, not wishful positioning.

`Omegon` is closer to:

- a very capable harness with memory

`Smriti` is closer to:

- a structured checkpoint and coordination backend

`memd` is trying to be:

- the cross-harness memory substrate with truth control

That core direction is still right.

### 2. Omegon is the better implementation donor

The strongest `Omegon` idea is not its branding.

It is the runtime shape:

- local runtime DB
- startup assembler
- one observable status surface
- tracked transport file for sync
- worktree-aware operator model

This is especially relevant because `memd` is already Rust.

Language match matters:

- lower translation cost
- easier structural borrowing
- easier performance and storage comparison
- easier direct module imitation

### 3. Omegon has the cleanest storage and sync pattern to steal

The most directly reusable pattern is:

- runtime truth in SQLite + WAL
- git transport in tracked JSONL
- import dedup by content hash
- union merge on the transport file

This is exactly the kind of practical surface `memd` still needs to harden.

Important limit:

- do not copy Omegon's flatter memory model as memd theory

What to copy is:

- transport and runtime split

What not to copy is:

- ontology flattening

### 4. Omegon productizes startup and operator state better than memd

`Omegon`'s `setup.rs` and `status.rs` show a tight pattern:

- detect repo root
- load memory backend
- import transport if DB is empty
- register runtime features
- assemble one status object
- expose that state to TUI and dashboard

This is one of memd's current weak spots.

`memd` theory already says:

- read once
- compile compact truth
- resume fast

But runtime surfaces are still more fragmented than the doctrine.

### 5. Omegon is stronger on worktree-based parallelism

`Omegon` gets an important systems truth right:

- mutable parallel work means worktrees

That is better than pretending coordination alone solves file collision.

This matters for:

- hive
- parallel harness sessions
- future multi-agent execution

Takeaway:

- memd should treat worktree isolation as the physical boundary
- memory and claims should sit on top of that, not replace it

### 6. Smriti has the best minimal coordination primitives

`Smriti` contributes the cleanest small set of coordination objects:

- claim
- intent type
- optional task id
- TTL
- branch state
- branch divergence signal
- freshness check

This is a very good fit for `memd` because these are:

- lightweight
- inspectable
- additive

They do not require `memd` to surrender its deeper theory.

### 7. Smriti's isolation model is worth stealing directly

The checkpoint mount / restore boundary based on sequence numbers is strong.

It gives:

- exact history cutoff
- clean restore semantics
- reduced branch bleed

That is better than vague "resume from checkpoint" semantics.

`memd` should likely copy this idea for:

- restore
- fork
- session continuity boundaries

### 8. Smriti is narrower than memd and should stay that way

Do not collapse `memd` into a checkpoint backend.

`Smriti` is valuable because it is disciplined and small.

But its scope is still narrower:

- less memory typing
- weaker correction doctrine
- less emphasis on canonical truth
- less emphasis on cross-harness memory ownership

So the right move is:

- steal coordination surfaces
- reject scope shrinkage

### 9. memd already overlaps a lot with both donors

`memd` already has or claims:

- wake packet / startup briefing
- typed memory
- session continuity
- optional semantic backend
- multi-session identity
- hive coordination concepts
- atlas navigation layer

So the gap is not mostly missing ideas.

The gap is:

- tighter runtime packaging
- smaller number of obvious commands
- stronger operator-visible coordination surfaces

### 10. The biggest donor win is implementation discipline, not new theory

After mining both repos, the main conclusion is:

- `memd` does not need a new memory philosophy from them

It needs:

- better compiled runtime surfaces from `Omegon`
- better minimal coordination primitives from `Smriti`

## What memd Should Steal From Omegon

### Steal 1: Runtime DB plus tracked transport file

Keep:

- runtime state in database
- branch/share transport in tracked file
- content-hash dedup on import

Apply to:

- promoted facts
- canonical truth exports
- portable memory snapshots

### Steal 2: One startup assembler

`memd` needs one place that resolves:

- repo root
- active workspace
- memory health
- active runtime state
- integration state
- wake bundle generation inputs

### Steal 3: One observable state surface

`memd state` should become:

- the canonical operator brief

Not just another helper command.

It should include:

- live truth
- current focus
- active workspace or hive session
- memory health
- coordination state
- freshness warnings

### Steal 4: Worktree-first parallel execution posture

Parallel mutable work should assume:

- separate worktrees

Then layer memd coordination on top.

### Steal 5: Namespace-scoped memory over shared physical storage

This maps well onto:

- lane-scoped memory
- branch-scoped working memory
- session-scoped overlays

## What memd Should Steal From Smriti

### Steal 1: Claims

Add a compact claim object with:

- agent
- branch or workspace
- scope
- task_id
- intent_type
- ttl
- status

### Steal 2: Freshness checks

Before continuing from prior memory state, memd should be able to say:

- unchanged since X
- changed since X
- here are the new checkpoints / truth updates

### Steal 3: Divergence signal

`memd state` should surface when active branches disagree on important decisions.

Not full compare output.

Just:

- enough signal to know when to inspect deeper

### Steal 4: Exact restore / fork isolation

Adopt exact history boundaries for:

- restore
- fork
- branch continuation

## What memd Should Keep Doing Better

### Keep 1: Truth-first memory

Do not relax:

- freshest verified local truth outranks stale memory

### Keep 2: Typed memory model

Do not flatten everything into:

- facts
- checkpoints

`memd` should keep:

- working
- session continuity
- episodic
- semantic
- procedural
- candidate
- canonical

### Keep 3: Correction and provenance as first-class

This is one of `memd`'s clearest advantages.

Do not trade it away for simplicity theater.

### Keep 4: Human-owned substrate posture

The memory substrate should remain:

- human-owned
- harness-routable
- inspectable

## What memd Should Reject

### Reject 1: Becoming only a harness memory system

Do not turn `memd` into an `Omegon` clone.

### Reject 2: Becoming only a checkpoint backend

Do not turn `memd` into a `Smriti` clone.

### Reject 3: Flattening ontology for implementation convenience

Implementation simplification is good.

Theory collapse is not.

## Recommended Implementation Order

### 1. Omegon-style runtime and transport split

Build or harden:

- runtime DB
- tracked export file
- import dedup

### 2. Omegon-style startup and status assembly

Build:

- one startup compiler
- one canonical state surface

### 3. Smriti-style claims

Build:

- claim create
- claim list
- claim close

### 4. Smriti-style freshness and divergence

Extend `memd state` with:

- freshness
- active branch summaries
- divergence signal

### 5. Omegon-style worktree-aware parallel posture

Treat:

- worktree as physical isolation
- memd as memory and coordination layer above it

### 6. Smriti-style restore / fork isolation

Make branch boundaries exact, not fuzzy.

## Bottom Line

`memd` has the better brain.

`Omegon` has the better body.

`Smriti` has the cleaner coordination knobs.

The correct move is not to copy either system wholesale.

The correct move is:

- keep memd doctrine
- steal Omegon runtime discipline
- steal Smriti coordination primitives
