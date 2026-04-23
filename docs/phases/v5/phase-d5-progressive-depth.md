---
phase: D5
name: ProgressiveDepth Bench
version: v5
status: planned
opened: 2026-04-22
depends_on: [V4]
axis: token_efficiency, cross_harness
---

# Phase D5: ProgressiveDepth Bench

## Goal

Measure the wake/lookup/resume quality ladder: shallow queries get a cheap summary, targeted queries get 1-3 records, resume reconstructs task state. Score = quality-per-token at each depth.

## Why this phase exists

V4 E4 ships the depth contract. D5 measures adherence: on a fixed query set, how tight is wake vs raw? How precise is lookup vs top-k? How complete is resume vs full-transcript?

## Deliver

1. **Scenario generator.** 30 queries per depth class (90 total). 10 overview queries, 10 targeted queries, 10 resume queries.
2. **Metrics per depth.** `token-cost`, `answer-completeness`, `irrelevant-record-ratio`, `latency-p95`.
3. **Quality-per-token composite.** `completeness / tokens_used` for each depth; depth contract enforces cost ceilings.
4. **Runner.** `memd bench substrate --suite progressive-depth`.

## Pass Gate

- pre: no bench
- post: wake p95 ≤2000 tokens with completeness ≥0.8; lookup completeness ≥0.85 with ≤500 tokens; resume completeness ≥0.95 with ≤6000 tokens
- evidence: depth-contract adherence rate ≥ 0.95 across all queries
- regression budget: no depth's p95 token cost regresses vs V4 E4 floor

## Product Win

"the right amount of context for the job" becomes a number, not a design claim.

## Evidence

- 30-query fixture set + expected-depth routing
- per-depth p95 token cost
- completeness histogram per depth

## Fail Conditions

- Depth leakage (wake returns records lookup should): compiler priority bug.
- Cost ceiling breach: budget enforcer regressed.

## Rollback

Bench-only.
