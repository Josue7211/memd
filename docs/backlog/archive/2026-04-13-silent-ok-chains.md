# Silent .ok() Chains Drop Corrupt Data

- status: `closed`
- found: `2026-04-13`
- scope: memd-server

## Summary

Procedural and atlas store functions use `.filter_map(|r| r.ok())` to iterate
DB rows. If a row read fails or JSON parse fails, the row silently vanishes
from results. No error, no log, no count of dropped rows.

## Symptom

- Corrupted procedure JSON → procedure disappears from listings
- Failed atlas region parse → region invisible to explore
- User sees fewer results than expected with no explanation

## Root Cause

- `procedural.rs:69-70` — `.filter_map(|r| r.ok()).filter_map(|p| serde_json::from_str(&p).ok())`
- `procedural.rs:147-148`, `182`, `242-243`, `459` — same pattern
- `atlas.rs:51-52`, `116-117`, `307`, `588-589` — same pattern
- Double `.ok()` chain means BOTH DB read errors AND JSON parse errors are silent

## Fix Shape

- Log dropped rows at warn level with the error
- Or collect errors separately and include count in response
- At minimum: `filter_map(|r| r.inspect_err(|e| eprintln!("warn: {e}")).ok())`

## Evidence

- 13 `.ok()` sites in production code across procedural.rs and atlas.rs
