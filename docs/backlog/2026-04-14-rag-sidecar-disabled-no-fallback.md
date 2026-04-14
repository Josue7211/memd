# RAG Sidecar Disabled, No Fallback

- status: `open`
- severity: `medium`
- phase: `V2-K2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

RAG backend (semantic search for memory) is disabled by default. No timeout, retry, or fallback cache when enabled. If backend is slow or unavailable, memory retrieval blocks indefinitely.

## Fix

- Add timeout to RAG calls
- Implement exponential backoff on failure
- Add fallback to local index if backend unavailable
- Document degraded mode
