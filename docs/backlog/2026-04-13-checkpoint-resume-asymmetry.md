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

- `checkpoint.rs:171-191` — checkpoint writes
- `resume/mod.rs:238-260` — resume reads
