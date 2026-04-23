---
milestone: v5
name: Substrate-Native Benchmark Suite
status: planned
opened: 2026-04-22
revised: 2026-04-22
depends_on: [v4, ../../verification/0.1.0-CONTRACT.md, ../../verification/0.1.0-AXIS-OWNERSHIP.md]
composite_pre: 3.45
composite_target: 4.20
axes_lifted: [procedural_reuse, cross_harness, raw_retrieval]
axes_integrated_with: [correction_retention]
---

# Milestone v5 Audit — Substrate-Native Benchmark Suite

## Goal

Ship memd's own benchmark suite. Open-source, reproducible, runnable by competitors. Public benches (LME, LoCoMo, MemBench, ConvoMem) measure flat RAG-over-transcript; V5 benches measure what memd is actually for — cross-session recall, correction propagation, cross-harness handoff, progressive depth, provenance integrity, typed retrieval, adversarial noise resistance. Owns three axis lifts per 0.1.0-AXIS-OWNERSHIP.md: PR +2 (2→4), CH +1 (3→4), RR +2 (4→6).

## 10-STAR axis targets (pre / post)

Scores match the 0.1.0-CONTRACT.md baseline (V4 post) and target per ownership table.

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 4 | 4 | no V5 work — non-goal per AXIS-OWNERSHIP |
| correction_retention | 15% | 4 | 4 | B5 measures C4 work; **integrates only, no credit** per AXIS-OWNERSHIP |
| procedural_reuse     | 15% | 2 | 4 | F5 typed-retrieval live-fire: routine detection via session 1 plant → session 2 invocation; token-savings ≥ baseline vs F4.7 observation-only |
| cross_harness        | 15% | 3 | 4 | C5 cross-harness bench verifies claude-code writes visible + stable in codex reads and round-trip |
| raw_retrieval        | 15% | 4 | 6 | A5 cross-session, D5 progressive-depth, E5 provenance-integrity, F5 typed-retrieval combined coverage; 7-suite substrate bench suite lives at `docs/verification/SUBSTRATE_BENCHMARKS.md` |
| token_efficiency     | 10% | 4 | 4 | no V5 work — non-goal per AXIS-OWNERSHIP |
| trust_provenance     | 10% | 3 | 3 | no V5 work — non-goal per AXIS-OWNERSHIP |

**Composite: 3.45 → 4.20** (weighted arithmetic: 4×0.20 + 4×0.15 + 4×0.15 + 4×0.15 + 6×0.15 + 4×0.10 + 3×0.10 = 4.20).

### Why procedural_reuse gains +2, not +3

V4's F4.7 seeded dead-path instrumentation but claimed no behavior credit (PR stayed at 2). V5 F5 wires routine-detection live-fire: facts planted in session 1 accessed in session 2 are cached as "routine X"; session 3 or later, routine X is invoked (not re-queried), producing token savings vs F4.7's observation-only path. Harness assertion: routine observed in S1, invoked in S2+, token_savings(routine) ≥ baseline_retrieval_cost, measured via per-routine cost ledger in F5 scorer. This is behavior credit, not instrumentation.

All regenerations scoring PR > 4 without live-fire invocation assertion failing are invalid.

## Suites (per V5 integration doc)

- **A5 CrossSessionRecall** — plant facts in session 1, query in session N, measure recall.
- **B5 CorrectionPropagation** — correct fact in session 1, assert session N retrieval uses corrected version and provenance shows the correction turn. **Integrates with C4 measurement.**
- **C5 CrossHarnessContinuity** — write in claude-code, read in codex, round-trip. Truth conserved, visibility honored.
- **D5 ProgressiveDepth** — wake/lookup/resume ladder. Shallow wake gets summary; resume reconstructs task.
- **E5 ProvenanceIntegrity** — every retrieved record carries source. Unsourced record in result set = fail.
- **F5 TypedRetrieval** — query shape routes to right type; wrong-type result penalized. **Owns routine-detection live-fire for PR lift.**
- **G5 AdversarialNoise** — plant wrong facts alongside canonical; memd must surface canonical, not noise.

## Completion gate

All 7 suites runnable via `memd bench substrate --all`, numbers published in `docs/verification/SUBSTRATE_BENCHMARKS.md`, reproducible within ±0.03 per suite on fresh clone. Mandatory: **No axis credit without harness proof** (per 0.1.0-CONTRACT.md). V5 claims three axis lifts; all three must have concrete harness fixtures + assertions in per-axis table below.

- Composite ≥ 4.20 in SUBSTRATE_BENCHMARKS.md via G5 aggregator run.
- Every axis lift backed by harness fixture + assertion (see Per-axis Harness Assertions below).
- V5 does not claim credit on SC, TE, TP (per AXIS-OWNERSHIP.md).
- Competitor (one of mempalace / supermemory / letta / mem0) runs the suite against their product; scorecard published.

## Per-axis harness assertions (required for axis credit)

| axis | concrete assertion | fixture | suite owner |
| --- | --- | --- | --- |
| procedural_reuse | routine X observed in S1, invoked in S2+, token_savings(X) ≥ 1×baseline_retrieval_cost | F5 scorer + cost ledger | F5 |
| cross_harness | write in claude-code S1 present + consistent in codex S2 and back in claude-code S3 | C5 cross-harness-continuity runner | C5 |
| raw_retrieval | combined recall@K across 7 suites ≥ baseline, per SUBSTRATE_BENCHMARKS.md | A5, D5, E5, F5, G5 aggregator | A5+G5 |

Missing any assertion → axis does not lift, milestone does not close.

## Non-goals

- **session_continuity** — non-goal per AXIS-OWNERSHIP.md (V4 owns this, V7 owns next lift)
- **correction_retention lift** — **measurement only, no credit** per AXIS-OWNERSHIP.md; B5 verifies C4's work, produces evidence for next milestone's audit trail
- **token_efficiency** — non-goal per AXIS-OWNERSHIP.md (V4 owns this, V8 owns next lift)
- **trust_provenance** — non-goal per AXIS-OWNERSHIP.md (V4 owns this, V7 owns next lift)
- tuning the suites to make memd look good — suites are honest or they don't ship
- merging substrate benches with public benches — they measure different things

## Flag-graduation calendar

V5 uses fewer flags than V4; most substrate behavior is always-on once wired. One flag owns milestone closure:

1. `MEMD_SUBSTRATE_AGG_PARALLEL` = 1 default after G5 Task G5.7 7-day clean window (1 of 5 windows needed for F4 corrections + F4.7 seed to graduate post-flag-gate).

No other V5-owned flags at release. F5 `--explain-route` ships always-on in CLI; routing behavior is intrinsic, not feature-flagged.

**Calendar spillover:** G5 harness + aggregator complete at V5 code close. One 7-day window (G5 window 1) runs in parallel with V6 planning/A6 execution. V5 flag graduation does not block V6 entry.

## Changelog

- 2026-04-22 opened (plan-spec phase).
- 2026-04-22 revised: composite_pre 4.0 → 3.45 (reconciled with 0.1.0-CONTRACT baseline post-V4); composite_target 5.5 → 4.20 (per AXIS-OWNERSHIP contract); axes_lifted demoted from 3 axes to owned-only (PR, CH, RR — removed non-existent `typed_retrieval` axis, added `raw_retrieval`); added `axes_integrated_with` field for CR measurement-only; procedural_reuse assertion clarified as "behavior credit via live-fire invocation, not observation"; per-axis harness assertions table added to enforce "no axis credit without harness proof" rule; flag-graduation calendar added with 1-window note (1 of 5 for F4 graduation); non-goals section made explicit (SC, CR-lift, TE, TP).
