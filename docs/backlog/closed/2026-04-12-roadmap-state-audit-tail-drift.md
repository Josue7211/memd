# Roadmap State Audit-Tail Drift

<!-- BACKLOG_STATE
status: open
found: 2026-04-12
scope: roadmap truth, continuity truth
-->

- status: `open`
- found: `2026-04-12`
- scope: roadmap truth, continuity truth

## Summary

Roadmap phase state and live continuity state can disagree while a phase is
engineering-verified but still has audit-tail work open.

## Symptom

- `ROADMAP.md` can say a phase is effectively done
- continuity memory can still report that same phase as current with remaining
  audit work
- agents get mixed signals about whether to advance to the next phase

## Root Cause

- there is no single explicit rule for when a phase flips from active audit tail
  to next-phase active work
- roadmap truth and continuity truth were updated by different heuristics

## Fix Shape

- define one phase-state transition rule
- apply it to roadmap, continuity output, and status surfaces
- keep `verified`, `verified_with_audit_tail`, and `complete` semantics aligned

## Evidence

- [[MILESTONE-v1]]
- [[2026-04-11-memd-ralph-roadmap]]
- [[ROADMAP]]
