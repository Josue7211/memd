# Phase 29 Context: `v4` Remember Refresh Writeback

## Why This Phase Exists

Durable `remember` writes should not leave visible bundle files stale until the
next manual resume. The visible memory surface needs to stay aligned with what
was just written.

## Inputs

- existing `remember` flow
- bundle markdown memory generation
- bundle resume refresh path

## Constraints

- keep `remember` on the existing typed memory path
- refresh visible bundle files immediately after successful writeback
- avoid duplicating memory generation logic

## Target Outcome

Durable `remember` writes refresh the visible bundle memory surfaces
immediately.
