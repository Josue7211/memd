# memd Vision

Date: 2026-04-09

## North Star

`memd` is the memory control plane for agents.
Its job is to keep sessions continuous, memory truthful, and multi-session coordination inspectable.

The product should feel like real memory:

- it remembers the right thing
- it keeps provenance attached
- it recovers across restarts
- it distinguishes fresh truth from stale state
- it helps live sessions coordinate without collapsing into noise

## Priority Order

1. Session continuity
2. Memory truth
3. Coordination across live sessions

That order matters.

- continuity without truth becomes durable confusion
- truth without continuity becomes brittle recall
- coordination without both becomes shared noise

## Product Thesis

`memd` should be the thing that makes a multi-agent runtime feel coherent over time.

It should:

- preserve the current working state without forcing transcript replay
- keep verified facts visible and inspectable
- make stale, dead, and superseded state explicit
- let multiple live sessions share the same project without guessing
- consolidate memory continuously instead of waiting for a human cleanup pass

## Borrowed Shape From Claude Code

Claude Code is useful as a source because it shows a full terminal-first assistant runtime, not just an agent-team overlay.

Relevant ideas to borrow:

- session memory as an active maintenance loop
- background consolidation and compaction
- worktree-aware isolation for parallel work
- IDE integration as part of the live context model
- bridge / remote-control session lifecycle
- explicit capability catalogs
- task taxonomy that separates local, remote, teammate, workflow, and maintenance flows

What not to borrow blindly:

- source-map reconstruction complexity
- feature-flag sprawl without sharp ownership
- hidden state mutations
- terminal UI coupling as the core identity

## `memd` Principles

### 1. Continuity

`memd` should let a session survive interruption.
When a session resumes, the user should see the current shape of work immediately, not reconstruct it manually.

### 2. Truth

`memd` should preserve provenance, freshness, and contradiction state.
The user should be able to tell what is verified, what is stale, and what was superseded.

### 3. Coordination

`memd` should make live collaboration explicit.
If multiple sessions are active, the system should show who is current, who is live, what is stale, and what is dead.

### 4. Maintenance

`memd` should improve memory continuously.
It should consolidate, compact, and refresh in the background where possible.

### 5. Inspectability

Every memory object should be viewable, linkable, and explainable.
Every important decision should have a visible reason and a visible source.

## User Experience Target

The best `memd` session feels like this:

- open it again and the current work is still there
- the freshest truth is already on top
- old state is not confused with live state
- coordination with other sessions is obvious
- maintenance happens without asking the user to babysit it

## What `memd` Is Not

- not transcript dump tooling
- not a hidden RAG wrapper
- not a generic agent planner
- not a memory blob with a search box
- not a coordination system that hides stale state

## What To Build Toward

1. Session continuity that is immediate and reliable
2. Truth-first retrieval with explicit freshness and provenance
3. Live coordination that is visible but not noisy
4. Background compaction and consolidation loops
5. Capability catalogs and lifecycle operations that are inspectable
6. Worktree and IDE awareness as part of the runtime model

## Success Criteria

`memd` is 10-star when:

- a session can be resumed without losing the brain
- memory can be trusted because provenance and freshness are visible
- multiple live sessions can coordinate without ambiguity
- maintenance improves the system automatically
- the user can tell at a glance what is current, stale, dead, or shared

