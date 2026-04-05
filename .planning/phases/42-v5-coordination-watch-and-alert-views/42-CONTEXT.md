# Context 42: `v5` Coordination Watch and Alert Views

## Why This Exists

Phase 40 introduced dashboard views and phase 41 added bounded drilldowns, but
operators still need to rerun coordination commands manually to notice changing
pressure. Active coworking needs a compact watch surface that can refresh over
time without turning into a transcript feed.

## Inputs

- phase 40 dashboard and history views
- phase 41 drilldown and filter views
- existing heartbeat, inbox, recovery, policy, and receipt primitives

## Desired Outcome

Add compact watch and alert surfaces that keep coordination pressure visible as
it changes, while reusing the same bounded categories from the dashboard and
drilldown work.
