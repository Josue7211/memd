# memd Canonical + Promotion Theory Lock v1

## Purpose

Define:

- what canonical memory really is
- how memory gets promoted
- how corrections change durable truth

Without this, the whole model stays fuzzy.

## Canonical Memory

### Canonical Definition

Canonical memory is the durable trusted memory surface that the system should prefer for recall before falling back to deeper raw evidence.

It is not:

- the raw event spine
- a semantic backend
- a flat dump of everything

It is:

- the promoted durable layer
- trust-aware
- source-linked
- correction-aware

## What Can Become Canonical

### Canonical Semantic Memory

Examples:

- architecture decisions
- repo constraints
- deployment rules
- source-of-truth facts

### Canonical Episodic Memory

Examples:

- important incidents
- major handoffs
- key investigations
- events worth long-term retention

Not every episode should become canonical.

### Canonical Procedural Memory

Examples:

- validated workflows
- recovery procedures
- durable operating policies

## What Should Not Become Canonical Easily

- transient local reasoning
- weak one-off guesses
- unverified speculative summaries
- stale beliefs

## Candidate Memory

Candidate memory exists to prevent premature canonization.

It holds:

- repeated signal
- possible truths
- possible procedures
- compressed candidates

Candidate memory is:

- non-canonical
- reviewable
- promotion-eligible
- discardable

## Promotion Rules

### Promotion should happen when signal is:

- repeated
- validated
- corrected into stability
- useful across sessions
- useful across harnesses

### Promotion should not happen just because signal is:

- recent
- verbose
- emotionally intense
- convenient to summarize

## Promotion Paths

### Episodic -> Candidate

When:

- event repeats
- event proves important
- event affects future work

### Candidate -> Canonical Semantic

When:

- pattern resolves into durable truth
- decision is stable
- correction confirms final state

### Candidate -> Canonical Procedural

When:

- workflow succeeds repeatedly
- recovery pattern proves durable
- operating policy survives re-use

### Episodic -> Canonical Episodic

When:

- event remains historically important
- event explains future work
- event becomes anchor for many later links

## Correction Rules

### Rule 1

Corrections must update semantic truth fast.

### Rule 2

Old truth should not disappear without trace.

It should become:

- superseded
- stale
- contested

### Rule 3

Procedural memory must also change when a correction affects how work should be done.

### Rule 4

Wake packet should update after meaningful correction.

## Provenance Requirements

Every canonical memory should preserve:

- source path
- source type
- freshness
- confidence
- promotion reason
- correction history

## Canonical Retrieval Rule

Recall should prefer:

1. wake packet
2. canonical memory
3. atlas expansion
4. raw evidence

This means:

- raw evidence remains available
- canonical memory remains the trusted default

## Promotion Failures To Avoid

### Failure 1: Transcript Canonization

Large text blocks get promoted just because they exist.

### Failure 2: Summary Slop

Compressed text gets promoted without preserving source truth.

### Failure 3: One-Off Procedure Drift

One successful action becomes policy too early.

### Failure 4: Uncorrected Durable Error

Wrong belief stays canonical after correction arrives.

## 10-Star Standard

Canonical + promotion system is good enough when:

- durable truth is reusable across sessions
- stale beliefs are replaced quickly
- procedures improve over time without becoming brittle
- raw evidence remains reachable
- promotion increases quality instead of just storage volume

## Locked Decisions

### D1. Candidate Memory Shape

Candidate memory should be one top-level layer with **typed internal lanes**:

- candidate-semantic
- candidate-procedural
- candidate-episodic

This keeps the model simple at the top and precise underneath.

### D2. Promotion Thresholds

Promotion thresholds should be type-specific.

Canonical semantic promotion requires:

- verified source
- stability after correction
- usefulness beyond one session

Canonical procedural promotion requires:

- repeated success
- reuse across sessions or harnesses
- low contradiction rate

Canonical episodic promotion requires:

- historical significance
- future explanatory value
- repeated retrieval pressure or linking value

### D3. Canonical Episodic Importance Rule

An episode becomes canonical when it is an **anchor event**.

Anchor events are events that:

- explain later work
- changed policy or truth
- define a major handoff
- represent a major incident
- are repeatedly revisited

Normal episodes stay episodic only.

## 10-Star Promotion Bias

Bias toward:

- slower promotion
- faster correction
- easier evidence drilldown

Better to under-promote than poison canonical memory.
