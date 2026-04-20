# ConvoMem Adapter Design

**Date:** 2026-04-20

**Goal**

Make `ConvoMem` retrieval benchmarking honest by aligning retrieved units, gold evidence units, and scoring units at the message level.

**Problem**

Current `ConvoMem` retrieval diagnostics are invalid:

- normalization stores `message_evidences` as raw objects
- expected targets are parsed as strings, so they collapse to empty
- retrieval invents `conv-*` chunk ids that do not correspond to evidence units

This can produce `0.0` even when relevant evidence is visibly present.

**Design**

Normalize each conversation message into a stable message id. Derive evidence target ids by matching each evidence object to one or more normalized conversation messages using exact `(speaker, text)` matching after light normalization.

The retrieval diagnostic should then:

- retrieve message-level docs with those stable ids
- compare retrieved ids against `message_evidence_ids`
- score with the same message-level contract MemPalace-style harnesses expect

**Scope**

- In:
  - stable message ids
  - evidence-id derivation during normalization
  - message-level retrieval docs for `ConvoMem`
  - tests proving ids line up and hits are possible
- Out:
  - full-eval `ConvoMem` protocol
  - public leaderboard copy updates
  - MemPalace cross-baseline replay

**Success**

- `ConvoMem` expected targets are non-empty when evidence exists
- retrieved ids and gold ids are the same unit type
- fresh `ConvoMem` rerun measures a real adapter contract instead of an impossible one
