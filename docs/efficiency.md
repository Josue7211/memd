# Efficiency

## Goal

Perfect memory is useless if retrieval burns tokens and latency.

`memd` should optimize for:

- minimal token overhead
- minimal repeated rereads
- low-latency hot-path retrieval
- semantic depth only when needed

## Core Rules

1. Atomic records first
- Store facts, decisions, statuses, and runbooks as small typed records.

2. Compact context packets
- Retrieval should return small packets, not blobs.

3. Summaries outrank raw docs
- Compact canonical summaries should be the first retrieval target.

4. Relevance beats raw recency
- Retrieval should rank by scope, stage, confidence, freshness, and query match.

5. Raw docs are evidence
- Use LightRAG and raw documents as fallback support, not the default first payload.

6. Hard caps everywhere
- Every retrieval path must have bounded result counts and bounded formatting.

7. Scope-first retrieval
- Local and synced context must be checked before long-term global memory.
- Clients should also provide a retrieval route and intent so the manager can skip irrelevant tiers instead of scanning everything.

8. Promotion is compression
- Auto-dream and writeback should reduce memory volume, not increase it.

9. Dead memory should leave the hot path
- Expired and stale memories must stop competing with active context.

10. Redundancy should collapse early
- Near-duplicate facts should merge under a redundancy key before they reach long-term storage.

## Compact Record Format

The platform should support a highly compact memory representation for transport and storage optimization.

Candidate direction:

- QMD-style compact records
- structured short fields
- lossless enough for reconstruction
- optimized for both machine parsing and token efficiency

This should be an optional serialization layer over the canonical schema, not a replacement for schema semantics.

## Hot Path Strategy

- `memd` structured store handles the first retrieval pass
- semantic retrieval is only invoked when compact structured memory is insufficient
- graph retrieval is a later-stage fallback, not a default dependency

## Current Implementation Direction

- SQLite-backed hot store for durability
- compacted `search` responses
- compacted `context` responses
- typed retrieval routing by route and intent
- compact record transport via `/memory/context/compact`
- verification endpoint for refreshing stale memories
- bounded default limits on result count and item size
- TTL-based expiry before retrieval
- stale marking for aging canonical memories
