# Phase 22 Context: `v4` Current-Task Resume Defaults

## Why This Phase Exists

Even with a fast hot path and short-term checkpoints, launch surfaces still
matter. If attach snippets and generated agent scripts resume with a generic
intent, the short-term lane is underused by default.

This phase makes current-task intent the default in those launch surfaces.

## Inputs

- Phase 19: hot-path resume defaults
- Phase 20: short-term checkpoints
- Phase 21: checkpoint refresh writeback

## Constraints

- keep the local fast resume path
- do not force semantic fallback
- preserve explicit overrides through normal CLI arguments

## Target Outcome

Attach snippets and agent scripts should resume with `current_task` intent by
default.
