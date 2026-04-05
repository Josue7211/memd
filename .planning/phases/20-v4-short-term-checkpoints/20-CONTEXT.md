# Phase 20 Context: `v4` Short-Term Checkpoints

## Why This Phase Exists

Fast resume is only half the short-term memory problem. Operators also need a
quick way to capture current-task state without shaping full `remember`
requests every time.

This phase adds a dedicated checkpoint command that stays on the same typed
memory pipeline but defaults to short-term semantics.

## Inputs

- existing `remember` flow
- bundle-backed defaults for project, namespace, workspace, and visibility
- requirement that short-term memory be easy and fast to use

## Constraints

- do not fork storage logic away from `remember`
- keep checkpoints typed and inspectable
- default checkpoint memory to short-lived task state, not permanent facts

## Target Outcome

Operators can write current-task memory quickly with one command and sensible
short-term defaults.
