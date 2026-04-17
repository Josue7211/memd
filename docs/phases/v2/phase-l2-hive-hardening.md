---
phase: L2
name: Hive Hardening
version: v2
status: pending
depends_on: [C2, J2]
backlog_items: [53, 67, 70, 71, 75, 81]
---

# Phase L2: Hive Hardening

## Goal

Multi-agent coordination works. Queen ops functional. Cross-harness continuity proven.

## Deliver

- Queen client methods (deny, reroute, handoff)
- Coordination mode enforcement (not just advisory)
- Cross-harness continuity E2E test
- Handoff quality verification
- Admission control / rate limiting

## Pass Gate

- Start work in Codex → continue in Claude Code with full context (E2E test)
- Queen deny blocks conflicting writes
- Rate limit: > 100 writes/minute triggers throttle
- Handoff packet contains all active working context

## Evidence

- Cross-harness E2E test
- Queen enforcement test
- Rate limiting test
- Handoff quality comparison

## Fail Conditions

- Cross-harness continuity loses context
- Queen ops don't enforce
- Rate limiting blocks legitimate writes

## Donor Extraction (from inspiration repos)

- **L2-D1** (Smriti `FreshnessInfo`): Freshness check with commit-based baseline. Client provides `since_commit_id`, server returns `changed: bool`, `new_checkpoints_count`, list of new checkpoints. memd: "what changed since my last wake?"
- **L2-D2** (Smriti `DivergenceSummary`): Multi-branch divergence detection. Normalize text, diff decisions between branches. Surface in hive board: `main_only_decisions` vs `branch_only_decisions`. Cap at 2 branches, 3 decisions per side.
- **L2-D3** (Omegon `sqlite.rs` — **DIRECT RUST LIFT**): SQLITE_BUSY retry. `PRAGMA busy_timeout = 5000`. WAL mode. Handles concurrent access without custom retry. Fix for backlog #75.
- **L2-D4** (Omegon `sqlite.rs` — **DIRECT RUST LIFT**): Lamport versioning for conflict resolution. `version: u64` on every fact, incremented on every mutation. On import: `incoming.version <= stored.version → skip`. Deterministic, no timestamp dependency.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert enforcement if it breaks single-agent workflows
