# memd Theory Lock v1

## What `memd` Is

`memd` is a multiharness second-brain memory substrate for the human.

It connects:

- models
- harnesses
- agents
- tools
- workflows

into one persistent memory system.

## What `memd` Is Not

It is not:

- a vector DB
- a notes app
- a transcript archive
- a repo reread workaround
- a single-agent memory toy

## Core Product Promise

Read once.
Remember once.
Reuse everywhere.

The system should let a fresh session answer:

- what are we doing
- where did we leave off
- what is true
- what should happen next
- why should we trust this

without repeated large rereads.

## Theory Lock: Native Memory Kinds

### 1. Working Context

What it is:

- the tiny active packet for current reasoning

Stores:

- current task
- active constraints
- immediate next move
- active hypotheses

### 2. Session Continuity

What it is:

- the resume layer

Stores:

- where we left off
- blockers
- open loops
- branch/workspace state
- next recommended action

### 3. Episodic Memory

What it is:

- memory of events and experiences

Stores:

- what happened
- when
- in what context
- with what result

### 4. Semantic Memory

What it is:

- stable project and world truths

Stores:

- decisions
- constraints
- facts
- architecture truths
- source-of-truth rules

### 5. Procedural Memory

What it is:

- memory of how to do things

Stores:

- workflows
- learned routines
- recovery patterns
- user and repo operating preferences

### 6. Candidate Memory

What it is:

- staging area before durable promotion

Stores:

- possible truths
- possible procedures
- repeated patterns

### 7. Canonical Memory

What it is:

- trusted durable memory layer

Contains promoted:

- semantic memory
- episodic memory worth keeping
- procedural memory

## Theory Lock: Native Control Functions

### 8. Correction + Provenance

Must do:

- show where memory came from
- show why it is trusted
- replace stale belief when corrected
- keep freshness and conflict visible

### 9. Wake Packet Compiler

Must do:

- compile tiny action-ready resume packets
- prefer memory over reread
- keep packets sharp, typed, and scoped

### 10. Hive Coordination

Must do:

- keep per-agent working state separate
- share canonical truth carefully
- share procedures carefully
- support handoff packets

## Theory Lock: Surfaces

These are surfaces, not the substrate:

- wake packet
- memory atlas
- canonical deep dive
- raw evidence
- Obsidian workspace

## Theory Lock: Memory Atlas

The memory atlas is the multidimensional navigation layer over canonical memory.

It is not the truth itself.

It supports:

- region navigation
- neighborhood traversal
- linked expansion
- zoom in and out
- multiple dimensions at once

Dimensions include:

- time
- salience
- trust
- provenance
- memory type
- scope
- project/domain
- harness/agent

## Theory Lock: Obsidian

Obsidian is:

- a first-class human workspace
- a source lane for notes and artifacts
- a readable rendered surface over memory

Obsidian is not:

- the control plane
- canonical truth by itself

## Theory Lock: Semantic Recall

Semantic recall is optional support, not core truth.

Its job:

- fuzzy related-context retrieval
- long-range association
- semantic expansion

It must not:

- outrank canonical memory
- replace provenance
- become the main truth layer

## Theory Lock: Live Loop

The live loop is:

1. capture event/artifact/correction
2. update working context
3. update session continuity
4. write episodic memory
5. repair semantic memory if truth changed
6. update procedural memory if pattern proved out
7. compile wake packet

## Theory Lock: Overnight Loop

The overnight loop is:

1. dream
2. autodream
3. autoresearch
4. autoevolve
5. promote accepted gains into semantic/procedural/canonical memory

This is Hermes-style always-on improvement, but inside one memory substrate.

## Theory Lock: Why This Can Beat Others

### Against raw-only systems

Keep raw truth, but also compile usable wake packets.

### Against rag-only systems

Keep truth typed and inspectable, not fuzzy by default.

### Against summary-only systems

Preserve evidence and correction paths.

### Against single-agent systems

Build shared human memory across harnesses.

## Theory Lock: Win Condition

`memd` wins when:

- new sessions resume flawlessly
- memory updates live while work happens
- corrections actually change future behavior
- humans and agents reread far less
- context gets smaller and sharper
- quality stays equal or improves
- many harnesses act like one second brain

## Theory Lock: Final Decisions

- canonical memory is a distinct durable surface
- candidate memory is one top-level staging layer with typed internal lanes
- atlas regions are hybrid: generated first, human-nameable, durable when useful
- no-RAG native retrieval stack includes wake, continuity, exact-span, entity, temporal, procedural, correction/provenance, canonical deep dive, and raw evidence fallback
- latency briefing is the first-hop warm-start bridge; wake packet is the richer action-ready resume packet; KV/prefix reuse belongs to transport capability, not memory kind
