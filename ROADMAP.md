# memd Roadmap

`ROADMAP.md` is the single roadmap source of truth for this repo.

<!-- ROADMAP_STATE
truth_date: 2026-04-15
version: v2
version_status: in_progress
current_milestone: M1
milestone_status: reopened
current_phase: triage
phase_status: in_progress
next_milestone: M2
next_step: M1 Tier 1 — fix operational pipeline (status noise, working memory, expiry, preferences, lanes)
active_blockers: ["memd-preferences-not-persisted-across-sessions", "working-memory-stale-records", "pipeline-lifecycle-broken"]
v1_status: frozen_architecture_complete
note: Priority reset 2026-04-15. Milestones now follow 10-STAR tiers exactly. M1=Tier1 (make it work), M2=Tier2 (make it correct), M3=Tier3 (make it provable), M4=Tier4 (make it 10-star). Dashboard is M4. Every stage of the 10-star model must be verified working in order.
-->

## Status Snapshot

- truth date: `2026-04-15`
- current version: `v2` (hardening)
- version status: `in_progress`
- v1 status: `frozen` — architecture complete, operations broken (honest score: 1.8/10)
- current milestone: `M1: Make It Work` (**reopened** — maps to 10-STAR Tier 1)
- current phase: triage — determining which operational gaps to fix first
- completed: `M0` (verified)
- M1: `reopened` — operational pipeline broken (gaps 1-9 from 10-STAR)
- M2: `reopened` — architectural correctness unverified (gaps 10-17 from 10-STAR)
- next step: M1 — fix operational pipeline so core memory actually works
- M0 benchmark baseline: LongMemEval 82.8%, LoCoMo 41.5%, MemBench 34.6%, ConvoMem 0.0% (retrieval-only)
- prior M1 benchmark: LongMemEval 96.0% (+13.2%) — but gate test was narrow, real usage proves core is broken
- 10-STAR composite: 1.8/10 (zero-generosity regrade 2026-04-14)

## Blockers

- **memd-preferences-not-persisted-across-sessions** (critical, core): Agents don't retain architecture decisions or workflow conventions across sessions. This breaks memd's core value prop. See `docs/backlog/2026-04-15-memd-preferences-not-persisted-across-sessions.md`.
- **working-memory-stale-records** (critical, core): Completed phase status (B2) still occupies working memory slots weeks after verification. Expiry pipeline never runs on phase completion. Stale records eat budget that should hold architecture decisions.
- **pipeline-lifecycle-broken** (critical, core): promote/expire/archive lifecycle doesn't execute in production. M1 gate tested store→recall on a single fact but never tested lifecycle. Records accumulate forever, working memory fills with noise.

## Process

- Status rules, phase-flip rules, product contract: [[docs/policy/INDEX.md]]
- V1 phases (frozen): [[docs/verification/milestones/MILESTONE-v1.md]]
- V1 → V2 migration mapping: [[docs/verification/MEMD-10-STAR.md]]

## V2 Milestones (Hardening — Make It Real)

Goal: 1.8/10 → 10/10. No new architecture. Make existing architecture work.

Milestones follow the 10-STAR tiers exactly. Each tier fixes a class of gaps.
Every node in the architecture graph (see `docs/core/architecture.md` mermaid diagram)
must pass verification at each milestone gate before moving to the next. No skipping.

Each phase has a Ralph doc (bounded goal, pass gate, evidence, rollback).
Benchmarks re-run at every milestone gate. Load one phase doc at a time.

Every milestone gate verifies every node in the architecture graph. Per-node
criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]

### M0: Baseline + Research (no code changes)

Establish the "before" number. Extract patterns from competitors.

| Phase | Name | Status | Backlog | Detail |
| --- | --- | --- | --- | --- |
| A2 | Inspiration Extraction | `verified` | #55 | [[docs/phases/phase-a2-inspiration-extraction.md]] |
| — | Benchmark Baseline | `verified` | #45, #60 | LME 82.8%, LoCoMo 41.5%, MB 34.6%, CM 0.0% (retrieval-only) |

**Gate**: Extraction notes for 8+ targets. Benchmark numbers recorded. **PASSED 2026-04-14.**

### M1: Make It Work — Tier 1 (REOPENED — 10-STAR gaps 1-9)

Fix the operational pipeline. Every stage of the live loop must function.

| Phase | Name | Status | Gaps | Phase Doc | Theory Lock |
| --- | --- | --- | --- | --- | --- |
| B2 | Signal vs Noise | `reopened` | 1, 2 | [[phase-b2-signal-vs-noise]] | [[memd-theory-lock-v1]] |
| C2 | Ghost Cleanup | `reopened` | 4, 5 | [[phase-c2-ghost-cleanup]] | [[memd-theory-lock-v1]] |
| F2 | Ingestion Pipeline | `reopened` | 3, 8 | [[phase-f2-ingestion-pipeline]] | [[memd-theory-lock-v1]] |

**Why reopened**: Gate tested one synthetic fact. Real usage: stale records never expire,
preferences lost every session, pipeline lifecycle doesn't run.

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-1]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Feature registry: [[docs/verification/FEATURES.md]]
- Live loop flow: [[docs/core/architecture.md#live-loop]]

**Execution plan**: [[docs/plans/M1-EXECUTION-PLAN.md]]
**Gate**: All nodes pass M1 tier. Live loop runs end-to-end.
**Test**: Store preference → new session → wake surfaces it. Stale records gone.

### M2: Make It Correct — Tier 2 (10-STAR gaps 10-17)

Fix architectural gaps.

| Phase | Name | Status | Gaps | Phase Doc | Theory Lock |
| --- | --- | --- | --- | --- | --- |
| D2 | Correction Flow | `reopened` | 10, 15, 16 | [[phase-d2-correction-flow]] | [[memd-canonical-promotion-theory-lock-v1]] |
| E2 | Atlas Activation | `reopened` | 13, 14 | [[phase-e2-atlas-activation]] | [[memd-atlas-theory-lock-v1]] |
| G2 | Lane Architecture | `reopened` | 17 | [[phase-g2-lane-architecture]] | [[memd-lane-theory-lock-v1]] |
| H2 | Recall Proof | `reopened` | 11, 12 | [[phase-h2-recall-proof]] | [[memd-evaluation-theory-lock-v1]] |

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-2]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Feature registry: [[docs/verification/FEATURES.md]]

**Gate**: All nodes pass M2 tier. Correction→recall proven. Atlas navigable.
Lanes are DB-tag routing. Cross-harness works.

### M3: Make It Provable — Tier 3 (10-STAR gaps 18-23)

Fix measurement gaps.

| Phase | Name | Status | Gaps | Phase Doc | Theory Lock |
| --- | --- | --- | --- | --- | --- |
| J2 | Isolation + Trust | `pending` | 20, 23 | [[phase-j2-isolation-trust]] | [[memd-theory-lock-v1]] |

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-3]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Benchmark registry: [[docs/verification/benchmark-registry.json]]
- Public benchmarks: [[docs/verification/PUBLIC_BENCHMARKS.md]]

**Gate**: All nodes pass M3 tier. LongMemEval ≥ 80%. Token efficiency measured.
Decay calibrated. Benchmark ≥ 90%.

### M4: Make It 10-Star — Tier 4 (10-STAR gaps 24-35)

Product gaps. Dashboard last.

| Phase | Name | Status | Gaps | Phase Doc | Theory Lock |
| --- | --- | --- | --- | --- | --- |
| K2 | Observability | `pending` | 32 | [[phase-k2-observability]] | [[memd-theory-lock-v1]] |
| L2 | Hive Hardening | `pending` | 28, 33, 34, 35 | [[phase-l2-hive-hardening]] | [[memd-hive-theory-lock-v1]] |
| M2-evo | Overnight Evolution | `pending` | 24, 25 | [[phase-m2-overnight-evolution]] | [[memd-theory-lock-v1]] |
| N2 | Integrations Polish | `pending` | 26, 29, 30, 31 | [[phase-n2-integrations-polish]] | [[memd-theory-lock-v1]] |
| I2 | Human Dashboard | `pending` | 27 | [[phase-i2-human-dashboard]] | — |

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-4]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Feature registry: [[docs/verification/FEATURES.md]]

**Gate**: All nodes pass M4 tier. Private items don't leak. Evolution proposes procedures.
Dashboard: browse, correct, navigate in browser. Zero console errors. Benchmark ≥ 90%.
**Demo**: "Store a fact. Correct it. Navigate it. Prove it. All in the UI."

## Benchmarks

Continuous gate at every milestone. Regression = stop.
Protocol + cadence: [[docs/verification/PUBLIC_BENCHMARKS.md]]

## Mining

A3 donor-to-phase mapping: [[docs/theory/2026-04-14-donor-extraction-to-v2-phases.md]]

## Backlog

83 items tracked. Full index: `docs/backlog/` directory.
Summary: [[docs/verification/MEMD-10-STAR.md#complete-gap-inventory]]

## Reference Docs

- [[docs/core/setup.md|Setup and harness behavior]]
- [[docs/verification/milestones/MILESTONE-v1.md|Milestone v1 verification]]
- [[docs/strategy/research-loops.md|Research loops]]
- [[docs/theory/models/2026-04-11-memd-ralph-roadmap.md|Detailed Ralph roadmap spec]]
- [[docs/theory/models/2026-04-11-memd-canonical-theory-synthesis.md|Canonical theory synthesis]]

## Non-Goals

- transcript dumping
- vendor lock-in
- using RAG as the only memory layer
- mixing project-local truth with global truth
- letting one provider silently overwrite another provider’s memory
