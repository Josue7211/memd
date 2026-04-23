---
phase: A3
name: Mining
version: v2
status: verified
depends_on: []
backlog_items: [55]
---

# Phase A3: Mining

## Goal

Mine external systems for implementation patterns memd can port directly,
without diluting memd's truth model or product scope.

## Deliver

- Comparative teardown docs for mined targets under `docs/theory/teardowns/`
- A concrete donor map: what memd already does, what donor does better, what to copy, what to reject
- A shortlist of port-ready runtime patterns for V2 execution
- Roadmap updates that convert mining output into named implementation phases

## Pass Gate

- At least 2 mined systems have written teardown docs with explicit:
  - strongest idea
  - wrong idea
  - memd overlap
  - direct lift targets
- Every mined system ends with a judgment:
  - `steal now`
  - `reference only`
  - `reject`
- At least one mined pattern is promoted into roadmap language as a future implementation target
- Mining output stays implementation-specific, not vague inspiration prose

## Evidence

- `docs/theory/teardowns/` contains donor analyses with explicit memd recommendations
- `docs/theory/2026-04-14-donor-map.md` consolidates donor decisions and target phases
- `docs/THEORY.md` links the new teardown docs
- `ROADMAP.md` reflects the mining phase and its outputs

## Fail Conditions

- Mining notes only summarize repos without saying what memd should do
- Donor concepts are copied as branding instead of translated into memd architecture
- Mining expands scope without identifying the bounded implementation surface

## Rollback

- Remove the phase doc and theory teardown links if the work turns into unbounded research with no port target
