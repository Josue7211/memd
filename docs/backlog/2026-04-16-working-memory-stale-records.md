---
status: open
severity: critical
phase: A3
opened: 2026-04-16
scope: memd-core
---
# Working Memory Holds Stale Records After Phase Completion

- status: `open`
- severity: `critical`
- phase: `V2-M2-evo`
- opened: `2026-04-16`
- scope: memd-core

## Problem

Verified phase status records still occupy working-memory slots long after phase
completion. Expiry and archival do not run when a phase flips to complete, so the hot
window fills with old status instead of architecture decisions and live task context.
This is directly visible in resume bloat and truncation.

## Fix

- Trigger archive/expire flow when milestone or phase completion is recorded
- Add lifecycle tests that prove stale status leaves working memory after completion
- Re-score resume bloat after archival to prove the working set recovers budget
- Prefer compact phase handoff records over many parallel canonical status rows
