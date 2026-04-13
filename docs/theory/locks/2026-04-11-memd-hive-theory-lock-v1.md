# memd Hive Theory Lock v1

## Purpose

Define how `memd` should support many agents and harnesses acting like one second brain without collapsing into chaos, duplication, or latency waste.

This document also locks the role of **latency briefing**.

## Canonical Goal

Many harnesses should feel like one brain.

Switching between:

- Codex
- Claude Code
- OpenClaw
- Hermes
- future harnesses

should feel closer to changing terminals than losing context.

## Hive Principles

### 1. Shared Brain, Local Attention

The hive should share:

- canonical truth
- relevant semantic memory
- relevant procedural memory

But each agent should keep its own:

- working context
- immediate local reasoning state
- transient scratch state

### 2. Human-Owned Memory

The human owns the substrate.

Agents and harnesses route through it.

No one harness should become the true owner of the memory system.

### 3. Shared Truth Needs Authority

Hive memory must track:

- source
- freshness
- scope
- authority
- correction state

Without this, shared memory becomes contamination.

## Hive Memory Layers

### Local Working State

Per-agent only.

Contains:

- current reasoning packet
- local active hypotheses
- local next-step planning

### Shared Session Continuity

Only what should move across agents and harnesses.

Contains:

- what we are doing
- where we left off
- active blockers
- next action

### Shared Canonical Memory

The durable truth layer the hive can trust.

### Shared Procedural Memory

The reusable operating knowledge the hive can share.

## Handoff Model

Handoffs should not move whole transcripts.

They should move:

- compact task state
- canonical references
- procedural references
- provenance/trust state
- next action

This is the minimum useful cross-agent handoff.

## Latency Briefing

### Canonical Definition

Latency briefing is the fastest transferable warm-start package for a session, harness, or agent.

It exists to reduce warm-up cost before deeper retrieval.

It is not:

- canonical truth
- full wake packet replacement
- transcript summary sludge

It is:

- a low-latency startup bridge

### What Latency Briefing Should Carry

At minimum:

- current task
- where we left off
- next action
- active blockers
- critical canonical refs
- critical procedural refs
- freshness/confidence/authority hints

### Relationship to Wake Packet

- `latency briefing` is the fastest first hop
- `wake packet` is the richer action-ready packet

The flow should be:

1. latency briefing
2. wake packet
3. deeper typed retrieval if needed

### Future Path: Shared Prefix / KV Reuse

Latency briefing should eventually support more than text.

For compatible model families, it should evolve toward:

- shared prefix reuse
- shared KV cache reuse
- warm-start inference acceleration

This is motivated by cross-model KV reuse work like DroidSpeak, which shows that compatible model pairs can reuse large parts of prefix/KV state with major prefill and throughput gains while keeping quality nearly unchanged.

This means the long-term latency briefing stack may include:

- compact semantic brief
- structured memory refs
- prefix reuse hints
- KV/shared-prefix warm-start data for compatible models

### Important Constraint

Latency briefing is a speed layer.

It must never become:

- the truth layer
- the only memory layer
- an opaque performance hack with no provenance path

## Hive Conflict Rules

### Rule 1

Local working context must not overwrite shared canonical truth automatically.

### Rule 2

Corrections to shared truth must remain visible and attributable.

### Rule 3

Procedural memory should be promoted only from repeated or validated wins, not one-off agent behavior.

### Rule 4

Shared memory must respect scope:

- private
- workspace
- project
- shared hive

## What the Hive Must Enable

### 1. Cross-Harness Continuity

Start in one harness.
Continue in another.
Brain stays intact.

### 2. Cross-Agent Handoff

Delegate, return, and resume without reconstruction.

### 3. Shared Procedural Learning

One harness or agent can improve the procedure; others can reuse it.

### 4. Lower Warm-Up Cost

Repeated startup cost should fall because the system transfers briefings, not giant history blobs.

## 10-Star Hive Standard

The hive is good enough when:

- handoffs preserve intent, truth, and next action
- switching harnesses does not feel like starting over
- local reasoning state stays isolated
- shared truth stays trustworthy
- latency briefing materially reduces warm-up cost

## Locked Decisions

### D1. Authority Model

Conflicting candidate truth should resolve by authority stack:

1. explicit human correction
2. canonical truth with fresher verified evidence
3. higher-confidence promoted memory
4. candidate memory with stronger provenance
5. local harness guess

Harnesses do not get authority just for existing.
Authority comes from:

- human ownership
- provenance quality
- freshness
- correction state

### D2. Latency Briefing Boundary

`latency briefing` is:

- smallest fast handoff bridge
- routing and warm-start first hop

`wake packet` is:

- richer action-ready resume packet
- enough to start actual work

Practical split:

- latency briefing answers: where am I and what should I load?
- wake packet answers: what should I do next and why?

### D3. Control Plane Representation For KV/Prefix Reuse

Shared-prefix or shared-KV capability should be represented as a **transport capability**, not a memory kind.

Control plane should track:

- compatible model family
- compatible harness path
- briefing format support
- warm-start transport availability

This keeps KV reuse in the speed layer, not the truth layer.
