# Ambiguous Glob Imports

Status: `closed`
Created: 2026-04-13
Phase: cross-phase

`runtime/mod.rs` has `use super::*` (line 1) + `pub(crate) use resume::*` (line 20).
2 symbols ambiguous at 4 call sites. Will become a hard Rust error (issue #114095).
**BLOCKER** — next Rust edition upgrade breaks the build.

## Symbols

- `read_bundle_resume` — used in `memory_surface.rs:564`, `status_runtime.rs:112`, `status_runtime.rs:164`
- `invalidate_bundle_runtime_caches` — used in `turn_runtime.rs:14`

## Fix

Add explicit imports in consuming files instead of relying on glob re-exports.
Or remove `use super::*` from `runtime/mod.rs:1` and import needed items explicitly.
