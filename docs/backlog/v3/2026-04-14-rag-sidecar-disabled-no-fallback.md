---
status: resolved
severity: medium
phase: B3
opened: 2026-04-14
resolved: 2026-04-21
scope: memd-core
---
# No Semantic Search Baseline (RAG Disabled, No Fallback) — RESOLVED 2026-04-21

- status: `resolved`
- severity: `medium`
- phase: `V2-N2`
- opened: `2026-04-14`
- resolved: `2026-04-21`
- scope: memd-core

## Problem

RAG backend (semantic search for memory) is disabled by default, so memd has no
semantic-search baseline at the product level. When enabled, there is still no timeout,
retry, or fallback cache. If the backend is slow or unavailable, retrieval can block
indefinitely.

## Fix

- Add timeout to RAG calls ✓ (commit `eec95de`, `MEMD_RAG_TIMEOUT_MS` default 300ms)
- Implement exponential backoff on failure — scoped to ingest only (commit `03e65ef`, 100ms → 500ms). Removed from read path per advisor review: backoff on retrieve/rerank amplifies tail latency without buying correctness.
- Add fallback to local index if backend unavailable ✓ — `fetch_dense_candidates` returns `Ok(vec![])` on timeout/error; search blend degrades to lexical (the existing local index).
- Record a semantic-search baseline and failure budget in verification ✓ — `RagHealthStatus.recent_failures` counter surfaced via `/healthz` (schema bump in `memd-schema`).
- Document degraded mode ✓ (this resolution section).

## Resolution 2026-04-21

### Degraded-mode behavior

| Call path | Behavior when sidecar slow/down |
| --- | --- |
| `fetch_dense_candidates` (user search) | Single attempt, timeout `MEMD_RAG_TIMEOUT_MS` (default 300ms), on fail → `Ok(vec![])` → blend degrades to pure lexical. No retry (would amplify p95 tail). |
| `rerank_candidates` (user search) | Same shape. On fail → `Ok(vec![])` → caller keeps original candidate order (no rerank). |
| `ingest_item` (background write) | 3 attempts (initial + 2 backoffs at 100ms, 500ms), per-attempt timeout = 2× `MEMD_RAG_TIMEOUT_MS`. On final fail → warn + counter bump, still non-fatal. |
| `healthz` | Single attempt, 500ms timeout (pre-existing). `recent_failures` exposes cumulative counter. |

### Env knobs

- `MEMD_RAG_URL` — sidecar URL. Absent/empty = bridge disabled entirely.
- `MEMD_RETRIEVAL_RAG_DENSE` — default ON, set `0`/`false` to skip dense fetch regardless of URL.
- `MEMD_RAG_TIMEOUT_MS` — per-attempt timeout in ms. Default 300ms. Floor 1.

### Failure budget

`RagHealthStatus.recent_failures` is a process-local `AtomicU64` that counts:
- retrieve timeout or error
- rerank timeout or error
- ingest final failure (after all retries exhausted)

It does NOT count healthz failures (those are already visible via `reachable=false`). The counter is cumulative for the server lifetime; resets on restart. Consumer contract: a positive value means degraded reads or dropped ingests since boot. Zero is skipped from JSON (backwards-compatible).

### Why no backoff on reads

Initial plan was exponential backoff everywhere. Advisor review 2026-04-21: backoff on user-facing reads multiplies worst-case tail latency by retry count without buying correctness — the fallback to lexical already handles the missing-data case at zero additional cost. Retry shifted to ingest only, where latency doesn't hit the hot path.
