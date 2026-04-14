---
phase: J2
name: Isolation + Trust
version: v2
status: pending
depends_on: [D2, G2]
backlog_items: [47, 48, 68, 73]
---

# Phase J2: Isolation + Trust

## Goal

Scope, visibility, and trust hierarchy enforced. Not just fields in schema.

## Deliver

- Scope check on all retrieval endpoints
- Visibility enforcement at retrieval and handoff
- Per-agent working context isolation
- Source quality ranking enforced (canonical > promoted > candidate)
- Multi-project isolation proof

## Pass Gate

- Adversarial test: agent A stores Private item → agent B cannot retrieve it
- Multi-project test: project X item not in project Y retrieval
- Trust test: canonical item outranks candidate item in retrieval
- Per-agent test: agent A's working context invisible to agent B

## Evidence

- Adversarial visibility test (automated)
- Multi-project isolation test
- Trust ranking test
- Agent isolation test

## Fail Conditions

- Private items leak across agents
- Project items cross-contaminate
- Candidate outranks canonical

## Rollback

- Revert if enforcement breaks legitimate shared retrieval
