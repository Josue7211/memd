---
phase: A3
name: Activate Retrieval
version: v3
status: pending
depends_on: []
notes: M4 dep relaxed 2026-04-16 — sidecar wiring is orthogonal to M4 dashboard/observability/hive polish. M4 (I2/M2-evo/N2) deferred for V3. Renamed from B3 to A3 on 2026-04-17 so phase IDs match execution order.
backlog_items:
  - "2026-04-14-rag-sidecar-disabled-no-fallback"
  - "2026-04-14-status-noise-runaway-checkpoint-loop"
  - "2026-04-13-status-noise-floods-memory"
  - "2026-04-14-memory-dedup-incomplete"
---

# Phase A3: Activate Retrieval

## Goal

Turn on signal classes that already exist in the codebase but are disabled or not wired. Lift LongMemEval and MemBench by activating dense retrieval, query sanitization, layered context, and priority dedup.

## Why this phase exists

Memd scores 0.860 on LongMemEval. Mempalace scores 0.966 with **pure cosine, no hybrid, no rerank** ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md#why-it-works]]). The gap is not algorithmic — it is that memd's bench default backend is `lexical` (`crates/memd-client/src/benchmark/public_benchmark.rs:1439`), `.memd/config.json` has `rag.enabled=false`, and `memd-server` does not import `memd-rag` at all. The 0.860 number is keyword-only retrieval. This phase makes embeddings reach the bench path.

## Deliver

1. **Sidecar wired into server retrieval** — `memd-server` consumes `memd-rag` for entity search and lookup paths (currently SQL-only).
2. **Bundle config defaults to enabled** — `rag.enabled=true` in fresh bundles; `MEMD_RAG_URL` resolution chain documented; bench harness defaults to `sidecar` when URL is set.
3. **Query sanitization pipeline** — port mempalace `query_sanitizer.py` (200/500-char passthrough/extract/tail/truncate) to Rust, applied before every retrieval call ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md#query-sanitization-pipeline-query_sanitizerpy]]).
4. **Layered context in wake packet** — wake assembles L0 (identity) + L1 (essential story) + L2 (on-demand) + L3 (deep search) per mempalace `layers.py` shape ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md#4-layer-context-assembly-layerspy]]).
5. **Priority dedup at retrieval** — supermemory pattern: canonical > working > search, exact-string dedup applied after fetch, before injection ([[.memd/lanes/architecture/A2-11-context-compilation-profile.md#retrieval-time-dedup-priority-based]]).
6. **Status admission cap** — kind=Status capped at 2 in wake output, or TTL hard-cut at 1h with -0.08 penalty ([[.memd/lanes/architecture/A2-13-temporal-freshness.md#ttl-calibration]]).

## Pass Gate

Bench-delta required (regenerate [[docs/verification/PUBLIC_LEADERBOARD.md]] before/after):

- pre: LongMemEval=0.860, LoCoMo=0.415, MemBench=0.346, ConvoMem=0.000
- post: **LongMemEval ≥ 0.93**, **MemBench ≥ 0.50**, LoCoMo no regression, ConvoMem no regression
- regression budget: no metric drops > 0.02
- evidence file: regenerated leaderboard committed alongside merge

Plus:
- `cargo test -p memd-server -p memd-client` green
- Wake packet inspection: ≤ 2 status items, canonical facts always present
- Sidecar reachable via `memd status` health probe

## Evidence

- Pre/post leaderboard diff
- Sample wake packet showing layered structure
- Sample retrieval trace showing sanitized query, dense candidates, dedup result
- Sidecar healthz output

## Product Win

Bench parity is necessary but not sufficient. The product-quality win A3 must also ship:

- **Wake packet reads like a curated briefing, not a status-flood.** L0/L1/L2/L3 layers make the identity + essential-story visible at a glance; on-demand items obviously on-demand.
- **Natural-language recall actually works.** Asking memd "what do I believe about X" returns canonical truth even when X never appears as a keyword in stored items. This is the dogfood test supermemory/mempalace pass today and memd fails today.
- **`memd status` reports dense-path health as a first-class surface.** If sidecar is down, user sees it; no silent lexical fallback.

Evidence (alongside bench-delta):
- Recorded before/after dogfood session on 5 natural-language queries memd fails today; annotate which surface improved
- Screenshot of wake packet before (status-flooded) vs after (layered)
- Side-by-side comparison with mempalace running the same fixture queries; note explicit wins and remaining gaps

## Fail Conditions

- LongMemEval < 0.93 after sidecar enabled — diagnose embedding pipeline before ship
- Wake packet still status-flooded — admission cap not enforced
- Server still calls SQL-only retrieval — `memd-rag` not actually wired
- Bench harness still defaults to `lexical` after rag_url configured

## Donor Anchors

- **A3-D1**: mempalace retrieval pipeline (sanitize → embed → vector → filter → rank → assemble) — [[.memd/lanes/architecture/A2-09-retrieval-pipeline.md]]
- **A3-D2**: mempalace embedding choice (all-MiniLM-L6-v2, 384-dim, cosine, L2-normalized) — [[.memd/lanes/architecture/A2-10-embedding-strategy.md]]
- **A3-D3**: supermemory priority dedup (static > dynamic > search, exact-match) — [[.memd/lanes/architecture/A2-11-context-compilation-profile.md]]
- **A3-D4**: mempalace TTL/freshness penalties for status suppression — [[.memd/lanes/architecture/A2-13-temporal-freshness.md]]

## Rollback

- Sidecar disable flag in `.memd/config.json` (`rag.enabled=false`) reverts to lexical-only
- Layered wake packet behind `wake.layered=false` flag during rollout
- Priority dedup behind `retrieval.priority_dedup=false` flag

## Out of scope

- Reranker (lands in B3)
- Embedding model swap to BGE-large (lands in B3)
- Atlas multi-hop expansion (lands in C3)
- Episode consolidation (lands in D3)
- ConvoMem adapter fix (lands in E3)
