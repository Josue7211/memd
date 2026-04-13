# Stale File References in Docs

Status: `closed` — FEATURES.md no longer exists
Created: 2026-04-13
Phase: cross-phase

## FEATURES.md

Files refactored from single files to directories:

- Lines 343, 580, 719, 765: `crates/memd-server/src/working.rs` → now `working/mod.rs`
- Lines 428, 479, 674: `crates/memd-client/src/render.rs` → now `render/mod.rs`
- Line 860: `crates/memd-server/src/repair.rs` → now `repair/mod.rs`

Deleted `.planning/` references:

- Line 195: `.planning/codebase/MEMORY-AUDIT.md`
- Line 480: `.planning/REQUIREMENTS.md`

## benchmark-registry.json

- Line 218: `crates/memd-server/src/working.rs` → now `working/mod.rs`

## Archive docs

20+ `.planning/` refs in `docs/superpowers/archive/`. Frozen history — low priority.
