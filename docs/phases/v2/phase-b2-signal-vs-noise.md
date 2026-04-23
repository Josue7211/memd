---
phase: B2
name: Signal vs Noise
version: v2
status: reopened
depends_on: [A2]
backlog_items: [42, 49, 66, 77, 82]
branch: research/mining
reopened_at: 2026-04-15
reopened_reason: Gate passed on synthetic test but real usage proves stale records never expire, preferences lost every session, working memory holds completed-phase noise weeks later.
---

# Phase B2: Signal vs Noise

Current status: `reopened` — original gate passed 2026-04-14 on synthetic fact, but real production use shows working memory still dominated by stale completed-phase records (B2 status items visible weeks after verification). Preferences and architecture decisions never surface in wake. Pipeline lifecycle (expire/promote) doesn't run.

## Reopened Scope

Original scope fixed checkpoint dedup and status caps. New scope must also fix:
- **Working memory expiry**: completed-phase records must auto-expire on phase flip
- **Preference persistence**: architecture decisions and user corrections stored and surfaced
- **Pipeline lifecycle**: promote/expire/archive must actually execute in production
- **Wake surfacing**: facts/decisions/preferences must appear in wake, not just pass a synthetic test

## Node Verification (from [[docs/verification/NODE-VERIFICATION-MATRIX.md]])

This phase owns M1-tier verification for:
- P1 (working context compiler): holds current-phase data, stale records expire
- M1 (working context): budget enforced, admission/eviction works
- S1 (wake packet): surfaces facts/decisions/preferences, not just status

## Goal

Facts, decisions, and procedures surface in wake packets. Status noise eliminated.

## Deliver

- Checkpoint dedup via redundancy_key
- Status cap enforced in working memory admission
- Wake kind scoring reweighted: facts/decisions outrank status
- Live memory contract: new captures appear in working memory within one cycle

## Pass Gate

- `memd eval` score ≥ 65 (from current ~35)
- Store a fact → next wake packet contains that fact (not buried under status)
- Working memory has ≤ 2 status items out of 8 slots
- Status noise reduction ≥ 80% measured by before/after item count

## Evidence

- E2E test: store fact, run wake, assert fact in packet
- Before/after working memory composition snapshots
- `memd eval` score regression test in CI

## Fail Conditions

- Facts still not surfacing after fix
- Status items still dominate working memory

## Donor Extraction (from inspiration repos)

- **B2-D1** (supermemory): Priority-based retrieval dedup — static > dynamic > search. Exact string match dedup at wake assembly time. Canonical facts always survive.
- **B2-D2** (mempalace `layers.py`): Hard cap on items per type in context. L1 essential story caps at 15 drawers, 3200 chars total. memd: cap Status at 2, total at 8.
- **B2-D3** (Omegon `sqlite.rs` — **DIRECT RUST LIFT**): Content hash dedup on checkpoint writes. `SHA256(normalize(content))[0..16]`. If hash exists, reinforce (increment count, bump version) instead of inserting duplicate.
- **B2-D4** (Omegon `decay.rs` — **DIRECT RUST LIFT**): Exponential decay with reinforcement-extended half-life. `confidence = e^(-ln(2) × days / halfLife)`. Profiles: Standard (14d/1.8x), Global (30d/2.5x), RecentWork (2d/1.0x). Map Status→RecentWork, Fact→Standard.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert scoring changes if eval score drops below pre-fix baseline
