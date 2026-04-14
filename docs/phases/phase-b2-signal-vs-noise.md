---
phase: B2
name: Signal vs Noise
version: v2
status: verified
depends_on: [A2]
backlog_items: [42, 49, 66, 77]
branch: research/mining
---

# Phase B2: Signal vs Noise

Current status: `verified` — all 6 tasks committed, pass gate verified 2026-04-14. eval=95, 0 status items, 100% noise reduction, E2E fact recall confirmed.

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
