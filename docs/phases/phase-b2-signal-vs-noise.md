---
phase: B2
name: Signal vs Noise
version: v2
status: pending
depends_on: [A2]
backlog_items: [42, 43]
---

# Phase B2: Signal vs Noise

## Goal

Facts, decisions, and procedures surface in wake packets. Status noise eliminated.

## Deliver

- Checkpoint dedup via redundancy_key
- Status cap enforced in working memory admission
- Wake kind scoring reweighted: facts/decisions outrank status
- Live memory contract: new captures appear in working memory within one cycle

## Pass Gate

- `memd eval` score ≥ 65 (from current ~35)
- Store a fact → next wake packet contains that fact (not buried under status)
- Working memory has ≤ 2 status items out of 8 slots
- Status noise reduction ≥ 80% measured by before/after item count

## Evidence

- E2E test: store fact, run wake, assert fact in packet
- Before/after working memory composition snapshots
- `memd eval` score regression test in CI

## Fail Conditions

- Facts still not surfacing after fix
- Status items still dominate working memory

## Rollback

- Revert scoring changes if eval score drops below pre-fix baseline
