# memd Retrieval Theory Lock v1

## Purpose

Define how `memd` should retrieve memory without collapsing into:

- flat semantic search
- giant rereads
- one-size-fits-all retrieval

## Core Rule

Retrieval should be **typed and staged**.

The system should ask:

- what kind of memory is needed?
- what depth is needed?
- what evidence level is needed?

before deciding how to retrieve.

## Retrieval Order

Default order:

1. wake packet
2. session continuity
3. typed targeted retrieval
4. atlas expansion
5. canonical deep dive
6. raw evidence

This means:

- never start with giant search if a wake packet is enough
- never start with raw evidence if canonical truth is enough

## Native Retrieval Modes

### 1. Resume Retrieval

Use when:

- starting a new session
- resuming after interruption
- switching harnesses

Primary targets:

- session continuity
- working context
- next action

Output:

- wake packet

### 2. Episodic Retrieval

Use when asking:

- what happened
- when did that occur
- what was the sequence

Primary targets:

- episodic memory
- time-linked events

### 3. Semantic Retrieval

Use when asking:

- what is true
- what decision did we make
- what constraint exists

Primary targets:

- semantic memory
- canonical memory

### 4. Procedural Retrieval

Use when asking:

- how do we do this
- what workflow works here
- what pattern should we follow

Primary targets:

- procedural memory
- canonical procedural promotions

### 5. Correction Retrieval

Use when asking:

- was this corrected before
- is this belief contested
- why should we trust this

Primary targets:

- correction/provenance state
- semantic memory
- raw evidence if needed

### 6. Atlas Retrieval

Use when:

- user or agent needs to explore neighboring context
- direct retrieval found the core but more context may help

Primary targets:

- linked canonical memory
- neighboring regions
- connected memory types

### 7. Evidence Retrieval

Use when:

- high trust needed
- ambiguity remains
- correction or contradiction exists

Primary targets:

- raw event spine
- source artifacts

## Native Retrieval Primitives

These should become first-class primitives, learned from `mempalace` and theory work:

### Exact-Span Retrieval

For:

- quoted phrases
- exact strings
- exact config or code references

### Entity Retrieval

For:

- people
- projects
- services
- named concepts

### Temporal Retrieval

For:

- dates
- recency
- sequences
- "last time" and "a week ago" style queries

### Procedural Retrieval

For:

- operation patterns
- workflows
- recovery sequences

### Preference / Policy Retrieval

For:

- user corrections
- operating preferences
- repo rules

### Semantic Expansion

For:

- fuzzy related-context search
- broader neighborhood expansion

This is optional helper behavior, not the truth layer.

## Retrieval Depth Model

### Depth 0: Wake Packet

- smallest useful action-ready packet

### Depth 1: Typed Recall

- targeted episodic/semantic/procedural retrieval

### Depth 2: Atlas Expansion

- linked navigation around the retrieved memory

### Depth 3: Canonical Deep Dive

- larger trusted context block

### Depth 4: Raw Evidence

- source truth

## Retrieval Safety Rules

### Rule 1

Do not let semantic expansion outrank canonical truth.

### Rule 2

Do not let a deeper retrieval replace a smaller sufficient answer.

### Rule 3

Do not drop provenance when compressing retrieval output.

### Rule 4

Do not let retrieval become benchmark patch soup.

Generalize into primitives instead:

- temporal
- exact-span
- entity
- procedural
- preference/policy

## What This Implies For memd

`memd` should not be built around one single "search" command.

It should be built around:

- retrieval mode selection
- typed retrieval
- progressive depth
- trust-aware evidence fallback

## Locked Decisions

### D1. Mode Selection

Retrieval mode selection should be:

- automatic by default
- inspectable when needed
- overridable by user or harness

The system chooses first.
The human can still force depth or mode.

### D2. Atlas Expansion Trigger

Atlas expansion should be conditional, not always-on.

Trigger atlas expansion when:

- confidence is incomplete
- user asks for broader context
- neighboring procedure/correction likely matters
- continuity packet references linked regions

Do not expand automatically when typed retrieval is already sufficient.

### D3. Minimal Native Retrieval Stack

Before any optional semantic backend, native `memd` retrieval should include:

- wake packet retrieval
- session continuity retrieval
- exact-span retrieval
- entity retrieval
- temporal retrieval
- procedural retrieval
- correction/provenance retrieval
- canonical deep dive
- raw evidence fallback

This is the minimum no-RAG serious stack.
