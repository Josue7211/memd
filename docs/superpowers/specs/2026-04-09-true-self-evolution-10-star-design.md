# True Self-Evolution 10-Star Design

## Goal

Turn `memd` self-evolution from a passive score into an active control plane that can:

- detect improvement pressure
- generate bounded proposals
- evaluate changes against baseline
- promote safe wins automatically
- keep risky wins isolated until they prove durable

The long-term target is true self-evolution, not just loop telemetry.

## Problem

Current `self-evolution` is a sentinel loop.

It checks whether the latest experiment report is:

- accepted
- fresh
- not restored
- backed by a non-zero composite score

That is useful, but small. It does not yet:

- open isolated evolution branches
- classify what kind of change is being proposed
- distinguish `accepted`, `merged`, and `durable`
- expand or contract autonomy based on evaluator precision

## 10-Star Shape

### Layer 0: Proposal Generation

Every weak loop, repeated operator action, or recurring gap can emit an evolution candidate.

Each candidate must declare:

- origin loop or gap
- target layer
- expected win
- risk class
- evaluation plan
- rollback plan

### Layer 1: Self-Tuning Runtime

The system may evolve cheap, reversible behavior directly:

- loop floors
- scoring weights
- prompt shaping
- memory compaction policy
- recovery heuristics
- workflow recipes

These changes should be able to auto-promote once accepted.

### Layer 2: Branch-Native Code Evolution

The system may open isolated evolution branches for allowed code classes.

Branch family:

- `auto/evolution/<layer>/<topic>-<timestamp>`

Examples:

- `auto/evolution/policy/prompt-efficiency-2026-04-09-1530`
- `auto/evolution/code/self-evolution-gates-2026-04-09-1605`

Nothing in this lane writes directly onto the active working branch.

### Layer 3: Durable Truth

A change is not durable because it passed once.

A promoted change becomes durable only after:

- later verification still shows a win
- surrounding context changes do not break it
- downstream protected signals do not regress

### Layer 4: Authority Expansion

Autonomy is earned, not assumed.

If a change class keeps passing with high precision, authority expands there.
If a change class regresses, authority contracts automatically.

This creates a live trust boundary instead of a one-time trust decision.

## Rollout

### Phase 1: Ship `2`

Two lanes:

- lane A: runtime evolution
- lane B: code evolution on isolated branches

Runtime policy may auto-promote.
Code proposals stay on evolution branches until accepted and reviewed.

### Phase 2: Upgrade Toward `3`

Unlock narrow auto-merge for low-risk code classes once durability metrics are strong enough.

The correct path is `2 -> 3`, not a reckless direct jump.

## States

Every evolution candidate has one of these states:

- `rejected`
- `accepted_proposal`
- `merged`
- `durable_truth`
- `demoted`

Important:

- `accepted_proposal` means it beat baseline in isolation
- `merged` means it landed in the allowed surface
- `durable_truth` means it still wins later

These are different states and must stay different in code and telemetry.

## Acceptance Gates

Every evolution branch or policy proposal must pass five gates.

### 1. Baseline Gate

The proposal must beat or preserve the originating signal relative to the pre-change baseline.

No plausibility-only acceptance.

### 2. Regression Gate

The proposal must not degrade protected signals:

- tests
- loop health
- review readiness
- portability
- repair behavior

### 3. Scope Gate

The proposal must declare the maximum allowed write surface:

- runtime policy only
- docs/spec only
- low-risk evaluation code
- broader implementation code

Wider scope means stricter acceptance.

### 4. Portability Gate

If a win only works in one harness, one branch state, or one local condition, it may not become general truth.

It must either:

- stay scoped
- ship with an adapter plan
- remain proposal-only

### 5. Durability Gate

Accepted wins must survive a later re-check before they count as durable.

Durability is the bridge from `2` to `3`.

## Phase-1 Auto-Merge Allowlist

Phase-1 `3` should only auto-merge low-risk change classes.

Allowed:

- runtime-policy artifacts
- docs/spec updates tied to accepted behavior
- loop thresholds
- scoring formulas
- evaluation heuristics
- promotion metadata
- branch bookkeeping

Proposal-branch only:

- storage schema
- memory repair semantics
- hive coordination protocol
- claim/task mutation rules
- API contract changes
- broader persistence behavior

## Artifacts

True self-evolution needs first-class artifacts, not ad hoc files.

### Evolution Manifest

Per proposal or branch:

- candidate id
- origin signal
- scope class
- allowed write surface
- baseline snapshot
- evaluation plan
- rollback plan

### Promotion Proposal

Written when a proposal is accepted:

- branch
- result summary
- metrics
- evidence
- merge eligibility
- durability due time

### Authority Ledger

Tracks autonomy by change class:

- precision
- rollback rate
- durability rate
- current authority tier

### Durability Ledger

Tracks whether merged changes remain good later.

This is what turns a win into durable truth.

## Branch Lifecycle

1. detect pressure from loops or gaps
2. generate bounded candidate
3. classify scope and risk
4. open isolated evolution branch when code change is needed
5. run evaluation gates
6. reject or accept as proposal
7. merge only if allowed by the current authority tier
8. re-check later for durability
9. promote to durable truth or demote

## Why Separate Branches Matter

Separate branches isolate experiments.

They do not, by themselves, make evolution safe.

Safety still requires:

- write-surface allowlists
- rebase and merge checks
- portability checks
- durability re-checks

Branch isolation is necessary, not sufficient.

## Initial Implementation Order

1. add evolution proposal artifact and schema
2. add scope classifier and allowlist enforcement
3. add branch manifest and proposal branch naming
4. add `accepted_proposal` versus `durable_truth` telemetry split
5. add authority ledger
6. add narrow phase-1 auto-merge for low-risk classes
7. add durability re-check queue

## Verification

Required coverage:

- proposal generation from weak loop pressure
- scope gate rejects forbidden write surfaces
- accepted proposal does not imply durable truth
- durability re-check can demote a merged change
- authority expands after repeated durable wins
- authority contracts after regressions or rollback spikes

## Non-Goals

Not phase 1:

- unrestricted self-editing
- silent mutation of the active user branch
- auto-merge for storage, coordination, or API semantics
- treating one accepted run as permanent truth

## Summary

The 10-star version is not "more auto-merge."

It is a layered self-evolution system that:

- changes itself at the cheapest valid layer first
- isolates risky changes on dedicated branches
- separates accepted wins from durable truth
- earns more authority only after proving reliability
