# memd Evaluation Theory Lock v1

## Purpose

Define how we will know the 10-star memory model is actually better.

Without this, the project can always claim progress without proving it.

## Core Rule

`memd` should not be judged by one benchmark alone.

Why:

- retrieval-only benchmarks are too narrow
- memory systems can overfit to one evaluation set
- second-brain behavior is broader than retrieval

## Evaluation Axes

The system must be evaluated across all of these:

### 1. Raw Retrieval Strength

Questions:

- can the system find the right source memory?
- can it compete with retrieval-first systems like `mempalace`?

### 2. Session Continuity

Questions:

- can a fresh session resume without manual reconstruction?
- can it answer what we are doing, where we left off, and what next?

### 3. Correction Retention

Questions:

- does user correction actually change future behavior?
- do stale beliefs get replaced quickly?

### 4. Procedural Reuse

Questions:

- does the system stop re-deriving known workflows?
- do promoted procedures improve later sessions?

### 5. Cross-Harness Continuity

Questions:

- does moving between harnesses preserve the same brain?
- does handoff maintain truth, scope, and next action?

### 6. Token Efficiency

Questions:

- does repeated-context cost drop?
- does the system avoid transcript rebuild and giant rereads?

### 7. Quality Preservation Or Gain

Questions:

- do smaller packets maintain quality?
- does sharper context improve output quality?

### 8. Trust + Provenance

Questions:

- can the user inspect why something is remembered?
- can the agent fall back to evidence when needed?

## Benchmark Classes

### Class A: Public Retrieval Benchmarks

Use:

- LongMemEval
- LoCoMo
- ConvoMem
- other strong public long-memory benchmarks

Purpose:

- prove baseline retrieval strength

### Class B: memd Native Product Benchmarks

Need custom benchmark suites for:

- fresh-session resume
- cross-agent handoff
- cross-harness continuity
- correction retention
- procedural reuse
- repeated-context reduction

Purpose:

- measure the real product moat

### Class C: Loop Telemetry

Track:

- prompt size
- reread count
- stale belief count
- correction recurrence
- accepted procedural promotion
- handoff quality

Purpose:

- make progress visible in real work, not just benchmark runs

## Required memd Native Journeys

### Journey 1: Fresh Session Resume

Input:

- prior work exists

Pass:

- new session answers current task, last stop point, blockers, and next action

### Journey 2: Correction Overwrite

Input:

- system makes a wrong assumption
- user corrects it

Pass:

- future sessions stop repeating the wrong assumption

### Journey 3: Cross-Harness Handoff

Input:

- work starts in one harness
- continues in another

Pass:

- context and next action survive transfer

### Journey 4: Procedural Reuse

Input:

- same workflow appears repeatedly

Pass:

- later sessions reuse procedure instead of rediscovering it

### Journey 5: Atlas Navigation

Input:

- wake packet

Pass:

- user or agent can move from packet to canonical context to raw evidence without starting search from scratch

## Anti-Cheating Rules

### Rule 1

Do not count quality loss as success just because token cost fell.

### Rule 2

Do not count benchmark gains that come only from benchmark-specific hacks as general architectural progress.

### Rule 3

Do not count summary compression as memory success if source truth or correction paths degrade.

### Rule 4

Do not count multiharness support as real unless continuity survives actual harness switching.

## 10-Star Standard

`memd` is truly 10-star only if it can:

- match or exceed strong retrieval baselines
- resume real work better than retrieval-only systems
- retain corrections
- reuse procedures
- reduce repeated context cost
- keep trust and provenance intact
- behave like one second brain across harnesses

## Locked Decisions

### D1. Composite 10-Star Scorecard

Use weighted multi-axis scoring:

- `20%` session continuity
- `15%` correction retention
- `15%` procedural reuse
- `15%` cross-harness continuity
- `15%` raw retrieval strength
- `10%` token efficiency
- `10%` trust + provenance

Quality preservation is a gating condition, not a bonus category.

If quality drops below threshold, total score fails regardless of token gains.

### D2. Token + Quality Rule

Token efficiency only counts when:

- answer quality is maintained
or
- answer quality improves

So the metric should be:

- first pass quality gate
- then measure repeated-context reduction

Never reward overcompression that harms:

- correctness
- continuity
- trust

### D3. Definitive memd Moat Benchmark

The definitive native moat benchmark should be:

- `Fresh Session Resume + Cross-Harness Continuation`

Why:

- it tests the real second-brain claim
- it exposes whether memory actually replaces reread
- it is harder to fake with retrieval hacks alone

Retrieval benchmarks still matter.
But this should be the flagship product benchmark.
