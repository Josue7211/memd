# Checkpoint / Resume Asymmetry

- status: `open`
- found: `2026-04-13`
- scope: memd-client

## Summary

Checkpoint saves individual memory items with full metadata (confidence, TTL,
tags). Resume loads an aggregated snapshot. The rehydration_queue is fetched
during resume but never persisted by checkpoint. Recovery artifacts are transient.

## Symptom

- Checkpoint saves detailed per-item state → resume restores aggregate counts only
- Rehydration items exist during a session but vanish if server restarts
- Confidence and TTL metadata saved at checkpoint are not restored into resume state

## Root Cause

- Checkpoint writes items via `remember` API (individual memory items)
- Resume reads via `context_compact` / `working` / `inbox` (aggregated snapshot)
- Different data shape in and out — no round-trip fidelity
- Rehydration queue is session-transient, not persisted

## Fix Shape

- Document this as intentional (checkpoint = durable write, resume = fresh read)
- Or add rehydration persistence so recovery artifacts survive server restart
- Depends on whether full round-trip fidelity is a product requirement

## Evidence

- `crates/memd-client/src/runtime/checkpoint.rs:171-191` — checkpoint writes
- `crates/memd-client/src/runtime/resume/mod.rs:238-260` — resume reads

## Dependencies

- independent: design decision more than bug
- related to [[docs/backlog/2026-04-13-status-noise-floods-memory.md|status-noise-floods-memory]] (checkpoint noise is the bigger problem)

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/locks/2026-04-11-memd-theory-lock-v1.md]] — live loop steps 2-3
