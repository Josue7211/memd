# Phase 21 Summary: `v4` Checkpoint Refresh Writeback

## Completed

- refreshed bundle resume automatically after short-term checkpoint writes
- regenerated bundle memory files from the refreshed snapshot
- kept the refresh on the local hot path without semantic fallback
- updated README guidance for the new short-term writeback behavior

## Verification

- `cargo test -q -p memd-client`
- `cargo test -q`

## Outcome

Short-term checkpoints now update the visible memory surface immediately, so
agents can rely on them without an extra manual resume step.
