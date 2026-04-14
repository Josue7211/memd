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

## Donor Extraction (from inspiration repos)

- **J2-D1** (Omegon `worktree.rs` — **DIRECT RUST LIFT**): Worktree-first parallel execution. `WorktreeInfo { path, branch, backend }`. Auto-detect jj vs git. Create worktree for physical isolation. Memory coordination sits on top.
- **J2-D2** (Smriti `WorkClaim`): Advisory claims with TTL. Non-blocking intent declarations: agent, scope, task_id, intent_type (implement/review/investigate), 4h default TTL. Query-time expiry filtering.
- **J2-D3** (Omegon `minds` table): Namespace-scoped memory isolation. Facts belong to exactly one mind. Search scoped to mind. Cascade delete on mind removal.
- **J2-D4** (Omegon `omegon-secrets` — **DIRECT RUST LIFT**): Secrets redaction with Aho-Corasick DFA. All secrets registered. Tool output passed through `redact()` before display.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert if enforcement breaks legitimate shared retrieval
