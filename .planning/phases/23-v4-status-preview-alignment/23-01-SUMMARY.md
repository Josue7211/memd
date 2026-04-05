# Phase 23 Summary: `v4` Status Preview Alignment

## Completed

- switched bundle status preview to `current_task` intent
- kept status preview on the fast local path without semantic fallback
- documented the new preview behavior in `README.md`

## Verification

- `cargo test -q -p memd-client`
- `cargo test -q`

## Outcome

Bundle diagnostics now reflect the same short-term path that operators and
agents use by default.
