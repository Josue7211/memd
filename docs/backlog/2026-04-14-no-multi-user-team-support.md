# No Multi-User / Team Support

- status: `open`
- severity: `medium`
- phase: `V2-N2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Memory scoping is per-agent only. No concept of team membership, org hierarchy, or shared memory pools. Each agent is isolated. Collaborative workflows cannot share decision history or learned facts.

## Fix

- Add team/org scoping layer
- Implement memory access control (read/write/execute permissions)
- Add shared memory pool concept
- Add to phase-N2 acceptance criteria (collaboration)
