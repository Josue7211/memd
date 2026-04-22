---
phase: A6
name: Episodic Ingest Pipeline
version: v6
status: planned
opened: 2026-04-22
depends_on: [V5]
axis: raw_retrieval, token_efficiency
plan_spec: docs/phases/v6/phase-a6-plan.md
---

# Phase A6: Episodic Ingest Pipeline

## Goal

Stop ingesting public-bench turns as flat RAG chunks. Each turn becomes a typed episodic record with full provenance (bench id, session id, turn index, speaker, timestamp). Downstream suites (B6 distillation, D6 compiler) operate on episodic records, not raw strings.

## Why this phase exists

Upstream benches (LongMemEval, LoCoMo, MemBench, ConvoMem) ship conversations as JSON arrays of turns. Current memd-client's bench runner concatenates turns into a context blob and runs BM25+embed. That fakes the ingest surface: memd's typing never gets a chance. A6 makes bench ingest go through the same path as real harness ingest — via `memd remember --kind episodic` equivalents — so every subsequent phase operates on memd's actual structure.

## Deliver

1. **Adapter module.** `crates/memd-client/src/benchmark/typed_ingest/episodic.rs` — per-bench fixture loaders that emit `MemoryRecord{kind: Episodic}` rather than raw chunks.
2. **Bench harness patch.** Existing public-bench runners (`public_benchmark.rs`) gain `--typed-ingest=episodic` flag that routes turns through the new adapter.
3. **Fixture-bench mapping card.** `docs/contracts/public-bench-ingest.md` — per-bench: what counts as a turn, what provenance fields map, which speaker labels are agent vs user.
4. **Ingest integrity test.** Round-trip: load LME sample → episodic store → query → confirm every turn round-trippable with provenance intact.
5. **Baseline lock.** Re-run canonical public benches with `--typed-ingest=episodic` (NO distillation, NO compiler) and record floor. Must not regress vs flat-RAG baseline by >1% — episodic alone is a lateral move, not a gain.

## Pass Gate

- pre: turns ingested as flat chunks; memd kind metadata absent for bench inputs
- post: `--typed-ingest=episodic` available on all four public bench runners; ingest-integrity test green; canonical scores within ±1% of prior baseline
- evidence: `.memd/benchmarks/public/results/typed-episodic-YYYY-MM-DD/`, ingest card committed
- regression budget: lateral (not a gain phase)

## Product Win

Every subsequent bench run speaks memd's ingest shape. B6's distillation has typed input to work from.

## Evidence

- ingest card
- per-bench round-trip test green
- baseline delta report ≤ 1%

## Fail Conditions

- Canonical regression >1% with episodic-only ingest: root-cause in ingest path, not in scorer.
- Provenance fields missing for any turn: hard fail.

## Non-Goals

- Distillation (B6).
- Promotion (C6).
- Scoring or retrieval changes.
