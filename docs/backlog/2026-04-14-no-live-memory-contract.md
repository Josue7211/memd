# No Live Memory Contract

- status: `open`
- severity: `high`
- phase: `V2-B2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Theory requires memory updates while agent works (live contract). No enforcement exists. Memory is read at session start and written at checkpoint—reads during session are stale. Corrections applied mid-session are not visible to the agent unless explicitly reloaded.

## Fix

- Implement live memory refresh contract (background reloads of updated items)
- Test that mid-session corrections propagate to agent
- Add enforcement at API boundary
- Add to phase-B2 acceptance criteria (live memory)
