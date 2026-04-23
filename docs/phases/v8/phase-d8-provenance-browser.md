---
phase: D8
name: Provenance Browser
version: v8
status: planned
opened: 2026-04-22
depends_on: [A8, V7 E7]
axis: trust_provenance
plan_spec: docs/phases/v8/phase-d8-plan.md
---

# Phase D8: Provenance Browser

## Goal

Click any fact; see every past version + who captured it + why. Renders V7 E7 correction chain + V4 A4/C4 provenance as a timeline with source-turn excerpts.

## Why this phase exists

E7 ships CLI `memd fact provenance --chain`. Reviewers won't use CLI. Browser renders it as a readable timeline with hover-to-see-turn-content.

## Deliver

1. **Timeline view.** Vertical timeline per fact; each node = canonical version.
2. **Turn excerpt hover.** Source turn preview (first 3 lines) on hover; full turn in modal.
3. **Judge rationale.** For correction-promoted nodes, shows judge confidence + rationale.
4. **Copy-link.** Every timeline node has a stable URL.
5. **Embeddable widget.** Timeline embeddable in A8 atlas panel + B8 correction modal.

## Pass Gate

- pre: provenance CLI-only
- post: timeline renders for 20-link chain; source turns load < 200ms; E2E tests green
- evidence: playwright suite, screenshot set

## Product Win

"memd tells me why it believes what it believes." The trust surface.

## Evidence

- E2E tests
- screenshots
- 20-link perf number

## Fail Conditions

- Source turn content leaks cross-scope: visibility regression.
- Timeline renders > 500ms on 20 links: virtualize or prune chain display.

## Non-Goals

- Editing provenance (immutable; corrections are the edit primitive).
- Bulk provenance export (CLI exists).
