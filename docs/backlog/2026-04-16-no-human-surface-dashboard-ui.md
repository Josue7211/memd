# No Human Surface / Dashboard UI

- status: `open`
- severity: `high`
- phase: `V2-I2`
- opened: `2026-04-16`
- scope: memd-dashboard

## Problem

memd has partial browser routes and components, but no complete human-facing surface
that satisfies the product contract. The current dashboard is split across stale server
HTML and a standalone SPA, graph/status flows are incomplete, and correction/navigation
are not yet proven end to end in a real browser.

## Fix

- Serve one dashboard from `memd-server` on the same origin
- Finish browse, graph, correction, procedure, and honest status routes against live data
- Remove stub status paths and hardcoded env assumptions
- Add browser E2E coverage with zero console errors and real pass-gate evidence
