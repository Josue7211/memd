---
phase: D8
name: Provenance Browser
version: v8
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [A8, V7 E7]
phase_doc: docs/phases/v8/phase-d8-provenance-browser.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: trust_provenance
---

# Phase D8 — Implementation Plan

## 0. Executive summary

Timeline UI over V7 E7 correction chain + V4 provenance. Turn excerpts on hover; judge rationale on correction nodes; stable shareable URLs; embeddable widget.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `apps/memd-atlas/src/provenance/TimelineRoute.tsx` | /fact/:id route. |
| `apps/memd-atlas/src/provenance/TimelineView.tsx` | Vertical timeline. |
| `apps/memd-atlas/src/provenance/TurnExcerpt.tsx` | Hover preview + modal. |
| `apps/memd-atlas/src/provenance/EmbeddableWidget.tsx` | Reusable widget. |
| `crates/memd-server/src/routes/provenance.rs` | Chain + turn endpoints. |
| `apps/memd-atlas/tests/e2e/provenance.spec.ts` | E2E. |

### Files to modify

| Path | Change |
| --- | --- |
| `apps/memd-atlas/src/panel/NodePanel.tsx` (A8) | Embed timeline widget. |
| `apps/memd-atlas/src/correction/CorrectionModal.tsx` (B8) | Embed timeline widget. |
| Phase doc. |

## 2. Schema changes

None.

HTTP:

```
GET /provenance/chain/:record_id        -> ChainLink[]
GET /provenance/turn/:turn_id           -> { content, captured_at, speaker, excerpt_first_3 }
```

## 3. API shape

Local: http://localhost:5180/fact/:id.

## 4. Test matrix

1. `timeline_renders_20_link_chain`
2. `turn_excerpt_loads_under_200ms`
3. `judge_rationale_shown_on_correction_nodes`
4. `stable_url_navigates_to_timeline_node`
5. `embeddable_widget_renders_in_atlas_panel`
6. `embeddable_widget_renders_in_correction_modal`
7. `turn_content_respects_visibility`
8. `timeline_virtualizes_above_50_links`
9. `e2e_smoke_green`

## 5. Fixtures

- `apps/memd-atlas/tests/fixtures/chain-20.json`
- `apps/memd-atlas/tests/fixtures/chain-100-stress.json`

## 6. Telemetry

E2E screenshots → `docs/verification/v8-runs/ui/provenance/`.

## 7. Feature flags

None.

## 8. Task list

### Task D8.1 — HTTP chain + turn

- [ ] Tests 2 + 7 failing.
- [ ] Commit: `feat(server/d8): provenance endpoints (D8)`.

### Task D8.2 — timeline

- [ ] Tests 1 + 3 failing.
- [ ] Commit: `feat(apps/d8): timeline view (D8)`.

### Task D8.3 — turn excerpt

- [ ] Tests 2 + 7 additional.
- [ ] Commit: `feat(apps/d8): turn excerpt (D8)`.

### Task D8.4 — stable URL

- [ ] Test 4 failing.
- [ ] Commit: `feat(apps/d8): stable URL (D8)`.

### Task D8.5 — embeddable widget

- [ ] Tests 5 + 6 failing.
- [ ] Commit: `feat(apps/d8): embeddable widget (D8)`.

### Task D8.6 — virtualize long chains

- [ ] Test 8 failing.
- [ ] Commit: `feat(apps/d8): virtualize >50 links (D8)`.

### Task D8.7 — E2E + CI

- [ ] Test 9 failing.
- [ ] Commit: `ci(apps/d8): provenance smoke (D8)`.

## 9. Bench impact

None direct; extends V5 E5 axis visibility.

## 10. Dependency graph

- Requires: A8, V7 E7.
- Blocks: E8 (reuses widget), F8.

## Exit criteria

1. Tests 1–9 green.
2. 20-link render < 500ms.
3. Widget embedded in A8 + B8.
4. Atomic commits.
