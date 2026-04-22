---
milestone: v6
name: Typed Ingest for Public Benches
status: planned
opened: 2026-04-22
depends_on: [v5]
composite_pre: 5.5
composite_target: 7.0
axes_lifted: [raw_retrieval, token_efficiency]
---

# Milestone v6 Audit — Typed Ingest for Public Benches

## Goal

Stop pretending public benches are flat-RAG. Apply memd's typing — episodic/semantic/canonical/candidate — to public-bench input. Distill turns into semantic facts, promote repeated high-confidence facts to canonical, compile a working-context window instead of dumping top-k chunks, route re-queries through progressive-depth. LME/LoCoMo/MemBench/ConvoMem canonical numbers lift — honestly.

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post |
| --- | --- | --- | --- |
| session continuity | 20% | 5 | 6 |
| correction retention | 15% | 6 | 7 |
| procedural reuse | 15% | 4 | 5 |
| cross-harness | 15% | 6 | 6 |
| raw retrieval | 15% | 7 | 8 |
| token efficiency | 10% | 5 | 8 |
| trust + provenance | 10% | 6 | 7 |

composite: 5.5 → 7.0

## Phases

- **A6** Episodic Ingest Pipeline — bench turns ingested as episodic, not raw chunks.
- **B6** Semantic Distillation — episodic → semantic facts via LLM extractor.
- **C6** Canonical Promotion — repeated high-confidence facts promoted to canonical lane.
- **D6** Working-Context Compiler on Bench Input — prompt assembled from typed retrieval, not top-k dump.
- **E6** Progressive-Depth Routing — model can re-query memd mid-answer; bench harness supports this.
- **F6** Iterative Reasoning Harness — multi-step reasoning over typed memory for temporal-reasoning question types.

## Completion gate

Canonical intrinsic (sidecar OFF):
- LongMemEval `qa_accuracy` ≥ 0.85
- LoCoMo `token_f1_avg` ≥ 0.75
- MemBench `mc_accuracy` ≥ 0.75
- ConvoMem LLM-judge `accuracy` ≥ 0.90

No regression on retrieval diagnostics (`session_recall_any@5` stays ≥ 0.95 on LME).

Every number carries a method card per I3 rules — no gaming.

## Non-goals

- exceeding published SOTA by benchmaxxing — honest canonical run or no publish
- touching public-bench scoring logic — we run upstream scorers, adapt ingest only
