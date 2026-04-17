---
phase: B3
name: Reranker + Embeddings
version: v3
status: pending
depends_on: [A3]
notes: Renamed from F3 to B3 on 2026-04-17 so phase IDs match execution order.
backlog_items:
  - "2026-04-14-no-behavior-changing-recall-proof"
  - "2026-04-14-rag-sidecar-disabled-no-fallback"
---

# Phase B3: Reranker + Embeddings

## Goal

Squeeze the last 3-5 points out of LongMemEval and bump LoCoMo by adding a reranker on top of A3's dense retrieval and trying a larger embedding model. Mempalace shows pure cosine = 96.6%, **+ Haiku/Sonnet rerank = 100%** ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md#no-reranking-in-core-pipeline]]).

## Why this phase exists

A3 activates the dense signal. B3 adds the second pass that mempalace uses to reach the ceiling. Reranker is optional in mempalace — flip on with a flag, score the top-K candidates, return the reordered list. Embedding model swap is a one-line config change (`AllMiniLM-L6-v2` → `BAAI/bge-large-en-v1.5`) with empirical +3.5pp on mempalace's bench ([[.memd/lanes/architecture/A2-10-embedding-strategy.md#benchmarked-alternatives-from-longmemeval_benchpy]]).

## Deliver

1. **LLM reranker on top-K** — after A3 dense retrieval returns top-N (default 20), pass to reranker, keep top-K (default 5). Reranker model configurable: `claude-haiku-4-5`, `claude-sonnet-4-6`, or local fallback. Behind `retrieval.rerank=true` flag.
2. **Reranker as a sidecar route** — `memd-sidecar` exposes `/rerank` endpoint, takes `{query, candidates[]}`, returns scored candidates. Server calls it after dense retrieval, before priority dedup.
3. **Embedding model swap path** — `MEMD_EMBED_MODEL` env var + bundle config; supported values: `all-minilm-l6-v2` (default), `bge-base-en-v1.5`, `bge-large-en-v1.5`. Migration: re-embed corpus on swap (track via embedding_model column on stored items).
4. **Query prefix convention** — fastembed wants `"query: " + query` for query-side embedding ([[.memd/lanes/architecture/A2-10-embedding-strategy.md#memd-sidecar-embedding]]). Apply automatically in retrieval path.
5. **Per-query embedding cache** — already partial in sidecar; extend to per-record-ID cache, evict on update.

## Pass Gate

Bench-delta required (regenerate [[docs/verification/PUBLIC_LEADERBOARD.md]]):

- pre: LongMemEval=0.93, LoCoMo=0.55 (post-A3 baseline; if A3 not green, do not start B3)
- post: **LongMemEval ≥ 0.97**, **LoCoMo ≥ 0.55** (additional bump comes in C3)
- regression budget: no metric drops > 0.02
- evidence: leaderboard regenerated with rerank=on AND rerank=off, both committed
- latency: rerank path P95 ≤ 1500ms (Haiku)

Plus:
- `cargo test -p memd-sidecar` green for `/rerank` route
- Re-embed migration runs on `bge-large-en-v1.5` swap, items get embedding_model stamp

## Evidence

- Pre/post leaderboard with rerank on/off split
- Latency histograms for rerank path
- Sample (query, top-N before rerank, top-K after rerank) trace
- Embedding-swap migration log

## Product Win

- **Top-K feels intentional, not random.** A human inspecting the rerank trace can explain why each result surfaced — the reranker's scoring story holds up to scrutiny.
- **Latency stays conversational.** P95 ≤ 1500ms with Haiku means agents still feel fast; longer-latency paths must be async or hidden behind an explicit "deep search" flag.
- **Embedding swap is boring.** Switching MiniLM ↔ BGE-large is a config change + background re-embed, not a migration incident. Track `embedding_model` on stored items so mixed corpora stay valid during rollout.

Evidence:
- 10 real dogfood queries: record top-N pre-rerank, top-K post-rerank, human judgment on why the order changed
- One-week dogfood usage log with zero agent complaints about wrong-order retrieval
- Migration runbook (single page) proving the re-embed swap is non-disruptive

## Fail Conditions

- LongMemEval < 0.97 with reranker on — diagnose top-N candidate quality before tuning rerank
- BGE-large embedding swap regresses any metric — keep MiniLM as default
- Reranker latency P95 > 3s — make path async or fall back to dense-only

## Donor Anchors

- **B3-D1**: mempalace optional rerank pipeline (96.6 → 100) — [[.memd/lanes/architecture/A2-09-retrieval-pipeline.md#no-reranking-in-core-pipeline]]
- **B3-D2**: mempalace embedding bench (BGE-large +3.5pp at 1024-dim) — [[.memd/lanes/architecture/A2-10-embedding-strategy.md#benchmarked-alternatives-from-longmemeval_benchpy]]
- **B3-D3**: fastembed query-prefix convention — [[.memd/lanes/architecture/A2-10-embedding-strategy.md#memd-sidecar-embedding]]

## Rollback

- `retrieval.rerank=false` reverts to A3 dense-only behavior
- `MEMD_EMBED_MODEL=all-minilm-l6-v2` reverts to MiniLM if BGE regresses
- Reranker route can be killed at sidecar level without server restart

## Out of scope

- Atlas multi-hop (C3)
- Episode consolidation (D3)
- ConvoMem adapter (E3)
- Cross-encoder local reranker (future, only if Haiku latency unacceptable)
