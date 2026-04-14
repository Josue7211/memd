# Contradiction Detection Never Triggers

- status: `open`
- severity: `high`
- phase: `V2-D2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Trust subsystem defines contested_status for items where multiple versions exist with conflicting content. Contradiction detection logic exists in code but never fires in practice—test coverage is absent and production runs show no contradictions detected despite real conflicts in memory.

## Fix

- Audit contradiction detection condition (why doesn't it trigger?)
- Add unit tests with synthetic contested scenarios
- Verify detector fires on real conflict + captures evidence
- Add to phase-D2 acceptance criteria (dispute resolution)
