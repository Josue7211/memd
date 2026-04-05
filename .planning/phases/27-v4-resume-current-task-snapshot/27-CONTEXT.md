# Phase 27 Context: `v4` Resume Current-Task Snapshot

## Why This Phase Exists

Fast resume is not enough if the hot lane still reads like raw counts and
blobs. Operators need the active focus, pressure, next recovery target, and
lane at a glance.

## Inputs

- bundle-backed resume output
- prompt rendering path
- generated bundle markdown memory files

## Constraints

- keep the snapshot compact
- preserve the current-task hot lane as the default surface
- avoid transcript dumping

## Target Outcome

Resume surfaces show a compact current-task snapshot that is immediately useful
without deeper inspection.
