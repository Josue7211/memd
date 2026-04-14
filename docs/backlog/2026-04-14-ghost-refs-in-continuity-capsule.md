# Ghost Refs in Continuity Capsule

status: open
severity: critical
phase: Phase I
opened: 2026-04-14

## Problem

ContinuityCapsule fields (left_off, blocker, next_action) reference
inbox items that may be expired or deleted. Expired items are never
filtered from continuity. Resume shows stale refs to `.planning/ROADMAP.md`
(file deleted), expired items, and orphaned IDs.

## Evidence

- resume/mod.rs ~line 1450-1457: no expired item filtering
- Inbox contains expired items marked Expired status
- No source_path validation (file may not exist)

## Fix

1. Filter expired items from continuity capsule fields
2. Validate source_path exists before including in capsule
3. Add ghost ref cleanup to drain/maintenance cycle
