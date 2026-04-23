---
phase: A8
name: Atlas Navigation UI
version: v8
status: planned
opened: 2026-04-22
depends_on: [V7]
axis: trust_provenance, procedural_reuse
plan_spec: docs/phases/v8/phase-a8-plan.md
---

# Phase A8: Atlas Navigation UI

## Goal

First UI surface for memd: interactive atlas graph over canonical records. Nodes = canonical facts, edges = correction chain + provenance pointers. Click a node → provenance panel. Search, filter by type, filter by visibility scope.

## Why this phase exists

memd's trust surface is invisible today. The atlas has been "fully built, completely dormant" (per V3 backlog). V8 wakes it up. A8 is the foundation every other V8 phase hangs on — B8 correction UX, C8 inspector, D8 provenance browser, E8 diff + rollback all embed atlas widgets.

## Deliver

1. **Web app shell.** `apps/memd-atlas/` (Vite + React + TypeScript). Uses existing memd-server HTTP API.
2. **Graph view.** Cytoscape.js or d3-force; renders 500-node corpus smoothly.
3. **Node details panel.** Shows record (id, kind, stage, content, provenance, chain preview).
4. **Search + filter.** Filter by kind, scope, stage (canonical/candidate/retracted), date range.
5. **Keyboard nav.** Tab through nodes; Enter to open panel; `/` to focus search.
6. **Dev server + CI-visual.** `pnpm dev` locally; CI screenshots via playwright.

## Pass Gate

- pre: no UI at all
- post: atlas renders 500-node corpus < 100ms; search returns in < 50ms; E2E tests green
- evidence: playwright traces, screenshots in `docs/verification/v8-runs/ui/atlas/`

## Product Win

First thing a stranger sees when they open memd. Competitors (mempalace graph, mem0 dashboard) are the bar — memd matches on day one of V8.

## Evidence

- playwright suite green
- screenshot set
- 500-node perf number

## Fail Conditions

- Renders but > 200ms interactive on 500 nodes: perf fix before merge.
- Private-scope records surface without auth context: V3 visibility regression.

## Non-Goals

- Writeable operations from atlas (B8 correction capture, E8 rollback).
- Mobile (out of V8 scope).
