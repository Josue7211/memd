---
phase: A2
name: Inspiration Extraction
version: v2
status: verified
depends_on: []
backlog_items: [55, 56]
---

# Phase A2: Inspiration Extraction

Current status: `verified`

## Goal

Deep-read mempalace + supermemory + Omegon + Smriti, extract patterns that close memd gaps faster.

## Deliver

- Updated architecture lane source material (`.memd/lanes/architecture/`)
- Per-gap extraction notes mapping external pattern → memd implementation
- Updated inspiration lane with deeper extraction notes
- Benchmark harness approach from mempalace adapted for memd
- 13 extraction targets with concrete implementation details (thresholds, algorithms, schemas)

## Pass Gate

- Each extraction target has a written note: what the pattern is, how to adapt it, which backlog item it closes
- At least 8 of the priority extraction targets documented (13 completed)
- No code changes in this phase — research only

## Evidence

- `.memd/lanes/architecture/` has 13 extraction targets (expanded from original 8)
- Each note contains: concrete implementation pseudocode, threshold values, data structures, memd adaptation
- Backlog items annotated with "extraction source: mempalace/supermemory"
- Priority extraction targets covered:
  1. A2-01: Benchmark harness (DCG/NDCG, ephemeral stores, 4-layer context stack)
  2. A2-02: Atlas entity graph (SQLite triples, temporal validity, D3 force config)
  3. A2-03: Ingestion pipeline (hash manifest, typed extraction, conversation mining)
  4. A2-04: Dedup (0.15 cosine threshold, two-pass, priority dedup)
  5. A2-05: Lane auto-activation (94-entry folder map, 4-priority routing, zero LLM)
  6. A2-06: Correction repair (scan→prune→rebuild, WAL audit, temporal invalidation)
  7. A2-07: Hooks capture (context/capture/spill modes, write storm diagnosis)
  8. A2-08: Plugin packaging + graph UI (5 adapters, LRU turn cache, data-driven graph)
  9. A2-09: Retrieval pipeline (query sanitization, pure vector search, layered context)
  10. A2-10: Embedding strategy (all-MiniLM-L6-v2, 384-dim, bge-large alternative)
  11. A2-11: Context compilation / profile (static/dynamic split, priority dedup)
  12. A2-12: Version chains (parentMemoryId/rootMemoryId vs supersedes Vec)
  13. A2-13: Temporal freshness (5-signal scoring, validity windows, rehearsal tracking)
- New backlog item: `research-not-stored-as-shared-memory` (dog-food failure)
- Comprehensive donor-to-phase mapping: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md`
- Every V2 phase doc (B2–N2) updated with "Donor Extraction" section listing which patterns land there
- Omegon and Smriti repos cloned locally for direct Rust code lifts
- 12 direct Rust lift targets identified from Omegon (same language, directly portable)

## Fail Conditions

- Extraction notes are vague ("use their approach") instead of specific
- No clear mapping to backlog items

## Rollback

- N/A (research only, no code changes)
