---
phase: C8
name: Memory Inspector
version: v8
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [A8]
phase_doc: docs/phases/v8/phase-c8-memory-inspector.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: trust_provenance
---

# Phase C8 — Implementation Plan

## 0. Executive summary

TanStack virtualized table; 100k rows; filter < 100ms; CSV + NDJSON export. Drill-in to A8 panel.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `apps/memd-atlas/src/inspector/InspectorRoute.tsx` | /inspector page. |
| `apps/memd-atlas/src/inspector/RecordsTable.tsx` | Virtualized table. |
| `apps/memd-atlas/src/inspector/FilterBar.tsx` | Column filters + presets. |
| `apps/memd-atlas/src/inspector/Export.tsx` | CSV/NDJSON. |
| `crates/memd-server/src/routes/inspector.rs` | Paged records endpoint. |
| `apps/memd-atlas/tests/e2e/inspector.spec.ts` | E2E. |
| `apps/memd-atlas/tests/perf/100k-rows.spec.ts` | Perf. |

### Files to modify

| Path | Change |
| --- | --- |
| Router in `App.tsx` | Add `/inspector`. |
| Phase doc. |

## 2. Schema changes

None.

HTTP:

```
GET /inspector/records?cursor=&page_size=1000&kind=&stage=&scope=&q=&from=&to=
```

Cursor-paged response with stable ordering.

## 3. API shape

Local: http://localhost:5180/inspector.

## 4. Test matrix

1. `table_renders_100k_rows_without_jank`
2. `scroll_frame_time_p95_under_16ms`
3. `filter_applies_under_100ms`
4. `filter_preset_save_and_reload`
5. `csv_export_matches_filter`
6. `ndjson_export_matches_filter`
7. `row_click_opens_a8_node_panel`
8. `visibility_filter_respected`
9. `cursor_paging_stable_across_refresh`
10. `e2e_smoke_green`

## 5. Fixtures

- `apps/memd-atlas/tests/fixtures/records-100k.ndjson` — synthetic corpus for perf test.
- `apps/memd-atlas/tests/fixtures/records-5k-mixed-scope.ndjson` — functional tests.

## 6. Telemetry

Perf traces per CI run → `docs/verification/v8-runs/ui/inspector/`.

## 7. Feature flags

None (UI-only).

## 8. Task list

### Task C8.1 — HTTP paged endpoint

- [ ] Test 9 failing.
- [ ] Commit: `feat(server/c8): paged inspector endpoint (C8)`.

### Task C8.2 — virtualized table

- [ ] Tests 1 + 2 failing.
- [ ] Commit: `feat(apps/c8): records table (C8)`.

### Task C8.3 — filters + presets

- [ ] Tests 3 + 4 failing.
- [ ] Commit: `feat(apps/c8): filters + presets (C8)`.

### Task C8.4 — export

- [ ] Tests 5 + 6 failing.
- [ ] Commit: `feat(apps/c8): export (C8)`.

### Task C8.5 — drill-in + visibility

- [ ] Tests 7 + 8 failing.
- [ ] Commit: `feat(apps/c8): drill-in + visibility (C8)`.

### Task C8.6 — E2E + perf CI

- [ ] Test 10 failing.
- [ ] Commit: `ci(apps/c8): inspector smoke + perf (C8)`.

## 9. Bench impact

None.

## 10. Dependency graph

- Requires: A8.
- Blocks: F8.

## Exit criteria

1. Tests 1–10 green.
2. 100k-row perf budget hit.
3. Atomic commits.
