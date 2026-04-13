# Shell-Unsafe memd Env Generation

<!-- BACKLOG_STATE
status: open
found: 2026-04-12
scope: bundle generation, helper scripts
-->

- status: `open`
- found: `2026-04-12`
- scope: bundle generation, helper scripts

## Summary

Generated `.memd/env` writes values that are not shell-safe, so helpers that
`source` the file can fail before running memd.

## Symptom

- helper scripts that `source .memd/env` can crash on values with spaces
- correction/write helper path broke while trying to record memory

## Root Cause

- `.memd/env` emitted shell assignments like
  `MEMD_PEER_GROUP_GOAL=coordinate the OpenClaw stack ...`
  without quoting/escaping
- helper scripts trust that file as shell-ready input

## Fix Shape

- emit shell-safe quoted values in `.memd/env`
- or stop sourcing shell text entirely and read structured config instead
- verify all generated helper scripts survive values with spaces

## Evidence

- [[docs/core/setup.md|Setup]]
- [[ROADMAP]]
