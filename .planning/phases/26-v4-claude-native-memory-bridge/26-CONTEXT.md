# Phase 26 Context: `v4` Claude Native Memory Bridge

## Why This Phase Exists

Claude Code already has a native memory system built around `CLAUDE.md`,
imports, `/memory`, and dream-style consolidation flows. `memd` should bridge
into that surface instead of forcing Claude onto a parallel convention.

## Inputs

- existing bundle memory files
- Claude Code native import model
- requirement that `memd` remain the source of truth while Claude loads memory
  natively

## Constraints

- do not invent a fake Claude memory path
- keep `memd` as the canonical memory substrate
- make the bridge obvious to verify from inside Claude Code

## Target Outcome

Bundles generate a Claude-native import target and example wiring so Claude can
load `memd` memory through `CLAUDE.md` and `/memory`.
