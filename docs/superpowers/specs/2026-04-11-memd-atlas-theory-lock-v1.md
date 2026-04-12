# memd Atlas Theory Lock v1

## Purpose

Define what the `memory atlas` actually is.

Without this, "atlas" risks becoming:

- vague metaphor
- visual garnish
- graph buzzword

## Canonical Definition

The memory atlas is the multidimensional navigation layer over canonical memory.

It is:

- a navigation surface
- a linking system
- a zoom system
- a region model

It is not:

- canonical truth itself
- the raw event spine
- just a vector index
- just Obsidian

## What the Atlas Must Enable

### 1. Region Navigation

Users and agents should be able to start from a meaningful region, such as:

- current task
- project area
- person
- subsystem
- theme
- active problem

### 2. Progressive Zoom

Users and agents should be able to move:

- from wake packet
- to region
- to linked nodes
- to canonical deep dive
- to raw evidence

### 3. Neighborhood Expansion

If one memory is relevant, the atlas should help answer:

- what nearby memories matter too?
- what connected procedure exists?
- what related correction exists?
- what older event explains this?

### 4. Cross-Dimensional Pivoting

The atlas should let users and agents pivot by:

- time
- trust
- provenance
- memory type
- scope
- harness
- salience

## Atlas Objects

### Region

Definition:

- a meaningful memory neighborhood

Examples:

- auth migration
- benchmark work
- user deployment preferences

### Node

Definition:

- a navigable unit inside a region

A node may represent:

- a canonical memory item
- a procedure
- an event cluster
- a durable fact

### Link

Definition:

- an explicit or inferred relationship between nodes

Link types may include:

- temporal
- causal
- procedural
- semantic
- corrective
- ownership/scope

### Trail

Definition:

- a path through linked memory chosen for a current question or task

This replaces the earlier weaker phrase "progressive wikilink trail."

Preferred term:

- `atlas path`

## Atlas Dimensions

The atlas is multidimensional, not just graph-shaped.

### Time

- when it happened
- sequence
- recency

### Memory Type

- episodic
- semantic
- procedural
- continuity

### Trust

- confidence
- freshness
- contested vs verified

### Scope

- human
- project
- workspace
- harness
- shared hive

### Salience

- what matters now
- what matters often
- what matters deeply

### Lane

- what domain the memory belongs to
- how related memory stays grouped across kinds

Starter lanes:

- inspiration
- design
- architecture
- research
- workflow
- preference

## Atlas Generation Principles

### Principle 1

Atlas structure should emerge from memory truth, not replace it.

### Principle 2

Atlas paths should improve navigation, not force extra reading.

### Principle 3

Atlas should support both:

- human exploration
- agent retrieval

### Principle 4

Atlas must stay source-linked through canonical memory and raw evidence.

### Principle 5

Lanes should cut across memory kinds.

Example:

- `design lane` may contain episodic sessions, semantic rules, procedural workflows, candidate ideas, and canonical truths

## Atlas vs Obsidian

Obsidian may be one rendered surface of atlas structure.

But atlas is broader than Obsidian:

- atlas is the navigation model
- Obsidian is one workspace expression of it

## Atlas Lanes

Lanes are the domain-grouping layer inside the atlas.

They answer:

- what world does this memory belong to?
- where should humans and agents start looking?

Best current starter set:

- `Inspiration lane`
- `Design lane`
- `Architecture lane`
- `Research lane`
- `Workflow lane`
- `Preference lane`

Lanes are:

- stable enough to navigate
- broad enough to hold many memory kinds
- human-readable
- compatible with generated expansion later

## Atlas vs Semantic Backend

Semantic backends may help discover related nodes.

But:

- semantic backend is helper logic
- atlas is the navigable memory layer

## 10-Star Atlas Standard

The atlas is good enough when:

- a user can start from the current task and move outward naturally
- an agent can pull nearby context without giant rereads
- moving deeper feels like zooming, not searching from scratch
- truth and source linkage remain intact through every depth

## Locked Decisions

### D1. Canonical Node Unit

The canonical atlas node should be a **promoted memory object**.

That means a node is one of:

- semantic object
- procedural object
- canonical episodic anchor
- continuity object when resumability matters

Raw events are not atlas nodes by default.
They stay beneath the atlas as evidence.

Clusters may exist, but they should behave as:

- region summaries
- derived views

not as the canonical node primitive.

### D2. Regions

Regions should be **hybrid**:

- generated first
- user-nameable
- durable when useful

This keeps atlas navigation natural while still allowing human curation.

### D3. Stored vs Derived Atlas Structure

Atlas structure should be **partially explicit and partially derived**.

Persist explicitly:

- important regions
- durable node ids
- durable high-value links
- user-named paths

Derive on read:

- low-value neighborhood expansion
- semantic adjacency
- temporary exploration paths

## 10-Star Atlas Implementation Bias

Default bias:

- store durable anchors
- derive flexible neighborhoods

This avoids over-modeling while keeping navigation stable.
