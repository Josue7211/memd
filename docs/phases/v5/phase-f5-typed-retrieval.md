---
phase: F5
name: TypedRetrieval Bench
version: v5
status: planned
opened: 2026-04-22
depends_on: [V4]
axis: raw_retrieval, typed_retrieval
---

# Phase F5: TypedRetrieval Bench

## Goal

Measure whether query shape routes to the right MemoryKind. "What did I decide about X" → Decision records. "How do I do Y" → Runbook. "Is Z true" → Fact. Wrong-type results penalized in scoring.

## Why this phase exists

memd's typed memory is the substrate differentiator. Without a bench that rewards correct typing, the system can silently regress to flat-RAG.

## Deliver

1. **Scenario generator.** 50 queries per kind × 11 kinds (Correction included post-C4) = 550 queries.
2. **Type router (introspection).** Expose `memd lookup --explain-route` showing which kinds were queried.
3. **Metrics.** `correct-type-rate@1` (top result is expected kind), `wrong-type-ratio`, `mixed-result-kind-distribution`.
4. **Runner.** `memd bench substrate --suite typed-retrieval`.
5. **Taxonomy card.** `docs/contracts/type-taxonomy.md` — which query shape → which kind.

## Pass Gate

- pre: no bench, types measured implicitly
- post: correct-type-rate@1 ≥ 0.85; wrong-type-ratio ≤ 0.05
- evidence: 550-query result NDJSON + report + taxonomy card reviewed
- regression budget: no single kind drops below 0.75 rate

## Product Win

"memd returns the right flavor of memory" becomes measurable.

## Evidence

- taxonomy card
- bench results
- per-kind confusion matrix

## Fail Conditions

- One kind starving: query router under-weighting that kind; tune, rerun.
- Correction kind misclassified as Fact: C4 provenance not surfaced at route stage.

## Rollback

Bench-only.
