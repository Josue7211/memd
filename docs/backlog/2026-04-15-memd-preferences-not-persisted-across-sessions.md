# memd Preferences and Architecture Knowledge Not Persisted Across Sessions

status: open
severity: critical
phase: core
opened: 2026-04-15

## Problem

Agents using memd do not retain project-level preferences, architecture decisions, or
workflow conventions across sessions. Every new session, the agent forgets:

- Where backlogs are tracked (docs/backlog/)
- How handoffs work (ROADMAP.md + phase docs + memd checkpoint, not random markdown files)
- How state is saved (memd commands, not Claude Code memory or superpowers plans)
- The project's architecture (memd-server is the single gateway on 8787)
- Established conventions (one UI, not two; backlog format; phase doc structure)

This means the user has to re-explain the same preferences every session. The agent
makes architecture-violating decisions (like building a disconnected React app on a
separate port) because it doesn't recall that memd-server is the single process.

This is the core value proposition of memd — durable memory that changes agent behavior.
If the agent can't remember how to use memd's own project, memd isn't working.

## Evidence

- I2 session: agent built React dashboard on port 5173 instead of serving from memd-server at 8787
- I2 session: agent wrote handoff plan to docs/superpowers/plans/ instead of using ROADMAP.md + backlog + memd checkpoint
- I2 session: agent tried to use Claude Code memory system instead of memd
- Agent did not recall backlog lives in docs/backlog/ despite having done this before
- Agent did not recall ROADMAP.md is authoritative state
- Agent did not recall memd-server serves its own UI

## Root Cause Hypotheses

1. Wake packet doesn't include enough architecture/preference data
2. Preferences stored as memories but not surfaced in wake/resume
3. Architecture decisions stored but not retrievable via memd lookup
4. Lane files exist but ingestion pipeline doesn't re-ingest them on wake
5. Too much status noise in wake packet crowds out architecture/preference memories

## Fix

1. Audit what `memd wake` actually surfaces — does it include architecture decisions?
2. Audit what `memd resume` returns — are preferences in the output?
3. Store critical architecture decisions as high-confidence canonical facts with appropriate lanes
4. Ensure wake packet includes top architecture + preference memories, not just status
5. E2E test: store preference → new session → agent recalls preference without prompting
6. This should be highest priority — it affects every memd feature
