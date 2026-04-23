---
milestone: v7
name: Correction E2E Behavior Change
status: planned
opened: 2026-04-22
revised: 2026-04-22
depends_on: [v6]
composite_pre: 4.45
composite_target: 4.90
axes_lifted: [session_continuity, correction_retention, trust_provenance]
axes_integrated_with: []
---

# Milestone v7 Audit — Correction E2E Behavior Change

## Goal

Prove correction behavior-change across a session boundary. V4 (C4 phase) owns
ingestion 1→4 (user says "X is Y", it's captured); V7 owns behavior change
4→5 ("session 2 retrieval uses Y not X without re-prompting"). User corrects
fact in S1; S2 sees the corrected value; provenance shows the correction
turn; user can rollback if correction was wrong. Closes the E2E loop that
V5 B5 measurement validated but did not prove operationally.

## 10-STAR axis targets (pre / post)

Baseline from V6 post (0.1.0-CONTRACT.md): SC=4, CR=4, TP=4. V7 lifts three
axes via behavior-change proof.

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity    | 20% | 4 | 5 | A7+C7+E7: correction in S1 focus context influences S2 wake (C7 metric, E7 chain, SC continuity survives correction loop) |
| correction_retention  | 15% | 4 | 5 | C7: S2 retrieval uses corrected value not prompt-repeated; B5 propagation passes, C7 next_session_behavior_rate ≥ 0.05 lift |
| trust_provenance      | 10% | 4 | 5 | E7+F7: every displayed value answers "from turn T, corrected at U?" within 2 clicks; provenance chain complete 1.000 |
| procedural_reuse      | 15% | 4 | 4 | no V7 work — maintained from V5 |
| cross_harness         | 15% | 4 | 4 | no V7 work — maintained from V5 |
| raw_retrieval         | 15% | 7 | 7 | no V7 work — maintained from V6 |
| token_efficiency      | 10% | 4 | 4 | no V7 work — maintained from V4 |

**Composite: 4.45 → 4.90** (weighted arithmetic: 0.20×5 + 0.15×5 + 0.10×5 + 0.15×4 + 0.15×4 + 0.15×7 + 0.10×4 = 4.90).

## Phases

- **A7** Correction lane ingestion — verify V4 C4 capture at scale (miss-rate ≤ 5%).
- **B7** Correction → canonical promotion rule — promotion_correctness_rate lifts ≥ 0.05.
- **C7** Next-session behavior change test — planted correction honored in S2; next_session_behavior_rate lifts.
- **D7** Contradiction detection — conflicts surfaced not merged.
- **E7** Provenance trail — corrected record carries source-turn pointer; chain_completeness = 1.000.
- **F7** "I learned X from Y" surface — user-visible correction log with turn backrefs.
- **G7** Rollback — user can undo correction mid-session; provenance and behavior verified.
- **H7** Atomic-commit-by-default — every memd write path (checkpoint, hook capture, canonical promotion, correction ingestion) atomically commits dirty tracked files in the host repo. Default ON; toggleable via `memd configure auto_commit=false` (V8 owns the CLI surface). Rationale: corrections and session state must be durable on disk at commit granularity, not dangling in working tree. Adds: new config key `auto_commit.enabled` (default true), unified writer-guard that commits-before-write for dirty repos, per-write-path opt-out for non-stateful operations (e.g. `memd lookup` does not commit). No axis credit claimed directly — durability is necessary condition for CR +1 and TP +1 lifts to survive crashes.

## Completion gate

1. V4 C4 capture validated (A7): real 30-day trace miss-rate ≤ 5%.
2. V5 B5 CorrectionPropagation suite passes 100% + 7-day clean.
3. C7 scenario (5 corrections planted, S2 queries) passes with next_session_behavior_rate ≥ 0.05 delta.
4. E7 chain_completeness = 1.000 hard gate.
5. G7 3-session dogfood (S1: 5 corrections + 2 contradictions + 1 rollback-target; S2: query each; S3: re-query + mid-session rollback): 10/10 over 7 days, provenance clean.
6. 10-STAR composite ≥ 4.90, zero axis below pre-value.
7. No blocker backlog on `axis: correction_retention` or `axis: session_continuity`.

Evidence: G7 harness NDJSON + 10-STAR scorecard regenerator run in `docs/verification/release-0-1-0/`.

## Per-axis harness assertions (required for axis credit)

| axis | concrete assertion | fixture | owner phase |
| --- | --- | --- | --- |
| session_continuity    | S1 correction lifts focus context; S2 wake reflects it; no re-prompt needed | G7 dogfood scenario | E7 (chain) + C7 (behavior) |
| correction_retention  | S1 T5 "author ≠ Tolstoy"; S2 query "who wrote War & Peace?" returns corrected value not prompt-repeated | C7 scenario + G7 | C7 |
| trust_provenance      | every correction: `memd fact provenance <fact-ulid>` shows turn T + correction turn C within 2 queries; chain_completeness = 1.000 | E7 audit + F7 surface | E7 |

Missing any assertion → axis does not lift, milestone does not close.

## V4 C4 Separation (binding)

This is the critical distinction from V4:

- **V4 C4 (ingestion 1→4)**: Correction captured, stored, accessible via API. G4 harness proves it's there.
- **V7 C7+E7 (behavior-change 4→5)**: Correction is **automatically used** in retrieval without re-prompting. Different code path. G7 harness proves behavior change across session boundary.

A milestone that claims +1 on CR without proving behavior-change lift (i.e., without C7 next_session_behavior_rate delta) is gaming the contract. This is the highest-risk phantom claim; block at review.

## Non-goals

- procedural_reuse lift (V5 owns)
- cross_harness lift (V5 owns)
- raw_retrieval lift (V6 owns)
- token_efficiency lift (V4 owns)
- automatic correction (V10)
- cross-user correction (V9)

## Feature-flag graduation

| Flag | Phase | Day | Condition |
| --- | --- | --- | --- |
| `MEMD_B7_CORRECTION_PROMOTE` | B7.7 | 7 | after 7-day clean, promotion_correctness_rate ≥ 0.90 |
| `MEMD_LEARNED_SURFACE` | F7 | N/A | production default, no gate |
| `MEMD_V7_ALLOW_BELOW_TARGET` | G7 | N/A | permanent = 0 (hard floor) |

## Changelog

- 2026-04-22 opened.
- 2026-04-22 revised: composite_pre 7.0 → 4.45 (V6 post, zero-generosity 0.1.0-CONTRACT);
  composite_target 7.8 → 4.90 (exact per contract); axes_lifted corrected to
  [session_continuity, correction_retention, trust_provenance] with binding
  V4 C4 separation rule; per-axis harness assertions added; flag-graduation
  calendar added; non-goals clarified to exclude non-owned axes.
- 2026-04-22 revised: added H7 phase — atomic-commit-by-default primitive.
  Default ON; `memd configure auto_commit.enabled=false` toggles OFF per
  operator choice (V8 owns the configure CLI surface). Durability precondition
  for CR and TP axis lifts surviving crashes; no direct axis credit claim.
