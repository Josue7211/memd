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

## Truly Dead Functions (zero callers anywhere)

- `helpers.rs:484` — `legacy_dashboard_html()` — never called
- `ui/mod.rs:49` — `empty_dashboard_html()` — never called
- `atlas.rs:594` — `persist_atlas_link()` — compiler warning, zero callers
- `preset.rs:127` — `is_wake_only_agent()` — compiler warning, zero callers

## Test-Only (suppressed correctly, keep)

- 6 harness pack render functions in `render/mod.rs` — called from tests only
- 7 autoresearch loop functions — Phase I scaffolding
- `HarnessPackView` trait + `render_harness_pack_markdown` — test infrastructure
- `render_harness_preset_markdown` — 1 test caller
