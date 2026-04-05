# Phase 28 Context: `v4` Resume Change Summaries

## Why This Phase Exists

Resume should explain what changed since the last pickup. Without that delta,
operators still have to diff the hot lane mentally or replay prior context.

## Inputs

- bundle resume state file
- prompt rendering path
- status and bundle markdown surfaces

## Constraints

- keep the delta compact
- compare hot-lane state, not full transcripts
- persist the previous snapshot locally inside the bundle

## Target Outcome

Resume can show a compact "since last resume" delta across prompt and bundle
surfaces.
