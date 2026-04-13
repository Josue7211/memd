# DESIGN.md

> Secondary design guidance. For current product truth use [[ROADMAP]] and
> [[docs/WHERE-AM-I.md|WHERE-AM-I]] first.

## Product Design Source of Truth

`memd` is a multiharness second brain for humans and agents.

Design must make that feel true.

Not:

- a note app
- a RAG dashboard
- a transcript graveyard
- a devtool full of memory jargon

But:

- one brain
- many surfaces
- progressive depth
- low-friction trust

## Core User Experience

The user should feel:

- I already read this once
- the system remembers it
- I can resume instantly
- I can go deeper only if I want
- I can trust why this is here
- every harness is using the same brain

## Primary Product Surfaces

Visible default product shape:

- `wake.md`
- `mem.md`
- `events.md`

Design rules:

- these are shared across harnesses
- harness-specific adapters should stay thin
- no duplicated per-harness wake, memory, or events payloads
- Claude Code should boot from `wake.md` only
- `mem.md` and `events.md` should feel available, not preloaded

### 1. Wake Packet

Smallest useful context.

Must answer:

- what are we doing
- where did we stop
- what changed
- what next

For Claude Code, this surface must stay brutally small.
It is the hot path.

### 2. Memory Atlas

Multidimensional navigation layer over memory.

Must support:

- zoom out to region
- move by linked neighborhood
- pivot by project, time, trust, type, or lane
- zoom into canonical context
- drill into raw evidence

Starter lanes:

- inspiration
- design
- architecture
- research
- workflow
- preference

### 3. Canonical Deep Dive

Trusted durable context when the task really needs more.

### 4. Raw Evidence

The trust anchor.

Never far away.

### 5. Obsidian Workspace

First-class human-readable surface.
Not the control plane.

## Information Design Principles

### Progressive Depth

Default shallow.
Deeper only on demand.

Practical loading model:

- `wake.md` is hot
- `mem.md` is explicit project/topic recall
- `events.md` is audit and recovery
- deeper trust and evidence should be easy to reach, but almost never forced into startup

### Typed Memory

The system should visibly distinguish:

- working context
- session continuity
- episodic memory
- semantic memory
- procedural memory
- candidate memory
- canonical memory

### Trust First

Every important memory should show:

- source
- freshness
- confidence
- correction state

### Read Once, Reuse Everywhere

The interface should reward reuse, not reread.

### Sharper Context, Better Output

Compression is only good if quality stays equal or improves.

### Wake Smarter

The product should improve future wake packets, not just store more text.

That means:

- detect gaps from recent work
- run dream/nightly or bounded maintenance passes
- accept only validated improvements
- refresh canonical memory and the next wake packet

## Visual Direction

The visual language should feel:

- precise
- infrastructural
- calm
- high-signal
- slightly scientific

Avoid:

- cute metaphors dominating the UI
- fake 3D brain slop
- overexplaining basic memory words
- dashboards that look like observability products with memory labels pasted on

Use:

- strong hierarchy
- layered panels
- graph/topology views only when they help navigation
- evidence-first drilldown
- compact packets over giant walls of text

## Canonical Language

Preferred product terms:

- multiharness second brain
- working context
- session continuity
- episodic memory
- semantic memory
- procedural memory
- canonical memory
- memory atlas
- lane
- wake packet
- raw evidence

Avoid as primary language:

- drawer
- room
- palace
- RAG as product identity

## Success Standard

The product is designed correctly when:

- a new session resumes without manual reconstruction
- the user trusts memory more than transcript scrollback
- moving between harnesses feels like changing terminals, not losing the brain
- deeper context is always available, but rarely required
