---
phase: C8
name: Memory Inspector
version: v8
status: planned
opened: 2026-04-22
depends_on: [A8]
axis: trust_provenance
plan_spec: docs/phases/v8/phase-c8-plan.md
---

# Phase C8: Memory Inspector

## Goal

Tabular, searchable, filterable view of every record memd holds. Complement to A8's graph view: when you need a list, not a map.

## Why this phase exists

Graph is for exploration; a table is for audit. A reviewer verifying "does memd actually hold this fact?" needs a table with columns.

## Deliver

1. **Records table.** Virtualized (TanStack Table); renders 100k rows.
2. **Columns.** id, kind, stage, scope, content preview, captured_at, updated_at, chain_length.
3. **Filters.** All columns filterable; saved filter presets per user.
4. **Export.** CSV + NDJSON export of current filter.
5. **Drill-in.** Row → same node panel as A8.

## Pass Gate

- pre: opaque memory bag
- post: 100k rows scroll smoothly; filter applies < 100ms; E2E tests green
- evidence: playwright suite, perf numbers

## Product Win

"I can audit what memd knows about me" claim is backed.

## Evidence

- E2E tests
- perf numbers
- export sample

## Fail Conditions

- Scroll jank > 50ms frame time on 100k rows: virtualization fix before merge.
- Filter ignores visibility scope: hard fail.

## Non-Goals

- In-table editing (B8 handles corrections).
- Bulk delete (out of V8 scope).
