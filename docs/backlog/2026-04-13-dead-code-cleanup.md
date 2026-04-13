# Dead Code Cleanup

Status: `open`
Created: 2026-04-13
Phase: cross-phase

85 `#[allow(dead_code)]` or `#[allow(unused_imports)]` across 25 files.

## Top Offenders

- `bundle/mod.rs` — 13 suppressed
- `runtime/mod.rs` — 7 suppressed
- `render/mod.rs` — 6 suppressed
- `verification/mod.rs` — 6 suppressed
- `workflow/autoresearch/mod.rs` — 7 suppressed
- `awareness/mod.rs` — 5 suppressed
- `evaluation/mod.rs` — 5 suppressed
- `hive/mod.rs` — 4 suppressed
- `coordination/mod.rs` — 4 suppressed
- `workflow/mod.rs` — 4 suppressed

## Specific Dead Functions

- `atlas.rs:594` — `persist_atlas_link()` only used in tests
- `preset.rs:127` — `is_wake_only_agent()` only used in tests
- `helpers.rs:483` — suppressed dead helper
