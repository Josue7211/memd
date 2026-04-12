# memd Roadmap

`ROADMAP.md` is the single roadmap source of truth for this repo.

<!-- ROADMAP_STATE
truth_date: 2026-04-12
version: v1
version_status: in_progress
current_phase: phase_e
phase_status: verified_with_audit_tail
next_phase: phase_f
next_step: cross_harness_wake_packet_proof
active_blockers:
  - feature_v1_wake_packet_audit
-->

## Status Snapshot

- truth date: `2026-04-12`
- current version: `v1`
- version status: `in_progress`
- current phase: `Phase E: Wake Packet Compiler`
- phase status: `verified_with_audit_tail`
- next phase: `Phase F: Memory Atlas`
- next step: `cross-harness wake-packet proof`
- active blocker: `FEATURE-V1-WAKE-PACKET` audit tail still open

## Current Focus

Finish the wake-packet audit tail, then start the memory-atlas slice without
letting roadmap truth drift from live continuity truth.

## Blockers

- `FEATURE-V1-WAKE-PACKET` audit tail is still open
- `.memd/env` generation is not shell-safe
- Claude Code bootstrap wording still overclaims parity

## Status Rules

- `pending`: not started
- `in_progress`: active build work
- `blocked`: cannot move without external fix or decision
- `verified`: engineering verification passed
- `verified_with_audit_tail`: engineering verification passed, follow-up audit still open
- `complete`: human-tested and accepted at the product level

## Product Contract

`memd` is a multiharness second-brain memory substrate for humans and agents.
It must preserve active truth, durable memory, provenance, corrections,
continuity, and recovery across sessions, tools, harnesses, and artifacts
without collapsing back into transcript reconstruction.

Current priority harnesses are Codex, OpenCode, Hermes, and OpenClaw. They are
the proving ground for this contract, not the whole definition of the product.

## Canonical Phases

Use phases for execution order. Detailed phase plans live in linked docs.

| Phase | Name | Status | Purpose | Detail |
| --- | --- | --- | --- | --- |
| A | Raw Truth Spine | `verified` | capture once, keep raw evidence, preserve source linkage | [[phase-a-raw-truth-spine]] |
| B | Session Continuity | `verified` | fresh-session resume without transcript rebuild | [[phase-b-session-continuity]] |
| C | Typed Memory | `verified` | explicit memory kinds instead of one flat store | [[phase-c-typed-memory]] |
| D | Canonical Truth | `verified` | corrections, trust, freshness, conflict handling | [[phase-d-canonical-truth]] |
| E | Wake Packet Compiler | `verified_with_audit_tail` | compile small action-ready memory packets | [[phase-e-wake-packet-compiler]] |
| F | Memory Atlas | `pending` | packet -> region -> evidence navigation | [[phase-f-memory-atlas]] |
| G | Procedural Learning | `pending` | learn reusable operating procedures | [[2026-04-11-memd-ralph-roadmap]] |
| H | Hive Coordination | `pending` | shared second brain across harnesses | [[2026-04-11-memd-ralph-roadmap]] |
| I | Overnight Evolution | `pending` | dream/autodream/autoresearch with trust gates | [[2026-04-11-memd-ralph-roadmap]] |

## Next Up

1. Close `FEATURE-V1-WAKE-PACKET`.
2. Start `Phase F`.
3. Fix shell-unsafe `.memd/env` generation.
4. Fix Claude Code bootstrap wording.
5. Define one rule for phase-state flips.

## Immediate Backlog

1. [[2026-04-12-claude-code-bootstrap-bridge-gap]] — `open`, found `2026-04-12`.
   Roadmap/docs still overstate Claude Code as if it had native memd bootstrap parity.

2. [[2026-04-12-shell-unsafe-memd-env-generation]] — `open`, found `2026-04-12`.
   Generated `.memd/env` is not shell-safe, so helper scripts can die before invoking memd.

3. [[2026-04-12-roadmap-state-audit-tail-drift]] — `open`, found `2026-04-12`.
   Roadmap phase state and live continuity state can disagree during verification/audit tails.

## Recently Closed

- `Phase A` raw truth spine: `verified`
- `Phase B` session continuity: `verified`
- `Phase C` typed memory: `verified`
- `Phase D` canonical truth: `verified`

## Reference Docs

- [[docs/core/setup.md|Setup and harness behavior]]
- [[docs/verification/milestones/MILESTONE-v1.md|Milestone v1 verification]]
- [[docs/strategy/research-loops.md|Research loops]]
- [[docs/superpowers/specs/2026-04-11-memd-ralph-roadmap.md|Detailed Ralph roadmap spec]]
- [[docs/superpowers/specs/2026-04-11-memd-canonical-theory-synthesis.md|Canonical theory synthesis]]

## Non-Goals

- transcript dumping
- vendor lock-in
- using RAG as the only memory layer
- mixing project-local truth with global truth
- letting one provider silently overwrite another provider’s memory
