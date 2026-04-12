# memd Ralph Execution Artifact

This file is a historical execution artifact and reference mirror.

Execution truth now lives in `ROADMAP.md`.

## Purpose

This is the older Ralph-loop execution artifact for the 10-star memory model.

It remains useful as historical planning context, but it is not the repo
roadmap source of truth and should not carry independent phase status.

Each phase must:

- define one bounded capability jump
- declare pass/fail criteria
- require evidence
- declare rollback conditions
- loop until it actually passes

No phase is "done" because code landed.
It is done when the behavior bar is met.

## Ralph Rules

Every loop must:

1. start from the current theory docs
2. target one bounded capability
3. write artifacts, not just code
4. run verification
5. report what passed and what failed
6. stop only on pass criteria
7. roll back or narrow scope if guardrails fail

## Global Evidence Requirements

Every phase should leave:

- spec or design delta
- implementation delta
- verification evidence
- token-cost evidence where relevant
- failure notes if something regressed

## Global Guardrails

No phase may:

- weaken raw truth retention
- weaken source provenance
- increase transcript dependence
- hide corrections
- make canonical truth fuzzier

## Phase 1: Raw Truth Spine

### Goal

Make raw event and artifact capture the unquestioned foundation.

### Inputs

- harness events
- hooks
- checkpoints
- files
- screenshots
- logs
- user corrections

### Deliver

- one raw event spine model
- source-linked ingest path
- read-once source registry

### Pass Gate

- raw inputs from major harnesses land in one source-linked spine
- no lossy hot-path extraction required
- unchanged sources do not get reread

### Evidence

- ingest tests
- source registry behavior
- provenance drilldown

### Fail Conditions

- raw artifacts lose source linkage
- reread behavior still duplicates unchanged sources

### Rollback

- revert new ingest transforms that degrade source fidelity

## Phase 2: Session Continuity

### Goal

A fresh session resumes cleanly in seconds.

### Deliver

- current-task state
- open loops
- blockers
- next action
- branch/workspace/session context

### Pass Gate

- a fresh session can answer:
  - what are we doing
  - where did we leave off
  - what changed
  - what next
- without transcript rebuild

### Evidence

- resume tests
- handoff tests
- attach/resume journey evidence

### Fail Conditions

- fresh sessions still require manual reconstruction

### Rollback

- revert resume compaction changes that drop critical continuity

## Phase 3: Typed Memory

### Goal

Separate memory by kind, not one flat store.

### Required Kinds

- working context
- session continuity
- episodic memory
- semantic memory
- procedural memory
- candidate memory
- canonical memory

### Pass Gate

- types are explicit in code, docs, and retrieval behavior
- no key feature depends on one undifferentiated memory bucket

### Evidence

- schema/docs alignment
- retrieval traces showing type-aware behavior

### Fail Conditions

- flat memory bucket still dominates retrieval and writeback

### Rollback

- revert fake type labels with no behavioral effect

## Phase 4: Canonical Truth + Provenance

### Goal

Durable trusted memory with visible source and correction behavior.

### Deliver

- correction overwrite path
- trust state
- freshness
- conflict handling
- promotion rules

### Pass Gate

- stale beliefs can be replaced
- source and confidence remain inspectable
- canonical truth is not silently overwritten by fuzzy recall

### Evidence

- correction tests
- provenance drilldown tests
- belief conflict tests

### Fail Conditions

- memory drift
- hidden corrections
- trust ambiguity

### Rollback

- revert promotion or merge logic that hides contradictions

## Phase 5: Wake Packet Compiler

### Goal

Compile tiny action-ready packets instead of forcing rereads.

### Deliver

- wake packet schema
- packet compiler
- packet evaluation

### Pass Gate

- packets are smaller than transcript/context rebuild baseline
- resume quality stays equal or improves
- repeated-context cost drops materially

### Evidence

- prompt size comparisons
- task success comparisons
- resume quality evals

### Fail Conditions

- packets get smaller but quality drops
- packets still require reread fallback too often

### Rollback

- revert over-compression paths

## Phase 6: Memory Atlas

### Goal

Create multidimensional navigation over memory.

### Deliver

- atlas model
- region/neighbor/trail navigation
- progressive zoom from packet to evidence

### Pass Gate

- users and agents can move from wake packet to linked canonical context to evidence
- navigation is useful without becoming the truth layer

### Evidence

- atlas navigation demos
- trail-to-evidence verification
- user-path examples

### Fail Conditions

- atlas duplicates truth instead of navigating it
- atlas becomes visual fluff with no retrieval advantage

### Rollback

- revert atlas features that do not improve navigation behavior

## Phase 7: Procedural Memory

### Goal

Memory learns how to operate, not just what is true.

### Deliver

- learned procedures
- operating policies
- reusable workflows
- recovery patterns

### Pass Gate

- repeated successful workflows can be promoted and reused
- future sessions stop re-deriving the same procedure

### Evidence

- repeated task traces
- procedural reuse tests
- policy reuse examples

### Fail Conditions

- procedures exist only as docs with no runtime effect

### Rollback

- revert procedural promotion that causes bad automation or brittle habits

## Phase 8: Hive Coordination

### Goal

Many harnesses and agents act like one second brain.

### Deliver

- shared truth
- shared procedures
- local working-state isolation
- handoff packets
- ownership and freshness rules

### Pass Gate

- switching harnesses feels like changing terminals, not losing the brain
- handoffs preserve truth and next action

### Evidence

- cross-harness continuity tests
- hive handoff journey tests
- ownership/conflict tests

### Fail Conditions

- shared memory contamination
- handoffs lose intent or context

### Rollback

- revert unsafe sharing paths and narrow scope boundaries

## Phase 9: Overnight Evolution

### Goal

Real always-on improvement loops strengthen memory over time.

### Deliver

- dream
- autodream
- autoresearch
- autoevolve
- accepted-learning promotion

### Pass Gate

- accepted improvements measurably improve memory behavior
- overnight work does not lower trust or raise drift

### Evidence

- loop telemetry
- before/after evaluation deltas
- rollback records

### Fail Conditions

- self-improvement lowers trust
- speculative changes get promoted without proof

### Rollback

- revert accepted-learning promotion and freeze the loop

## Phase 10: Multiharness Second Brain Product Bar

### Goal

Prove the system actually behaves like one second brain for the human.

### Pass Gate

- read once, reuse everywhere is true in real workflows
- fresh sessions resume cleanly
- memory updates live
- corrections change future behavior
- context stays smaller and sharper
- quality stays equal or improves

### Evidence

- end-to-end user journeys
- token-cost reduction evidence
- quality preservation evidence
- cross-harness continuity evidence

### Fail Conditions

- product still feels like memory tooling instead of real memory

### Rollback

- narrow the claim and reopen failed phases instead of shipping false confidence

## Loop Mapping

The existing research loops map into this roadmap:

- Prompt Surface Compression -> Phase 5
- Live Truth Freshness -> Phases 1 and 4
- Capability Contract Detection -> Phase 8
- Event Spine Compaction -> Phase 1
- Correction Learning -> Phase 4
- Long-Context Avoidance -> Phases 2 and 5
- Cross-Harness Portability -> Phase 8
- Controlled Self-Evolution -> Phase 9
- Branch Review Quality -> all phases
- Docs Spec Drift -> all phases

## Definition of Knowing

We know the roadmap is working when each phase can answer:

- what changed
- what evidence proves it
- what still fails
- what would force rollback

If we cannot answer those, we do not know yet.
