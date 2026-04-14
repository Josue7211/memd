# No Session Orphan Detection

- status: `open`
- severity: `high`
- phase: `V2-C2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

No way to distinguish crashed sessions from completed ones. Resume finds wrong continuation point. Orphaned sessions accumulate in DB. No heartbeat or liveness check.

## Fix

- Add session heartbeat (TTL)
- Implement liveness check on resume
- Add orphan detection and cleanup
- Add to phase-C2 acceptance criteria (session management)
