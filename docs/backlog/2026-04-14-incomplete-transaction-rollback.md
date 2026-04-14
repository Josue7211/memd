# Incomplete Transaction Rollback

- status: `open`
- severity: `high`
- phase: `V2-C2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Checkpoint pipeline writes multiple items in sequence. Partial failures (e.g., second write fails after first succeeds) leave DB in inconsistent state. No rollback. Corruption on error.

## Fix

- Wrap checkpoint in transaction
- Add rollback on partial failure
- Test failure scenarios (disk full, timeout mid-write)
- Add to phase-C2 acceptance criteria (atomicity)
