# No Semantic Search Baseline (RAG Disabled, No Fallback)

- status: `open`
- severity: `medium`
- phase: `V2-N2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

RAG backend (semantic search for memory) is disabled by default, so memd has no
semantic-search baseline at the product level. When enabled, there is still no timeout,
retry, or fallback cache. If the backend is slow or unavailable, retrieval can block
indefinitely.

## Fix

- Add timeout to RAG calls
- Implement exponential backoff on failure
- Add fallback to local index if backend unavailable
- Record a semantic-search baseline and failure budget in verification
- Document degraded mode
