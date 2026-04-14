# No Selective Memory Reset

- status: `open`
- severity: `medium`
- phase: `V2-D2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Memory can only be reset in full (wipe everything). No surgical correction of single corrupted items. Fixing one bad fact requires losing everything else.

## Fix

- Add selective delete (by ID, tag, or key)
- Implement soft delete with restore capability
- Add to phase-D2 acceptance criteria (correction granularity)
