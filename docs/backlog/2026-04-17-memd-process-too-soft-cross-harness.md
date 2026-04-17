---
status: open
severity: high
phase: A3
opened: 2026-04-17
scope: harness-core
---
# memd Process Too Soft Across Harnesses

- status: `open`
- severity: `high`
- phase: `V2-N2`
- opened: `2026-04-17`
- scope: harness-core

## Problem

memd process enforcement is too soft across harnesses. A session can successfully
load memd wake state, then drift into the host harness's default workflow instead of
staying inside memd's own process.

In practice this means an agent can:

- read `wake.md` once, then stop following memd lifecycle rules
- ignore repo voice mode after startup
- use generic planning or branch-finish flows instead of roadmap-driven memd flow
- skip `memd checkpoint` at the end of meaningful work
- skip `memd handoff` before offering completion / next-step options
- fail to apply `--roadmap-set current_phase=... --roadmap-set phase_status=...`
  even when the repo advertises roadmap-driven state

That makes memd feel advisory instead of authoritative. If a model using Claude,
Codex, Grok, Kimi, OpenCode, or another harness can drift after wake, then memd is
not yet enforcing its own product contract.

## Evidence

- secret-broker session on 2026-04-17 loaded memd wake/resume/handoff surfaces, but
  then drifted into generic planning and generic branch-finish workflow
- repo voice mode was not enforced continuously after startup
- memd checkpoint/handoff closeout had to be repaired manually at the end
- roadmap state existed in memd instructions, but agent behavior was not blocked
  when it ignored that path
- current cross-harness surfaces (`.memd/agents/codex.sh`,
  `.memd/agents/CLAUDE_IMPORTS.md`) mostly provide reminders, not hard gates

## Root Cause Hypotheses

1. memd policy is encoded mainly as prose in `wake.md` instead of machine-readable
   required-state
2. wake/resume do not emit enforceable lifecycle receipts that later steps can verify
3. harness integrations import memd context, but do not run a pre-send validator
4. roadmap presence does not automatically block generic non-memd workflow tools
5. voice mode is announced, but not validated on every outgoing response
6. memd has no explicit `drifted` session state that forces recovery before completion

## Fix

- add machine-readable bundle policy for required lifecycle steps
- add receipts for wake, resume, roadmap, checkpoint, and handoff
- add cross-harness pre-send validator that blocks output when required memd steps are
  missing or stale
- add strict / locked enforcement modes so memd can hard-fail instead of reminding
- make roadmap state first-class and block generic finish/plan flows when roadmap
  requirements are active
- add drift detection and repair flow when a session leaves memd process
- add cross-harness tests proving a session cannot complete work without checkpoint +
  handoff after meaningful repo changes
