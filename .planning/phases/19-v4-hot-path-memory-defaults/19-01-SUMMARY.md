# Phase 19 Summary: `v4` Hot-Path Memory Defaults

## Completed

- added explicit `--semantic` flags to bundle resume and handoff flows
- removed automatic semantic fallback from the default hot path
- kept evaluation and explicit deep-recall workflows semantic-aware
- updated README and generated bundle placeholders to teach the fast-default workflow
- added regression coverage for the generated memory placeholder guidance

## Verification

- `cargo test -q -p memd-client`
- `cargo test -q`

## Outcome

Short-term memory defaults are now aligned with the product requirement: fast,
local, and bundle-backed first; semantic recall only when explicitly requested.
