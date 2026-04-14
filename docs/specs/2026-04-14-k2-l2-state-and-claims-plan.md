# K2/L2 Port Plan: Canonical State, Claims, Freshness, Divergence

## Goal

Turn the best `A3` mining output into the first implementation target:

- `Omegon`-style canonical runtime/state surface
- `Smriti`-style claims, freshness checks, and divergence signal

This plan deliberately spans both:

- `K2` Observability
- `L2` Hive Hardening

because the operator state surface and coordination surface should be built together.

## Why This Is First

This target has the best ratio of:

- high leverage
- bounded scope
- low theory risk
- direct operator benefit

It also reduces ambiguity before later work in:

- `F2` ingestion
- `E2` atlas
- `N2` integrations

## User-Facing Outcome

One command:

- `memd state`

should answer:

- what truth is active
- what session or workspace is active
- what claims exist
- whether memory is stale
- whether branches disagree
- what needs operator attention now

## Scope

### K2 slice

- canonical `memd state` output
- startup/runtime health summary
- freshness summary
- divergence summary
- structured state JSON for UI and future adapters

### L2 slice

- claim create
- claim list
- claim close
- TTL expiry handling
- conflict detection hooks

## Non-Goals

- queen deny/reroute enforcement in full
- backup/restore
- full cross-harness E2E
- exact restore/fork boundary implementation

Those stay in later `L2` / `J2` work after the shared surface exists.

## Proposed Commands

### `memd state`

Human-readable canonical operator brief.

Sections:

- live truth
- focus
- workspace / branch / session
- claim summary
- freshness
- divergence
- memory health
- warnings

### `memd state --json`

Machine-readable version for:

- dashboard
- adapters
- tests

### `memd claim create`

Inputs:

- agent
- scope
- intent type
- task id
- workspace or branch
- ttl

### `memd claim list`

Outputs:

- active claims
- expired claims
- likely conflicts

### `memd claim close`

Inputs:

- claim id
- optional outcome

## Data Shape

### Claim

```json
{
  "id": "claim_123",
  "agent": "codex",
  "scope": "repo",
  "intent_type": "edit",
  "task_id": "k2-state-surface",
  "workspace": "main",
  "branch": "feature/state-surface",
  "status": "active",
  "ttl_seconds": 1800,
  "created_at": "2026-04-14T12:00:00Z",
  "expires_at": "2026-04-14T12:30:00Z"
}
```

### State summary

```json
{
  "live_truth": {},
  "focus": {},
  "session": {},
  "claims": {
    "active": [],
    "conflicts": []
  },
  "freshness": {
    "status": "unchanged",
    "since_checkpoint": null
  },
  "divergence": {
    "status": "none",
    "items": []
  },
  "warnings": []
}
```

## Execution Order

1. Add internal state assembler
   - one function builds the full operator state model
2. Add `memd state` human renderer
3. Add `memd state --json`
4. Add claim model + storage
5. Add claim CLI
6. Add freshness computation
7. Add divergence computation
8. Add tests

## Evidence Needed

- `memd state` snapshot fixture
- `memd state --json` schema fixture
- claim lifecycle test
- TTL expiry test
- freshness changed vs unchanged test
- divergence signal test with two conflicting branch decisions

## Risks

- if state assembly pulls from too many places, output becomes slow and noisy
- if claims are advisory only with no signal quality, users ignore them
- if divergence is over-eager, operators get alert fatigue

## Guardrails

- default output must fit on one screen
- warnings only when action is needed
- divergence shows summary first, not raw diff spam
- claims must expire automatically

## Direct File Targets

- `crates/memd-client/src/...state...`
- `crates/memd-client/src/...claim...`
- `crates/memd-client/src/cli/...`
- tests under `crates/memd-client/src/main_tests/`

Exact files should be chosen after codebase read in the execution phase.

## Definition of Ready

This plan is ready when work starts on:

- `K2` state surface
- `L2` claims/freshness/divergence

without reopening donor analysis.
