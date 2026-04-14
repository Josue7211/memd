# No Backward Compatibility Contract

- status: `open`
- severity: `medium`
- phase: `V2-K2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Schema changes break old memory bundles. No migration path. No compatibility guarantee. Upgrading memd risks data loss.

## Fix

- Add schema versioning
- Implement migration functions for schema changes
- Add compatibility layer (read old versions)
- Test upgrade path end-to-end
