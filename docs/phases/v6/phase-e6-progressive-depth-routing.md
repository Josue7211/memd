---
phase: E6
name: Progressive-Depth Routing on Bench
version: v6
status: planned
opened: 2026-04-22
depends_on: [D6]
axis: token_efficiency, raw_retrieval
plan_spec: docs/phases/v6/phase-e6-plan.md
---

# Phase E6: Progressive-Depth Routing on Bench

## Goal

Give the bench-answer model the ability to re-query memd mid-answer. First pass: wake-depth recall (budget 2000 tokens). If the model asks, go deeper: targeted `lookup --depth=targeted` or full `resume --depth=resume`. The model stops when it's confident. Bench harness supports this multi-call shape.

## Why this phase exists

D6 compiles once up front — a single shot. Harder bench items (multi-hop LoCoMo, temporal LME) benefit from a second or third lookup once the question is clearer. V4 E4 shipped the depth flag; E6 wires the bench harness to honor it.

## Deliver

1. **Harness multi-call shape.** Bench runner accepts a tool-call loop: model emits `memd_lookup(query, depth)` pseudo-call, runner resolves, injects result, continues. Max 3 depth calls per question.
2. **Depth routing policy.** `docs/contracts/bench-depth-routing.md` — when to go from wake → targeted → resume. Default: wake first; escalate on first empty-result or low-confidence answer.
3. **Telemetry.** Per-question: depth calls made, tokens per depth, final answer tokens. Emit to `.memd/benchmarks/public/results/depth-telemetry-<date>.ndjson`.
4. **Hard cap.** Abort after 3 depth calls or 10k total retrieval tokens — log as `depth_budget_exceeded`.
5. **Baseline lift test.** LoCoMo multi-hop subset must lift ≥ 0.04 vs D6 baseline. LME temporal-reasoning subset ≥ 0.03.

## Pass Gate

- pre: one-shot retrieval
- post: multi-call depth routing live on bench harness; cumulative V6 lifts: LME ≥ +0.07, LoCoMo ≥ +0.07, MemBench ≥ +0.06, ConvoMem ≥ +0.03
- evidence: depth NDJSON, routing samples, delta report
- regression budget: no bench may drop below D6 baseline

## Product Win

memd isn't a one-shot retriever anymore. It's a queryable substrate during the answer.

## Evidence

- routing card
- depth NDJSON
- delta report

## Fail Conditions

- Lifts missed: routing policy under-triggers or over-triggers — tune policy, not bench.
- Depth calls > 3 on > 5% of questions: policy is thrashing, root-cause.

## Non-Goals

- Multi-step reasoning chains (F6).
- Cross-bench routing (within-bench only for V6).
