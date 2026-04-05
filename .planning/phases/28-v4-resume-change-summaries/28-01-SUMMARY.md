# Phase 28 Summary: `v4` Resume Change Summaries

## Completed

- persisted the last hot-lane snapshot under `state/last-resume.json`
- added compact "since last resume" deltas to prompt and bundle views
- kept the comparison anchored to the hot lane instead of transcripts

## Verification

- `cargo test -q -p memd-client`
- `cargo test -q`

## Outcome

Resume now explains what changed instead of only dumping the current state.
