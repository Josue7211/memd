# Phase 31 Context: `v4` Automatic Short-Term Memory Management

## Why This Phase Exists

Short-term memory should improve with less manual ceremony. The bundle already
supports explicit checkpoints, but meaningful coordination transitions still
required operators to remember to capture state themselves.

This phase automates high-signal short-term capture on coordination boundaries
without turning memory into a transcript dump.

## Inputs

- bundle auto short-term capture policy
- claims, claim transfer, and peer message workflows
- current-task checkpoint and bundle refresh paths

## Constraints

- only capture high-signal state transitions
- do not dump raw transcript content
- keep the hot lane compact and current-task oriented
- refresh visible bundle memory after auto-capture

## Target Outcome

`memd` automatically captures meaningful coordination transitions into the
short-term hot lane while staying compact and inspectable.
