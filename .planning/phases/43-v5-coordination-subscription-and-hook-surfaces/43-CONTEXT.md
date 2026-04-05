# Context 43: `v5` Coordination Subscription and Hook Surfaces

## Why This Exists

Phase 42 added a useful local watch loop, but the change detection is still
embedded in the CLI path. Other surfaces such as hooks, MCP wrappers, and
future UI layers need a compact way to reuse the same coordination change feed
without duplicating polling and diff logic.

## Inputs

- phase 40 dashboard views
- phase 41 drilldown views
- phase 42 watch and alert views
- existing coordination inbox, recovery, policy, and receipt data

## Desired Outcome

Expose a compact subscription or hook-friendly change surface that reports only
meaningful coordination deltas across the same bounded categories already used
by dashboard, drilldown, and watch views.
