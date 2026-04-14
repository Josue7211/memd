# No Admission Control / Rate Limiting

- status: `open`
- severity: `medium`
- phase: `V2-J2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Memory API has no rate limiting or admission control. A single noisy agent can flood memory with low-quality items, degrading retrieval for other agents. No protection against DOS or spam.

## Fix

- Add per-agent rate limiter
- Implement admission scoring (reject low-confidence items)
- Add queue backpressure
- Add to phase-J2 acceptance criteria (stability)
