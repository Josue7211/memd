---
phase: A8
name: Atlas Navigation UI
version: v8
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [V7]
phase_doc: docs/phases/v8/phase-a8-atlas-navigation-ui.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: trust_provenance, procedural_reuse
---

# Phase A8 — Implementation Plan

## 0. Executive summary

Web app at `apps/memd-atlas/` (Vite + React + TS). Cytoscape graph over 500-node corpus; search/filter; node panel; keyboard nav; playwright suite.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `apps/memd-atlas/package.json` | pnpm workspace member. |
| `apps/memd-atlas/vite.config.ts` | Dev server at :5180. |
| `apps/memd-atlas/src/main.tsx` | Entry. |
| `apps/memd-atlas/src/App.tsx` | Shell + router. |
| `apps/memd-atlas/src/graph/AtlasGraph.tsx` | Cytoscape wrapper. |
| `apps/memd-atlas/src/graph/layout.ts` | Force layout config. |
| `apps/memd-atlas/src/panel/NodePanel.tsx` | Details panel. |
| `apps/memd-atlas/src/api/memd.ts` | HTTP client (typed). |
| `apps/memd-atlas/src/search/SearchBar.tsx` | Search + filter. |
| `apps/memd-atlas/src/keyboard/navigation.ts` | Kbd nav controller. |
| `apps/memd-atlas/tests/e2e/atlas.spec.ts` | Playwright. |
| `apps/memd-atlas/tests/perf/500-nodes.spec.ts` | Perf budget. |

### Files to modify

| Path | Change |
| --- | --- |
| `pnpm-workspace.yaml` | Add `apps/memd-atlas`. |
| `crates/memd-server/src/routes/atlas.rs` (new HTTP routes) | `/atlas/records`, `/atlas/chain/:id`. |
| Phase doc. |

## 2. Schema changes

None. HTTP-only.

HTTP shape:

```
GET /atlas/records?kind=&scope=&stage=&limit=500
GET /atlas/records/:id
GET /atlas/chain/:id
```

## 3. API shape

Local dev: `pnpm -C apps/memd-atlas dev` → http://localhost:5180.

## 4. Test matrix

1. `atlas_graph_renders_500_nodes_under_100ms`
2. `atlas_search_returns_under_50ms`
3. `atlas_node_panel_opens_on_click`
4. `atlas_node_panel_shows_kind_stage_content_provenance`
5. `atlas_filter_by_kind`
6. `atlas_filter_by_scope`
7. `atlas_filter_by_stage_excludes_retracted_by_default`
8. `atlas_keyboard_tab_navigation`
9. `atlas_keyboard_slash_focuses_search`
10. `atlas_private_scope_requires_auth_context`
11. `atlas_http_api_typed_client_schema_matches_server`
12. `atlas_playwright_smoke_green`

## 5. Fixtures

`apps/memd-atlas/tests/fixtures/corpus-500.json` — 500 records with mixed kinds/stages/scopes.

## 6. Telemetry

CI uploads playwright traces + screenshots to `docs/verification/v8-runs/ui/atlas/`.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_ATLAS_UI_ENABLED` | `0` | Serve atlas HTTP routes only when set. Graduated after A8 lands. |

## 8. Task list

### Task A8.1 — workspace + shell

- [ ] Scaffold Vite + React; hello-world green.
- [ ] Commit: `scaffold(apps/a8): memd-atlas app shell (A8)`.

### Task A8.2 — HTTP routes

- [ ] Implement `/atlas/records*`.
- [ ] Test 11 failing.
- [ ] Commit: `feat(server/a8): atlas HTTP routes (A8)`.

### Task A8.3 — graph

- [ ] Tests 1 + 3 failing.
- [ ] Cytoscape render 500-node.
- [ ] Commit: `feat(apps/a8): atlas graph render (A8)`.

### Task A8.4 — panel

- [ ] Test 4 failing.
- [ ] Commit: `feat(apps/a8): node details panel (A8)`.

### Task A8.5 — search + filter

- [ ] Tests 2 + 5 + 6 + 7 failing.
- [ ] Commit: `feat(apps/a8): search + filter (A8)`.

### Task A8.6 — keyboard nav

- [ ] Tests 8 + 9 failing.
- [ ] Commit: `feat(apps/a8): keyboard nav (A8)`.

### Task A8.7 — visibility guard

- [ ] Test 10 failing.
- [ ] Commit: `feat(apps/a8): visibility guard (A8)`.

### Task A8.8 — playwright + CI

- [ ] Test 12 failing.
- [ ] Screenshots to v8-runs.
- [ ] Commit: `ci(apps/a8): playwright smoke (A8)`.

## 9. Bench impact

None direct; indirect lift to trust_provenance + procedural_reuse axes.

## 10. Dependency graph

- Requires: V7 closed.
- Blocks: B8, C8, D8, E8, F8.

## Exit criteria

1. Tests 1–12 green.
2. Perf budgets hit.
3. Playwright CI green.
4. Screenshots committed.
5. Atomic commits.
