# Claude Code Bootstrap Bridge Gap

<!-- BACKLOG_STATE
status: open
found: 2026-04-12
scope: roadmap, setup docs, harness wording
-->

- status: `open`
- found: `2026-04-12`
- scope: roadmap, setup docs, harness wording

## Summary

Claude Code does not have native memd bootstrap parity with Codex, but some
roadmap/doc wording still reads as if it does.

## Symptom

- users can infer that `memd init` or `memd setup` is universally sufficient
  across harnesses
- roadmap wording collapses native attach and bridge attach into one class

## Root Cause

- Codex native harness flow and Claude Code bridge-import flow were summarized
  together too loosely
- roadmap compression kept product intent but lost the harness distinction

## Fix Shape

- keep native Codex/OpenClaw attach wording separate from Claude Code bridge
  wording
- say explicitly that Claude Code depends on `CLAUDE_IMPORTS.md` plus
  `/memory` verification
- never use "universal bootstrap" language unless every harness actually has
  parity

## Evidence

- [[docs/core/setup.md|Setup]]
- [[MILESTONE-v1]]
- [[ROADMAP]]
