# memd 10-Star Memory Model v2

## Core Thesis

`memd` should become the best **multiharness second-brain memory substrate** for humans and AGI-style systems.

Top-line product truth:

`memd` is a multiharness memory system that connects models, agents, tools, and workflows into one second brain for the human.

It should also function as token-efficiency infrastructure:

- read once
- remember once
- reuse everywhere
- cut repeated context cost substantially
- maintain or improve output quality through sharper memory packets

Not:

- a vector DB
- a summary engine
- a transcript compressor
- a repo reread workaround

But:

- a live memory operating system
- a shared second brain across harnesses
- a flawless session continuity system
- a truth-preserving memory control plane
- a hive-ready shared memory substrate
- a context compression system that reduces cost without reducing intelligence

## North Star

The north star is not "agent memory."

The north star is:

- one human-owned memory substrate
- many harnesses and models routing through it
- one persistent second brain underneath
- one read-once memory system that makes repeated context rebuilding unnecessary

This means `memd` must work across:

- Codex
- Claude Code
- OpenClaw
- OpenCode
- Hermes
- future harnesses and agents

Without turning memory into:

- transcript bloat
- per-tool silos
- repeated repo rereads
- fragmented agent-local state

## Compression Principle

`memd` should compress context the way a great memory system compresses experience:

- remove repetition
- preserve signal
- preserve provenance
- preserve actionability

The target is not cheaper but worse.

The target is:

- lower token cost
- lower latency
- cleaner context
- equal or better output quality

If done correctly, `memd` should deliver major repeated-context reduction on memory-heavy workflows while maintaining or improving result quality.

## The 10-Star Model

### 1. Raw Event Spine

What it stores:

- turns
- docs
- code artifacts
- screenshots
- logs
- hook events
- checkpoints
- user corrections

Job:

- preserve raw truth before loss
- keep provenance intact
- make later reconstruction possible

Rule:

- raw first
- never force lossy extraction on the hot path

### 2. Working Context

What it is:

- the tiny active packet an agent needs right now

Contains:

- current task
- active hypotheses
- immediate constraints
- next move

Job:

- minimize rereads
- keep the model focused

### 3. Session Continuity

What it stores:

- what we were working on
- where we left off
- current blockers
- open loops
- active branch or workspace state
- next recommended action

Job:

- let a fresh session resume cleanly in seconds

This is the answer to:

- what are we doing
- where did we stop
- what matters now

### 4. Episodic Memory

What it stores:

- what happened
- when it happened
- in what context
- with what outcome

Examples:

- a failed deploy
- a correction from the user
- a bug investigation
- a session handoff

Job:

- preserve timeline and situational memory

### 5. Semantic Memory

What it stores:

- stable project truths
- decisions
- constraints
- architecture facts
- source-of-truth rules

Job:

- answer what is true right now

Rule:

- corrections must update semantic truth fast

### 6. Procedural Memory

What it stores:

- how to do things
- repeated workflows
- operating patterns
- repo conventions
- user preferences that affect execution
- learned recovery patterns

Job:

- stop re-deriving known procedures every session

### 7. Candidate Memory

What it stores:

- repeated patterns not yet trusted enough
- possible truths
- possible procedures
- possible durable summaries

Job:

- hold memory before promotion
- avoid polluting canonical memory

Shape:

- one top-level layer
- typed internal lanes for semantic, procedural, and episodic candidates

### 8. Canonical Memory

What it is:

- the durable trusted memory surface

Contains:

- promoted semantic memory
- promoted episodic records worth keeping
- promoted procedures

Job:

- be the main durable source for recall
- outlive any single session or machine

### 9. Correction + Provenance Loop

What it does:

- tracks where memory came from
- shows why it is trusted
- replaces stale beliefs when corrected
- handles conflicts and freshness

Job:

- make memory inspectable
- make trust explicit

This is what stops:

- stale beliefs
- repeated mistakes
- fake confidence

### 10. Hive + Latency Layer

What it does:

- supports shared memory across agents
- supports shared memory across harnesses
- separates per-agent working state from shared truth
- ships compact briefing packets between sessions and agents
- prepares for prefix or KV reuse later

Authority rule:

- human correction outranks harness guess
- fresher verified truth outranks stale truth
- provenance outranks convenience

Boundary:

- latency briefing = fastest warm-start bridge
- wake packet = richer action-ready resume packet

Control-plane rule:

- KV/prefix reuse is a transport capability
- not a memory kind

## Atlas Decisions

Atlas node should be a promoted memory object:

- semantic object
- procedural object
- canonical episodic anchor
- continuity object when resumability matters

Regions should be hybrid:

- generated first
- human-nameable
- durable when useful

Atlas should store durable anchors and derive flexible neighborhoods on read.

## Promotion Decisions

Promotion should be type-specific.

Semantic promotion needs:

- verified source
- correction stability
- cross-session usefulness

Procedural promotion needs:

- repeated success
- reuse
- low contradiction

Canonical episodic promotion needs:

- anchor-event status
- future explanatory value
- repeated retrieval or linking value

Bias:

- slower promotion
- faster correction
- easy evidence drilldown

## Evaluation Rule

`memd` only counts as winning when quality is preserved.

Composite 10-star scorecard:

- `20%` session continuity
- `15%` correction retention
- `15%` procedural reuse
- `15%` cross-harness continuity
- `15%` raw retrieval strength
- `10%` token efficiency
- `10%` trust + provenance

Flagship moat benchmark:

- fresh session resume plus cross-harness continuation

Job:

- make hiveminds practical
- make cross-harness continuity practical
- reduce repeated warm-up cost
- keep shared memory from becoming chaos

## Memory Atlas

The memory atlas is the multidimensional navigation layer over canonical memory.

It is not the truth itself.

It exists so users and agents can:

- move from wake packet to linked context
- expand by neighborhood
- pivot by time, trust, scope, and memory type
- drill into raw evidence only when needed

The atlas should support:

- region navigation
- atlas paths through linked memory
- neighborhood expansion
- multidimensional pivots by time, trust, memory type, scope, and harness

Starter lanes:

- `Inspiration lane`
- `Design lane`
- `Architecture lane`
- `Research lane`
- `Workflow lane`
- `Preference lane`

Lane means domain grouping across memory kinds.
It is not a replacement for memory kinds.

## Obsidian

Obsidian is a first-class human workspace and readable rendered surface.

It is not the control plane and it is not canonical truth by itself.

## Retrieval Principle

Retrieval should be typed and staged.

Default order:

1. wake packet
2. session continuity
3. typed retrieval
4. atlas expansion
5. canonical deep dive
6. raw evidence

Native retrieval modes should include:

- resume retrieval
- episodic retrieval
- semantic retrieval
- procedural retrieval
- correction retrieval
- atlas retrieval
- evidence retrieval

Native retrieval primitives should include:

- exact-span retrieval
- entity retrieval
- temporal retrieval
- procedural retrieval
- preference/policy retrieval

## Operational Flow

### Live Loop

1. capture raw event
2. update working context
3. update session continuity
4. write episodic record
5. repair semantic truth if correction or durable fact changed
6. update procedural memory if a reusable pattern proved out
7. compile tiny wake packet

### Consolidation Loop

1. inspect recent episodic records
2. extract candidate truths and candidate procedures
3. merge duplicates
4. expire weak signal
5. promote strong signal into canonical memory

### Resume Loop

1. load session continuity
2. merge with semantic memory
3. pull relevant procedures
4. compile working context
5. continue without big reread

### Hive Loop

1. keep per-agent working context local
2. sync shared semantic and procedural truth carefully
3. exchange compact handoff packets
4. preserve authority, freshness, and provenance

## What Makes This Better Than Current Systems

### Better than raw-only systems

- keeps raw truth
- but also compiles usable wake packets
- and learns procedures

### Better than summary-only systems

- does not throw away evidence
- keeps correction path explicit
- does not compress by becoming dumb

### Better than vector-only systems

- memory is typed
- not all retrieval becomes similarity search

### Better than transcript-shaped memory

- compiles memory into smaller, sharper packets
- reduces token waste
- can improve reasoning quality by removing irrelevant context

### Better than retrieval-only systems

- keeps raw retrieval strength in scope
- but also solves resume, correction, procedural learning, and hive continuity

## What We Keep From mempalace

- raw-first storage discipline
- small wake-up context
- retrieval-stage optimization matters more than clever extraction
- navigable memory topology is useful

## What We Reject From mempalace

- metaphor as full architecture
- lossy compression as default memory path
- semantic search as the whole memory system
- treating navigation structure as canonical truth

## What We Keep From Hermes

- procedural memory matters
- always-on loops matter
- lifecycle hooks matter
- scheduled automation matters
- checkpoint and rollback thinking matters

## What We Reject From Hermes

- agent-first ownership model
- provider abstraction as the memory theory
- profile-local memory as the main architecture
- treating skills alone as the full procedural memory model

## Canonical Memory Rule

Canonical memory is the durable trusted memory surface.

It may contain promoted:

- semantic memory
- episodic memory worth keeping
- procedural memory

It should never become:

- a transcript dump
- an uncorrected summary layer
- a fuzzy semantic backend

Candidate memory exists to prevent premature canonization.

Promotion should happen when signal is:

- repeated
- validated
- stable under correction
- useful across sessions or harnesses

## Hive Rule

The hive should share:

- canonical truth
- relevant semantic memory
- relevant procedural memory

The hive should not automatically share:

- local working context
- transient scratch state

Latency briefing is the fastest transferable warm-start package.

It should eventually support:

- compact semantic briefing
- structured memory refs
- prefix reuse hints
- shared-prefix / KV warm-start for compatible model families

It is a speed layer, not the truth layer.

## Evaluation Rule

`memd` should not be judged by one benchmark alone.

It must prove strength across:

- raw retrieval
- session continuity
- correction retention
- procedural reuse
- cross-harness continuity
- repeated-context reduction
- quality preservation or gain
- trust and provenance

### Better than single-agent systems

- built for shared truth and handoff

## Canonical Product Promise

On a fresh session, `memd` should answer:

- what are we working on
- where did we leave off
- what changed
- what is true
- how should we proceed
- why should we trust this

Without:

- transcript bloat
- giant repo rereads
- manual reconstruction

Across:

- different models
- different harnesses
- different sessions
- different machines

## Short Names For Graphs

- Raw event spine
- Working context
- Session continuity
- Episodic memory
- Semantic memory
- Procedural memory
- Candidate memory
- Canonical memory
- Correction and provenance
- Hive and latency
