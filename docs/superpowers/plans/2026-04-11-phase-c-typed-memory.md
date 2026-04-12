# Phase C Typed Memory Implementation Plan

> Execute local only unless the user explicitly re-allows mini workers.

**Goal:** make the 10-star typed-memory model visible in code and behavior so memory no longer reads like one flat bucket.

**Phase bar:** working context, session continuity, episodic memory, semantic memory, procedural memory, candidate memory, and canonical memory must exist as explicit runtime concepts.

## First Slice

- map stored memory kinds into top-level typed-memory families
- keep stage explicit so candidate vs canonical stays visible
- surface typed-memory labels in lookup and bundle inspection output
- prove the mapping with narrow tests

## First Slice Pass

- lookup output shows top-level typed-memory labels
- lookup defaults change by retrieval intent instead of one flat default kind set
- bundle memory surfaces show top-level typed-memory labels
- working-memory traces carry typed-memory labels end to end
- mapping tests pass for semantic, procedural, episodic, and session continuity cases

## Guardrails

- do not relabel memory without behavioral or surface impact
- do not collapse candidate vs canonical into one label
- do not hide the underlying storage `kind`

## Next Slice

- make retrieval traces type-aware instead of only `kind`-aware
- make canonical/session continuity/procedural lanes visible in verification
- add docs and scorecard evidence for typed-memory behavior

## Completion Note

Phase C is now complete in code and verification:

- lookup output shows top-level typed-memory labels
- lookup defaults change by retrieval intent instead of one flat default kind set
- bundle memory surfaces show top-level typed-memory labels
- working-memory traces carry typed-memory labels end to end
- mapping tests pass for semantic, procedural, episodic, and session continuity cases
- compiled memory quality reports probe session continuity, procedural, and canonical lanes
- benchmark artifacts surface typed-retrieval evidence in `latest.md`
- verification surfaces now use the exact typed-trace proof name in an active test target
