---
phase: A5
name: CrossSessionRecall Bench
version: v5
status: planned
opened: 2026-04-22
depends_on: [V4]
axis: session_continuity
---

# Phase A5: CrossSessionRecall Bench

## Goal

Ship a runnable bench that plants N facts in session 1 and queries for them in session K (K=2,4,8). Measures memd's substrate promise: session continuity as a number, not a feeling.

## Why this phase exists

Public benches measure flat RAG-over-transcript; none of them cut sessions, force compaction, and re-query. V4 A4 proves ledger survives compaction; A5 turns that into a measurable.

## Deliver

1. **Bench spec.** YAML schema under `.memd/benchmarks/substrate/cross-session-recall.yaml` defining scenarios.
2. **Scenario generator.** N=20 / 50 / 100 facts, K cuts in {2,4,8}. Mix canonical + semantic + preference.
3. **Runner.** `memd bench substrate --suite cross-session-recall` invokes generator, runs sessions, collects metrics.
4. **Metrics.** `recall@1`, `recall@3`, `answer-exact-match` per cut; token-cost-per-recall; median latency.
5. **Baseline.** Run against memd current, lock canonical numbers as floor.
6. **Scorer.** Deterministic exact-match + cached LLM-judge fallback (codex-lb gpt-5.4).

## Pass Gate

- pre: no bench, no number
- post: suite runs on CI, produces NDJSON + markdown report; canonical numbers floor = recall@3 ≥ 0.90 at K=2, ≥ 0.80 at K=8
- evidence: `.memd/benchmarks/substrate/results/cross-session-recall-YYYY-MM-DD.ndjson` + report
- regression budget: any pass-gate regression blocks merge

## Product Win

"memd remembers X at session 8" becomes a claim with a number behind it.

## Evidence

- bench spec YAML
- 10 CI runs
- markdown report in `docs/verification/SUBSTRATE_BENCHMARKS.md`
- reproducibility test: second clone matches ±0.03

## Fail Conditions

- Floor missed: root-cause (A4 regression, retrieval drift). Do not tune scorer to pass.
- LLM-judge cost blown: reduce K range, keep exact-match only.

## Rollback

Bench runs behind `memd bench substrate`; no runtime behavior change.
