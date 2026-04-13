# Silent Event Loss

- status: `closed`
- found: `2026-04-13`
- scope: memd-server

## Summary

Production paths use `let _ =` to discard errors from event recording.
Primary memory data is safe — events are best-effort telemetry written
after the main operation succeeds. But event spine integrity depends on
these, and session continuity reads from the event log.

## Symptom

- If DB is locked or disk full, event-spine entries silently vanish
- No log, no error, no signal to the caller
- Session continuity may show gaps in the event timeline

## Root Cause

- `let _ = self.record_item_event(...)` in 3 places in `main.rs`
- `let _ = self.store.record_event(...)` in `routes.rs:1351`
- `let _ = self.upsert_atlas_region(...)` in `atlas.rs:445,447`
- `let _ = self.auto_retire_stale_procedures()` in `procedural.rs:213`
- `let _ = record_lifecycle_event(...)` in `repair/mod.rs:32,231`

## Fix Shape

- Replace `let _ =` with `.inspect_err(|e| eprintln!("warn: {e:#}"))` at minimum
- Or propagate errors and let caller decide
- `main.rs` event writes are the highest priority — those feed the event spine

## Evidence

- cargo build shows no warnings for this (silent by design)
- Confirmed by grep: 12 production `let _ =` sites in memd-server
