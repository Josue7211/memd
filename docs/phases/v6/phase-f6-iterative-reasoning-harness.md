---
phase: F6
name: Iterative Reasoning Harness + V6 Completion Gate
version: v6
status: planned
opened: 2026-04-22
depends_on: [A6, B6, C6, D6, E6]
axis: raw_retrieval, token_efficiency, trust_provenance
plan_spec: docs/phases/v6/phase-f6-plan.md
---

# Phase F6: Iterative Reasoning Harness + V6 Completion Gate

## Goal

Two jobs: (1) add a multi-step reasoning harness over typed memory for temporal-reasoning question types (LME temporal subset, LoCoMo sequential-reasoning); (2) close V6 — regenerate PUBLIC_BENCHMARKS.md, update method cards, write MILESTONE-v6.md, lift 10-STAR composite ≥ 7.0.

## Why this phase exists

Some questions ("what did the user say about flights before mentioning the hotel?") need sequenced lookups: first find the hotel mention, then scope time-before. A plain retriever can't express this. F6 adds a scratchpad reasoning harness that chains lookups over canonical + semantic + episodic. Then F6 is the gate — aggregated numbers and method cards.

## Deliver

1. **Reasoning harness.** `crates/memd-client/src/benchmark/typed_ingest/reasoning.rs` — multi-step scratchpad that chains typed lookups, up to 5 steps, with intermediate-state persistence.
2. **Reasoning card.** `docs/contracts/iterative-reasoning.md` — step schema, termination rules, trace format.
3. **Temporal subset run.** Isolated baseline on LME temporal + LoCoMo sequential subsets with and without reasoning harness.
4. **Aggregated V6 report regenerator.** Rewrites `docs/verification/PUBLIC_BENCHMARKS.md` with all four canonical numbers, method-card links, V6 delta history.
5. **Method cards.** Per-bench: `docs/verification/method-cards/{lme,locomo,membench,convomem}-v6.md` — what typed layers were on, seeds, compiler budgets, routing policy, reasoning-harness usage.
6. **10-STAR composite.** G-style regenerator writes `docs/verification/MEMD-10-STAR.md` with V6 axis deltas; refuses composite < 7.0 unless `--allow-below-target`.
7. **Reproducibility script.** `scripts/public-bench-reproduce.sh` matches numbers ±0.03 from fresh clone.
8. **MILESTONE close.** `docs/verification/milestones/MILESTONE-v6.md` filled in; ROADMAP flipped.

## Pass Gate

- pre: no reasoning harness; numbers scattered
- post canonical (sidecar OFF):
  - LME `qa_accuracy` ≥ 0.85
  - LoCoMo `token_f1_avg` ≥ 0.75
  - MemBench `mc_accuracy` ≥ 0.75
  - ConvoMem LLM-judge `accuracy` ≥ 0.90
  - Retrieval diagnostic `session_recall_any@5` ≥ 0.95 on LME (no regression)
  - 10-STAR composite ≥ 7.0
- evidence: per-bench NDJSON, method cards, reproducibility run, MILESTONE-v6 filled
- regression budget: any canonical regression blocks close

## Product Win

memd ships honest public-canonical numbers that other systems are measured against. 10-STAR claim earned.

## Evidence

- method cards (4)
- reasoning card
- regenerated PUBLIC_BENCHMARKS.md
- regenerated MEMD-10-STAR.md
- reproducibility script pass
- MILESTONE-v6.md

## Fail Conditions

- Any canonical target missed: publish nothing until root-cause. No gaming.
- Reproducibility script off by >0.03 on any metric: fix seed-handling before close.

## Non-Goals

- Exceeding SOTA by benchmaxxing.
- Touching public-bench scoring logic — adapt ingest + retrieval only; run upstream scorers.
