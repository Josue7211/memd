# Skill Gating: Config Flags Only

- status: `open`
- severity: `medium`
- phase: `V2-M2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Skill gating is implemented as config flags (enable/disable). No runtime enforcement, evaluation criteria, or policy engine. Cannot conditionally gate skills based on memory state or agent competency.

## Fix

- Add skill gating policy layer (evaluation function)
- Implement runtime enforcement at skill call boundary
- Add criteria functions (memory recency, success rate, etc.)
- Add to phase-M2 acceptance criteria (adaptive gating)
