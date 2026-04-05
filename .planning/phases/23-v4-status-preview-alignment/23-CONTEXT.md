# Phase 23 Context: `v4` Status Preview Alignment

## Why This Phase Exists

Once current-task intent became the default launch behavior, bundle status
preview still lagged behind and inspected a more generic resume path. That
makes diagnostics less trustworthy than the real operator experience.

This phase aligns the preview with the actual hot path.

## Inputs

- Phase 22: current-task resume defaults
- existing bundle status preview

## Constraints

- keep the preview cheap and local
- do not reintroduce semantic fallback
- reflect the real default launch contract

## Target Outcome

`memd status --output .memd` should preview the same current-task lane that
default attach and agent launch flows use.
