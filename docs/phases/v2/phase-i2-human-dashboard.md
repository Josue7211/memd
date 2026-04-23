---
phase: I2
name: Human Dashboard
version: v2
status: pending
depends_on: [D2, E2, G2]
backlog_items:
  - "2026-04-16-no-human-surface-dashboard-ui"
  - "2026-04-15-dashboard-not-served-from-memd-server"
  - "2026-04-15-graph-page-crash-entity-search-type-mismatch"
  - "2026-04-15-memory-entity-record-type-mismatch"
  - "2026-04-15-dashboard-env-hardcoded-tailscale-ip"
  - "2026-04-15-memd-preferences-not-persisted-across-sessions"
---

# Phase I2: Human Dashboard

## Goal

Humans can browse, correct, and navigate memory through a web UI.

## Deliver

- Memory browser (list, search, filter by kind/lane/tag)
- Correction UI (edit, supersede, mark contested)
- Atlas graph view (regions, entities, links)
- Procedure viewer
- Status dashboard with honest scoring

## Pass Gate

- User can find a specific fact in < 3 clicks
- User can correct a wrong fact through the UI
- Atlas graph renders with clickable nodes
- Status shows real eval score, not lies
- Zero console errors in browser

## Evidence

- Browser test screenshots
- Correction flow walkthrough
- Graph rendering proof
- Console error check

## Fail Conditions

- UI renders but doesn't connect to live data
- Corrections don't persist
- Graph is empty or unnavigable

## Donor Extraction (from inspiration repos)

- **I2-D1** (supermemory `memory-graph/`): Data-driven graph component. React component receives pre-fetched `GraphApiDocument[]` — no API calls from graph itself. App provides `onLoadMore` callback.
- **I2-D2** (supermemory `shared/types.ts`): Stable vs recent profile projection. Split memory into `static` (canonical facts) and `dynamic` (working context) for display. Canonical pinned at top.
- **I2-D3** (Smriti CLI): Compact state brief. `--compact` flag omits artifact content, preserves labels + recovery commands. Full details via `memd explain <id>`.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert UI changes that break API
