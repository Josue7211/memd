# Phase 1 Discussion Log

**Mode:** Auto
**Date:** 2026-04-04

## Auto Selection

[auto] Selected all gray areas:

- provenance drilldown
- repair action shape
- working-memory control semantics
- source-trust / procedural / self-model minimums

## Auto-Resolved Decisions

### Provenance drilldown

- use existing explain and memory surfaces as the primary extension point
- API and CLI first, UI later
- traverse from summary memory to source metadata and linked raw artifacts

### Repair actions

- add bounded, auditable repair actions
- map onto existing lifecycle semantics where possible
- avoid broad automatic rewrites in `v1`

### Working-memory control

- keep deterministic policy in `v1`
- make eviction reasons explicit and inspectable
- defer learned policy to `v2`

### Source-trust / procedural / self-model

- make these explicit enough to stop them from being implicit
- stay minimal and `v1`-scoped
- do not expand into full superhuman-memory semantics yet

## Deferred

- learned retrieval policy
- branchable beliefs and world models
- collective trust semantics

---

*Decisions are captured canonically in `01-CONTEXT.md`.*
