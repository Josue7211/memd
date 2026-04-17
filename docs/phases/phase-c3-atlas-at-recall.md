---
phase: C3
name: Atlas at Recall
version: v3
status: pending
depends_on: [A3, B3]
notes: Renamed from E3 to C3 on 2026-04-17 so phase IDs match execution order.
backlog_items:
  - "2026-04-13-atlas-dormant"
  - "2026-04-14-atlas-fully-built-completely-dormant"
  - "2026-04-13-architecture-knowledge-not-in-lanes"
---

# Phase C3: Atlas at Recall

## Goal

Activate the dormant atlas: populate entity edges at ingest, traverse them at retrieval to bring multi-hop neighbors into the candidate pool. Targets LoCoMo specifically, where multi-session conversational memory needs subject→predicate→object expansion.

## Why this phase exists

`docs/backlog/2026-04-14-atlas-fully-built-completely-dormant.md` confirms the schema, tables, and API exist but nothing writes to them and nothing reads from them at recall time. Mempalace pattern is **entity extraction is a pre-graph pass at ingest, not retroactive** — co-occurrence + `[[entity]]` links + NER produce edges, every edge is source-backed with valid_from/valid_to ([[.memd/lanes/architecture/A2-02-atlas-entity-graph.md#entity-extraction-pre-graph]]).

## Deliver

1. **Ingest-time entity extraction** — every store/correct call runs an extraction pass: co-occurrence inside the same item, `[[wiki-style]]` links as strong edges, NER for named entities. Edges written with `(subject, predicate, object, valid_from, valid_to, source_item_id, confidence)`.
2. **Multi-hop traversal at recall** — after dense retrieval (A3) returns candidates, expand each by 1-hop atlas neighbors filtered by valid_from/valid_to. Optionally 2-hop behind a budget cap. Expanded set goes into reranker (B3).
3. **Temporal validity windows** — corrections don't delete edges; old edge gets `valid_to=now()`, new edge inserted with `valid_from=now()`. Time-scoped queries become possible: "what did we believe at T?"
4. **Atlas health surfaces** — `memd status` shows edge count, region count, dormant/active ratio. If edges_total < expected per stored items, surface as "atlas dormant" warning.
5. **Region-based retrieval routing** — atlas regions (clusters of densely-linked entities) become first-class filters: `memd lookup --region foo` narrows search before retrieval.

## Pass Gate

Bench-delta required (regenerate [[docs/verification/PUBLIC_LEADERBOARD.md]]):

- pre: LongMemEval=0.97, LoCoMo=0.55, MemBench=0.50 (post-B3 baseline)
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

## Product Win

- **Atlas is navigable by a human, not just traversed by code.** User can walk from one entity to related entities via `memd lookup` (region filter, 1-hop expansion visible in output), without reading source.
- **Answers show their hops.** When an atlas edge contributed to a retrieval, the response explains the subject→predicate→object path. No black-box recall.
- **Atlas "dormant" is a loud state.** If edges/items ratio drops below threshold, `memd status` says so in plain English; user doesn't need a Grafana dashboard to notice.

Evidence:
- Dogfood walkthrough: user navigates atlas from a known entity to an unexpected-but-valid neighbor; record the session
- Sample answer that cites an atlas hop chain (subject → predicate → object) as retrieval provenance
- `memd status` output from both a healthy and a deliberately-underpopulated corpus, showing the dormant-warning path

## Fail Conditions

- LoCoMo < 0.65 — diagnose extraction quality (are entities actually being detected?) before traversal tuning
- Atlas edges_total stays at 0 after ingest — extraction pass isn't running
- Multi-hop expansion adds candidates but rerank still picks wrong — B3 reranker may need atlas-aware prompts
- 2-hop traversal P95 > 200ms — keep at 1-hop only

## Donor Anchors

- **C3-D1**: mempalace knowledge graph schema with valid_from/valid_to triples — [[.memd/lanes/architecture/A2-02-atlas-entity-graph.md#mempalace-knowledge-graph-schema-sqlite-triples]]
- **C3-D2**: mempalace pre-graph entity extraction (co-occurrence + wiki links + NER) — [[.memd/lanes/architecture/A2-02-atlas-entity-graph.md#entity-extraction-pre-graph]]
- **C3-D3**: temporal correction pattern (insert new + close old, no delete) — [[.memd/lanes/architecture/A2-02-atlas-entity-graph.md#temporal-filtering]]

## Rollback

- `atlas.extract_at_ingest=false` disables the extraction pass (writes still allowed)
- `retrieval.atlas_expansion=false` reverts to dense-only candidates
- Region routing flag-gated; defaults off if regression detected

## Out of scope

- Episode consolidation (D3)
- Decay calibration (D3)
- ConvoMem adapter (E3)
- Visual graph rendering in dashboard (already in I2 scope)
