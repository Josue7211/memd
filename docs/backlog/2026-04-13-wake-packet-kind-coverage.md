# Wake Packet Memory Kind Coverage Asymmetry

- status: `open`
- found: `2026-04-13`
- scope: memd-client

## Summary

Wake packets only surface memory kinds that match retrieval intent via
`context_compact()`, `working()`, `inbox()` APIs. Kinds like Constraint
or Topology may exist in DB but never appear in wake packets if no
retrieval intent requests them.

## Symptom

- Agent stores a Constraint memory → next session wake packet doesn't include it
- Not a bug per se — a design gap for Phase H where harnesses need complete awareness

## Root Cause

- Wake packet is built from 3 API calls with fixed intent filters
- No "all kinds" sweep for wake packet compilation
- Memory kinds are filtered at retrieval time, not at packet build time

## Fix Shape

- Add a sweep pass in wake packet compilation for high-confidence canonical items
  across all kinds, not just those matching the current retrieval intent
- Or add a `wake_kinds` list to the harness preset that specifies which kinds
  to always include regardless of intent

## Evidence

- `wakeup.rs:85-279` — wake packet builder
- `resume/mod.rs:114-152` — the 3 API calls that feed the packet
