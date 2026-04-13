# Ambiguous Glob Imports

Status: `open`
Created: 2026-04-13
Phase: cross-phase

`runtime/mod.rs` has `use super::*` (line 1) + `pub use resume::*` (line 20).
3 symbols are ambiguous. Will become a hard Rust error (issue #114095).

## Symbols

- `read_bundle_resume` — used in `memory_surface.rs:564`, `status_runtime.rs:112`
- `read_bundle_handoff` — used in `status_runtime.rs:164`
- `invalidate_bundle_runtime_caches` — used in `turn_runtime.rs:14`

## Fix

Add explicit imports in consuming files instead of relying on glob re-exports.
