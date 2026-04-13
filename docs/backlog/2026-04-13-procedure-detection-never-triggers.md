# Procedure Detection Never Triggers in Runtime

- status: `closed`
- found: `2026-04-13`
- scope: memd-worker, memd-server
- severity: critical

## Summary

Phase G "verified" but `detect_procedures()` is only called in unit tests and
manual CLI (`memd procedure detect`). No runtime hook, no worker trigger, no
maintenance call. Procedures table is permanently empty during real usage.
The pipeline is 90% wired, 0% operational.

## Symptom

- `memd procedure list` returns empty
- Wake packet never shows procedures section
- Agents re-derive the same workflows every session

## Root Cause

- `detect_procedures()` at `procedural.rs:431-607` works correctly but is never called automatically
- `maintain_runtime()` at `store_runtime_maintenance.rs:68-172` does NOT include procedure detection
- memd-worker at `main.rs` runs verification + decay + consolidation but NOT procedure detection
- Phase G pass gate verified via isolated unit tests, not operational proof

## Fix Shape

- Add `client.procedure_detect()` call in memd-worker's maintenance loop after consolidation
- Or add procedure detection to `maintain_runtime()` modes
- Trigger: after consolidation completes, scan events for new procedures

## Evidence

- `crates/memd-server/src/procedural.rs:431-607` — `detect_procedures()` implementation
- `crates/memd-worker/src/main.rs:126` — worker loop (decay + consolidation, no detect)
- `crates/memd-server/src/store_runtime_maintenance.rs:68-172` — maintain modes (no detect)
- `crates/memd-server/src/routes.rs:1697` — `POST /procedures/detect` route exists
- `crates/memd-client/src/lib.rs` — `procedure_detect()` client method exists

## Dependencies

- independent: can be fixed standalone (one line in worker)
- blocks: [[docs/backlog/2026-04-13-dogfood-verification-gap.md|dogfood-verification-gap]] (eval assertion "procedures non-empty" depends on this)

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/locks/2026-04-11-memd-theory-lock-v1.md]] — live loop step 6 (update procedural memory)
- [[docs/phases/phase-g-procedural-learning.md]] — Phase G pass gate claims auto-detect
- [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md]] — 10-star axis: procedural reuse (15%)
