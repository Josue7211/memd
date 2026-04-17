---
phase: E3
name: Atlas at Recall
version: v3
status: pending
depends_on: [B3, F3]
backlog_items:
  - "2026-04-13-atlas-dormant"
  - "2026-04-14-atlas-fully-built-completely-dormant"
  - "2026-04-13-architecture-knowledge-not-in-lanes"
---

# Phase E3: Atlas at Recall

## Goal

Activate the dormant atlas: populate entity edges at ingest, traverse them at retrieval to bring multi-hop neighbors into the candidate pool. Targets LoCoMo specifically, where multi-session conversational memory needs subject→predicate→object expansion.

## Why this phase exists

`docs/backlog/2026-04-14-atlas-fully-built-completely-dormant.md` confirms the schema, tables, and API exist but nothing writes to them and nothing reads from them at recall time. Mempalace pattern is **entity extraction is a pre-graph pass at ingest, not retroactive** — co-occurrence + `[[entity]]` links + NER produce edges, every edge is source-backed with valid_from/valid_to ([[.memd/lanes/architecture/A2-02-atlas-entity-graph.md#entity-extraction-pre-graph]]).

## Deliver

1. **Ingest-time entity extraction** — every store/correct call runs an extraction pass: co-occurrence inside the same item, `[[wiki-style]]` links as strong edges, NER for named entities. Edges written with `(subject, predicate, object, valid_from, valid_to, source_item_id, confidence)`.
2. **Multi-hop traversal at recall** — after dense retrieval (B3) returns candidates, expand each by 1-hop atlas neighbors filtered by valid_from/valid_to. Optionally 2-hop behind a budget cap. Expanded set goes into reranker (F3).
3. **Temporal validity windows** — corrections don't delete edges; old edge gets `valid_to=now()`, new edge inserted with `valid_from=now()`. Time-scoped queries become possible: "what did we believe at T?"
4. **Atlas health surfaces** — `memd status` shows edge count, region count, dormant/active ratio. If edges_total < expected per stored items, surface as "atlas dormant" warning.
5. **Region-based retrieval routing** — atlas regions (clusters of densely-linked entities) become first-class filters: `memd lookup --region foo` narrows search before retrieval.

## Pass Gate

Bench-delta required (regenerate [[docs/verification/PUBLIC_LEADERBOARD.md]]):

- pre: LongMemEval=0.97, LoCoMo=0.55, MemBench=0.50 (post-F3 baseline)
- post: **LoCoMo ≥ 0.65**, LongMemEval no regression, MemBench no regression
- regression budget: no metric drops > 0.02
- evidence: leaderboard regenerated; LoCoMo subset broken down by hop-count to show multi-hop wins

Plus:
- `cargo test -p memd-server` green for atlas extraction + traversal
- Atlas edge count > 0 after fresh ingest; ratio (edges / items) ≥ 0.5
- Multi-hop traversal P95 ≤ 50ms additional latency on top of dense retrieval

## Evidence

- Pre/post LoCoMo leaderboard, broken down by question type (single-hop vs multi-hop)
- Sample atlas state after ingest (entity count, edge count, region count)
- Sample retrieval trace showing dense-candidates → 1-hop expansion → reranker input
- `memd status` output showing live edges_total

## Fail Conditions

- LoCoMo < 0.65 — diagnose extraction quality (are entities actually being detected?) before traversal tuning
- Atlas edges_total stays at 0 after ingest — extraction pass isn't running
- Multi-hop expansion adds candidates but rerank still picks wrong — F3 reranker may need atlas-aware prompts
- 2-hop traversal P95 > 200ms — keep at 1-hop only

## Donor Anchors

- **E3-D1**: mempalace knowledge graph schema with valid_from/valid_to triples — [[.memd/lanes/architecture/A2-02-atlas-entity-graph.md#mempalace-knowledge-graph-schema-sqlite-triples]]
- **E3-D2**: mempalace pre-graph entity extraction (co-occurrence + wiki links + NER) — [[.memd/lanes/architecture/A2-02-atlas-entity-graph.md#entity-extraction-pre-graph]]
- **E3-D3**: temporal correction pattern (insert new + close old, no delete) — [[.memd/lanes/architecture/A2-02-atlas-entity-graph.md#temporal-filtering]]

## Rollback

- `atlas.extract_at_ingest=false` disables the extraction pass (writes still allowed)
- `retrieval.atlas_expansion=false` reverts to dense-only candidates
- Region routing flag-gated; defaults off if regression detected

## Out of scope

- Episode consolidation (C3)
- Decay calibration (C3)
- ConvoMem adapter (A3)
- Visual graph rendering in dashboard (already in I2 scope)
