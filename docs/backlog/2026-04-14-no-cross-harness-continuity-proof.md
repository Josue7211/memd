# No Cross-Harness Continuity Proof

- status: `open`
- severity: `high`
- phase: `V2-L2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Theory says an agent can start work in one harness (e.g., Claude Code), context-switch to another (e.g., Codex), and resume with full continuity. No E2E test exists that proves this works. Cross-harness resume paths are untested.

## Fix

- Add E2E test: create session in harness-A, checkpoint, delete local state, resume in harness-B
- Verify working memory reconstructs correctly
- Verify inbox state persists across harness boundary
- Add to phase-L2 acceptance criteria (multi-agent continuity)
