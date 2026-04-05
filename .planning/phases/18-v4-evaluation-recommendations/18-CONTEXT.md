# Phase 18 Context: `v4` Evaluation Recommendations

## Why This Phase Exists

Bundle evaluation can already score memory health, persist snapshots, diff
against a baseline, and fail under explicit gates. Operators still need to map
those findings into concrete corrective actions by hand.

This phase closes that gap by deriving actionable recommendations from the
actual resume snapshot.

## Inputs

- Phase 14: evaluation foundations
- Phase 15: snapshot persistence
- Phase 16: regression diffs
- Phase 17: failure gates

## Constraints

- recommendations must come from real bundle state, not generic canned text
- keep the summary compact and the markdown artifact richer
- avoid auto-executing policy changes in this slice

## Target Outcome

`memd eval` should tell an operator what to do next when memory quality is weak
or drifting.
