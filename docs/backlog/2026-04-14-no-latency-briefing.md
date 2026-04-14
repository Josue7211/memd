# No Latency / SLA Briefing

- status: `open`
- severity: `medium`
- phase: `V2-K2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Retrieval latency unmeasured. No SLA defined. No performance contract. Agents don't know if memory API is blocking them or running in acceptable bounds.

## Fix

- Add latency instrumentation (p50, p95, p99)
- Define SLA (e.g., 100ms for retrieve, 50ms for checkpoint)
- Measure against target in CI
- Document performance expectations
