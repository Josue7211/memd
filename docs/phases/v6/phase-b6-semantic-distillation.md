---
phase: B6
name: Semantic Distillation
version: v6
status: planned
opened: 2026-04-22
depends_on: [A6]
axis: raw_retrieval, token_efficiency
plan_spec: docs/phases/v6/phase-b6-plan.md
---

# Phase B6: Semantic Distillation

## Goal

Turn episodic turn records (A6 output) into semantic fact records via an LLM extractor. Every durable assertion in a conversation ("I live in Seattle", "my flight is Thursday", "the deadline is May 1") becomes a `Fact`/`Decision`/`Preference` with provenance back to source turns. Public benches gain a typed semantic layer — the first real retrieval substrate improvement.

## Why this phase exists

Episodic alone is lateral. What moves canonical numbers on LME/LoCoMo/ConvoMem is separating the durable assertions from chatter so the retriever at answer-time queries facts, not utterance fragments. B6 is the first V6 phase that can move a public-bench number.

## Deliver

1. **Extractor client.** `crates/memd-client/src/benchmark/typed_ingest/distiller.rs` — LLM-judge-style call to codex-lb (gpt-5.4 by default; gpt-5.4 for high-confidence pass). Input: episodic record + prior conversation window. Output: zero or more `SemanticCandidate { kind, content, provenance }`.
2. **Prompt card.** `docs/contracts/semantic-distillation.md` — extraction prompt, expected output schema, kind-taxonomy cheatsheet.
3. **Candidate store.** Distilled semantic records land as `stage: candidate` — not yet canonical. C6 promotes.
4. **Dedupe.** Near-duplicate candidates collapse via `content_hash` + cosine similarity > 0.92.
5. **Cache.** Extraction results cached by `(turn_id, prompt_version)`; re-runs are free.
6. **Baseline lift test.** Canonical bench runs with `--typed-ingest=episodic+semantic` (no compiler, no promotion) vs episodic-only. Must lift LME `qa_accuracy` by ≥ 0.02 to prove extraction is pulling weight.

## Pass Gate

- pre: no semantic layer on bench input
- post: extractor runs, candidate store populated, dedupe tested; canonical LME `qa_accuracy` lifts ≥ 0.02 vs A6 baseline
- evidence: extraction NDJSON per bench run, cached results, delta report
- regression budget: LoCoMo and MemBench ≤ 1% regression tolerated (semantic extraction can drop recall on narrative-heavy corpora; C6 recovers)

## Product Win

memd's extraction pipeline becomes a measured contributor on public canonical numbers.

## Evidence

- prompt card
- cached extraction runs
- delta report ≥ +0.02 LME
- judge-cost report (budget in milli-USD per 1k turns)

## Fail Conditions

- <0.02 lift: extractor prompt is under-specified or extraction overtriggers — do not tune bench to mask.
- >10% cost blow-up vs expected budget: cache miss-rate investigation before scaling.

## Non-Goals

- Canonical promotion (C6).
- Retrieval path changes (D6).
