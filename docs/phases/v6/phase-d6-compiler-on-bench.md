---
phase: D6
name: Working-Context Compiler on Bench Input
version: v6
status: planned
opened: 2026-04-22
depends_on: [C6]
axis: token_efficiency, raw_retrieval
plan_spec: docs/phases/v6/phase-d6-plan.md
---

# Phase D6: Working-Context Compiler on Bench Input

## Goal

Stop concatenating top-k chunks into the bench prompt. Apply V4 D4's working-context compiler: canonical lane pinned, recent episodic last, semantic facts budget-packed, preferences surface. Answer-time prompt is a typed window, not a chunk salad.

## Why this phase exists

V4 shipped the compiler for harness wake. V6 moves it to bench answer-time. This is the token-efficiency lever: same answer, fewer tokens, or better answer, same tokens. Public benches with context-length limits (LoCoMo) benefit most.

## Deliver

1. **Compiler bench shim.** `crates/memd-client/src/benchmark/typed_ingest/compiler.rs` — wraps `runtime::resume::compiler::*` from V4 D4 for bench-answer path.
2. **Budget profiles.** Per-bench budgets: LME 2000 tokens (short answers), LoCoMo 3500 (multi-turn), MemBench 2500, ConvoMem 3000. Profiles committed in `.memd/benchmarks/public/compiler-budgets.yaml`.
3. **Section priority.** `canonical > preferences > recent_episodic > semantic > raw_episodic`; overflow drops from the bottom.
4. **Token counter.** Reuse V4's `compute_wake_token_metrics` — no divergent tokenizer.
5. **A/B harness.** `--compiler=on|off` flag on bench runner for clean comparisons.
6. **Baseline lift test.** Must lift MemBench `mc_accuracy` ≥ 0.03, LoCoMo `token_f1_avg` ≥ 0.03, and cut mean prompt tokens ≥ 25% on LME.

## Pass Gate

- pre: top-k chunk dump as prompt
- post: compiler runs, budgets honored, A/B harness working; cumulative V6 lifts: LME ≥ +0.04 (held from C6), LoCoMo ≥ +0.03 (new), MemBench ≥ +0.06 (cumulative)
- evidence: prompt diffs per bench sample, token-usage histograms, delta report
- regression budget: no canonical bench may regress below C6 baseline

## Product Win

"memd saves 25%+ tokens per answer" is now a number.

## Evidence

- compiler budgets committed
- prompt-diff samples
- token histograms
- delta report

## Fail Conditions

- Mean prompt-tokens drop <25% on LME: budget profiles too loose.
- Any canonical regression: compiler dropping high-value records — priority order wrong.

## Non-Goals

- Mid-answer re-query (E6).
- Iterative reasoning loops (F6).
