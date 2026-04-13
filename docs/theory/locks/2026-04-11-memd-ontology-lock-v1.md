# memd Ontology Lock v1

## Purpose

This document locks the core ontology for the `memd` second-brain model.

Goal:

- define the native memory objects
- define allowed transitions
- reduce theory drift before implementation

## Root Distinction

There are four different classes of things:

1. **memory kinds**
2. **control functions**
3. **surfaces**
4. **backends/helpers**

These must not be mixed.

## 1. Memory Kinds

### Working Context

Definition:

- the smallest active reasoning packet needed now

Contains:

- current task
- active constraints
- active hypotheses
- immediate next move

Properties:

- tiny
- volatile
- action-oriented

### Session Continuity

Definition:

- the resume layer for active work

Contains:

- where we left off
- open loops
- blockers
- branch/workspace state
- next recommended action

Properties:

- recent
- resumable
- higher priority than generic episodic recall on session wake

### Episodic Memory

Definition:

- memory of events, experiences, and sequences in time

Contains:

- event
- timestamp
- context
- outcome

Properties:

- timeline-aware
- source-linked
- not all episodes must become canonical

### Semantic Memory

Definition:

- stable truths, facts, and constraints

Contains:

- decisions
- facts
- architecture truths
- policy truths

Properties:

- should be corrected when proven wrong
- should remain inspectable

### Procedural Memory

Definition:

- memory of how to operate

Contains:

- workflows
- operating patterns
- recovery patterns
- reusable tactics
- learned preferences that affect execution

Properties:

- action-oriented
- reusable
- promoted from repeated or validated success

### Candidate Memory

Definition:

- staging zone for signal not yet trusted enough for durable promotion

Contains:

- candidate facts
- candidate procedures
- candidate summaries
- repeated patterns

Properties:

- non-canonical
- promotion-eligible
- expiration-friendly

### Canonical Memory

Definition:

- durable trusted memory surface

Contains promoted:

- semantic memory
- episodic memory worth keeping
- procedural memory

Properties:

- durable
- source-linked
- controlled by provenance/correction rules

## 2. Control Functions

### Correction + Provenance

Definition:

- the trust system over memory

Tracks:

- source
- confidence
- freshness
- conflict state
- correction state
- promotion history

### Wake Packet Compiler

Definition:

- the system that compiles memory into action-ready packets

### Hive Coordination

Definition:

- the system that manages shared truth, handoff, and per-agent isolation

## 3. Surfaces

### Wake Packet

- compiled action-ready memory slice

### Memory Atlas

- multidimensional navigation layer over canonical memory

### Canonical Deep Dive

- deeper trusted context surface

### Raw Evidence

- source anchor surface

### Obsidian Workspace

- human-readable workspace and source lane

## 4. Backends / Helpers

These are optional implementation helpers, not the model:

- semantic retrieval backend
- embeddings
- graph storage
- vector index
- sync backend

## Allowed Transitions

### Live Path

1. raw event enters system
2. working context updates
3. session continuity updates
4. episodic memory record written
5. semantic memory updated if truth changed
6. procedural memory updated if reusable procedure proved out
7. wake packet recompiled

### Promotion Path

1. repeated or validated signal enters candidate memory
2. candidate reviewed by correction/provenance rules
3. accepted signal promoted into canonical memory

### Correction Path

1. correction arrives
2. provenance state updates
3. stale semantic truth is superseded
4. procedural memory updated if needed
5. wake packet updates

### Recall Path

1. wake packet first
2. atlas/deep dive second
3. raw evidence last

## Disallowed Collapses

The system must not collapse:

- semantic memory into episodic memory
- procedural memory into semantic memory
- atlas into canonical memory
- Obsidian into control plane
- semantic backend into truth layer

## Locked Decisions

### D1. Canonical Memory

Canonical memory should remain a **distinct durable surface**.

Why:

- recall needs a trusted durable target
- promotion needs a visible destination
- users and agents need a clear truth-preferred layer

### D2. Candidate Memory

Candidate memory should remain one top-level staging layer with typed internal lanes.

This keeps:

- top-level simplicity
- type-level precision

### D3. Session Continuity

Session continuity should remain a **distinct memory kind**.

It may draw from episodic memory, but it should not collapse into episodic memory.

Reason:

- resume is a first-class product behavior
- continuity has different retrieval priority
- continuity has stronger freshness pressure
