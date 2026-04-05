# Phase 21 Context: `v4` Checkpoint Refresh Writeback

## Why This Phase Exists

Short-term checkpoints are only useful if the rest of the bundle surface sees
them immediately. Requiring a second manual `resume` step after every checkpoint
would turn a fast memory flow back into ceremony.

This phase makes checkpoint writes refresh the bundle memory files
automatically.

## Inputs

- Phase 19: hot-path resume defaults
- Phase 20: dedicated short-term checkpoints

## Constraints

- keep refresh on the local hot path
- do not reintroduce semantic fallback into default checkpoint writes
- preserve the existing resume-backed memory file rendering

## Target Outcome

Short-term checkpoint writes should update the visible memory files with no
extra manual step.
