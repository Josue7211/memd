# memd Roadmap

`ROADMAP.md` is the single roadmap source of truth for this repo.

<!-- ROADMAP_STATE
truth_date: 2026-04-16
version: v2
version_status: in_progress
current_milestone: M3
milestone_status: in_progress
current_phase: P2
phase_status: pending
next_milestone: M4
next_step: P2 — Measurement Proof (per-kind token counter, CI benchmark gate, diagnostics report). J2+O2 complete.
active_blockers: []
v1_status: frozen_architecture_complete
note: M3 in progress. J2 verified. O2 verified 2026-04-16 (decay diagnostics endpoint, sensitivity analysis, consolidation quality scoring, A/B recall test, defaults 21/0.12/14.0 confirmed). 166 server tests passing. P2 unblocked.
-->

## Status Snapshot

- truth date: `2026-04-16`
- current version: `v2` (hardening)
- version status: `in_progress`
- v1 status: `frozen` — architecture complete, operations broken (honest score: 1.8/10)
- current milestone: `M3: Make It Provable` (10-STAR Tier 3)
- current phase: J2 (Isolation + Trust) — pending
- completed: `M0` (verified), `M1` (verified 2026-04-15, eval 95), `M2` (verified 2026-04-16)
- M1: `verified` — B2+C2+F2 pass gates, remote deployed, eval 95
- M2: `verified` — D2+G2+E2+H2 pass gates, 624 tests, benchmarks zero regression, node verification 15✓/6~/0✗, remote deployed
- next step: M3 planning — J2 (Isolation + Trust) phase research
- M0 benchmark baseline: LongMemEval 82.8%, LoCoMo 41.5%, MemBench 34.6%, ConvoMem 0.0% (retrieval-only)
- prior M1 benchmark: LongMemEval 90% full-eval (50 items, LLM-graded, `session_recall_any@10`=96%). Retrieval-only baseline (500 items) was 82.8%. These are different metrics — do not compare directly.
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
| B2 | Signal vs Noise | `verified` | 1, 2 | [[phase-b2-signal-vs-noise]] | [[memd-theory-lock-v1]] |
| C2 | Ghost Cleanup | `verified` | 4, 5 | [[phase-c2-ghost-cleanup]] | [[memd-theory-lock-v1]] |
| F2 | Ingestion Pipeline | `verified` | 3, 8 | [[phase-f2-ingestion-pipeline]] | [[memd-theory-lock-v1]] |

**Verified 2026-04-15**: All three phases pass gates. Eval score 95 (gate >= 65).
Commits: `566feff` (B2), `d959c36` (C2). F2 no code changes (pipeline existed).
Remote server deployed at services VM via systemd user service.

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-1]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Feature registry: [[docs/verification/FEATURES.md]]
- Live loop flow: [[docs/core/architecture.md#live-loop]]

**Execution plan**: [[docs/plans/M1-EXECUTION-PLAN.md]]
**Gate**: All nodes pass M1 tier. Live loop runs end-to-end.
**Test**: Store preference → new session → wake surfaces it. Stale records gone.

### M2: Make It Correct — Tier 2 (10-STAR gaps 10-17) — VERIFIED

Fix architectural gaps.

| Phase | Name | Status | Gaps | Phase Doc | Theory Lock |
| --- | --- | --- | --- | --- | --- |
| D2 | Correction Flow | `verified` | 10, 15, 16 | [[phase-d2-correction-flow]] | [[memd-canonical-promotion-theory-lock-v1]] |
| E2 | Atlas Activation | `verified` | 13, 14 | [[phase-e2-atlas-activation]] | [[memd-atlas-theory-lock-v1]] |
| G2 | Lane Architecture | `verified` | 17 | [[phase-g2-lane-architecture]] | [[memd-lane-theory-lock-v1]] |
| H2 | Recall Proof | `verified` | 11, 12 | [[phase-h2-recall-proof]] | [[memd-evaluation-theory-lock-v1]] |

**Server-side progress (2026-04-15)**:
- D2: Entity-based contradiction detection (old_item entity lookup), correction tag boost, preferred=true, trust_rank hierarchy. 3 tests added (e2e correction, contradiction marks siblings contested, existing 6 pass).
- G2: Lane tag in compact_record/wake packet. Lane diversity admission (cap=5 per lane). Backfill lanes wired to maintain. 
- E2: Salience gate removed from auto_link_entity (new entities start at 0.0). Entity link backfill wired to maintain. Atlas navigation test (4 hops). 
- H2: Cross-session correction persistence test passes. Cross-harness retrieval test passes. Eval-framework items remain: correction retention eval, lane relevance eval, A/B influence test, benchmark re-run.
- Total: 623 tests, 0 failures (+5 new M2 tests from 618 baseline).

**Decisions logged**:
- Contradiction detection only works for path-based entities (shared source_path). Content-based entities have different canonical_key → different entity → no siblings. This is a design limitation, not a bug. Future: topic-extraction entity keys.
- Query lane boost (G2.2) implemented — `query: Option<String>` added to WorkingMemoryRequest. `detect_content_lane` runs on query text to detect lane context. Differential scoring: +0.08 same-lane match, +0.02 different-lane, +0.06 no-query-context (backward compat). Reasons trace includes `lane_match`/`lane_mismatch`. 2 unit tests added. CLI `memd working --query "..."` wired.
- Entity link backfill findings appear in API response but not persisted payload_json. Non-blocking data gap.

**Remaining for M2 gate**:
- [x] H2 correction retention eval — passive checks in eval_bundle_memory (superseded leak detection)
- [x] H2 lane diversity eval — passive check in eval_bundle_memory (lane diversity)
- [x] H2 A/B influence test — server test (h2_ab_influence_corrections_improve_retrieval)
- [x] H2 benchmark re-run — LME 82.8% (gate 80%), LoCoMo 41.5% (gate 41.5%), MemBench 34.6% (gate 30%) — zero regression
- [x] Node-by-node code-level verification — 15 ✓, 6 ~, 0 ✗. [[docs/verification/milestones/M2-NODE-VERIFICATION.md]]
- [x] Deploy new binary to remote server + smoke test — deployed to openclaw via systemd user service, correction flow verified

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-2]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Node verification: [[docs/verification/milestones/M2-NODE-VERIFICATION.md]]
- Feature registry: [[docs/verification/FEATURES.md]]

**Execution plan**: [[docs/plans/M2-EXECUTION-PLAN.md]]
**Gate**: All nodes pass M2 tier. Correction→recall proven. Atlas navigable.
Lanes are DB-tag routing. Cross-harness works. Benchmark re-run no regression.
**VERIFIED 2026-04-16**: 626 tests, 0 failures. Node verification 15✓/6~/0✗. Benchmarks zero regression.
Binary deployed to openclaw. Correction flow smoke-tested on remote. G2.2 query lane boost implemented (differential: +0.08 same-lane, +0.02 different-lane, +0.06 no-query-context).

### M3: Make It Provable — Tier 3 (10-STAR gaps 18-23)

Fix measurement gaps.

| Phase | Name | Status | Gaps | Phase Doc | Theory Lock |
| --- | --- | --- | --- | --- | --- |
| J2 | Isolation + Trust | `verified` | 20, 23 | [[phase-j2-isolation-trust]] | [[memd-theory-lock-v1]] |
| O2 | Lifecycle Calibration | `verified` | 21, 22 | [[phase-o2-lifecycle-calibration]] | [[memd-canonical-promotion-theory-lock-v1]] |
| P2 | Measurement Proof | `pending` | 18, 19 | [[phase-p2-measurement-proof]] | [[memd-evaluation-theory-lock-v1]] |

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-3]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Benchmark registry: [[docs/verification/benchmark-registry.json]]
- Public benchmarks: [[docs/verification/PUBLIC_BENCHMARKS.md]]

**Execution plan**: [[docs/plans/M3-EXECUTION-PLAN.md]]
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
