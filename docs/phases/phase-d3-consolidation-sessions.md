---
phase: D3
name: Consolidation + Sessions
version: v3
status: pending
depends_on: [A3, B3, C3]
notes: Renamed from C3 to D3 on 2026-04-17 so phase IDs match execution order.
backlog_items:
  - "2026-04-14-no-decay-calibration"
  - "2026-04-14-memory-dedup-incomplete"
  - "2026-04-14-no-overnight-evolution-loop"
  - "2026-04-14-no-consolidation-quality-proof"
---

# Phase D3: Consolidation + Sessions

## Goal

Tighten long-tail recall by adding storage-time dedup, calibrating decay against the LongMemEval >7d slice, and consolidating session events into episodes that survive across the working-memory churn. Overlaps M2-evo (M4) on overnight loop infra, but D3 is **bench-gated** while M2-evo is infra-only.

## Why this phase exists

LongMemEval's hardest items are >7d-old facts that should still be recallable. Memd's decay formula is uncalibrated (`docs/backlog/2026-04-14-no-decay-calibration.md`). LoCoMo's hardest items are cross-session ones — same speakers, same topic, different conversation. Mempalace ships verbatim retention as a doctrine and dedup at storage time at 0.15 cosine threshold ([[.memd/lanes/architecture/A2-04-dedup.md]]). Both move the long-tail.

## Deliver

1. **Storage-time dedup at 0.15 cosine threshold** — group incoming items by `(source_path, lane, kind)`, compute pairwise cosine within group, merge near-duplicates (≥ 0.85 similarity). Keep richest survivor; preserve all provenance refs ([[.memd/lanes/architecture/A2-04-dedup.md#mempalace-dedup-algorithm]]).
2. **Episode schema + session boundaries** — `Episode { id, mind, title, narrative, date, session_id }` with `episode_facts(episode_id, fact_id, relation)` junction. FTS5 index on `narrative`. Session boundaries detected from event spine gaps (>30min idle = new session).
3. **Decay calibration vs LongMemEval >7d slice** — run benchmark with current decay constants, then sweep `base_half_life_days` and `reinforcement_factor` to find Pareto-optimal point. Calibrated values land in `MemoryPolicyDecay` config.
4. **Episode consolidation in dream loop** — overnight (or N-hour) job groups last-window events into episodic narratives; idempotent re-runs are no-ops.
5. **`memd dedup --dry-run`** — maintenance command reports candidate merges before applying. Survivor inherits all provenance refs from merged items. Never erase history.

## Pass Gate

Bench-delta required (regenerate [[docs/verification/PUBLIC_LEADERBOARD.md]]):

- pre: LongMemEval=0.95, LoCoMo=0.75, MemBench=0.75 (post-C3 baseline; V3 0.70 floor cleared everywhere except ConvoMem, which E3 owns)
- post intrinsic (sidecar OFF, primary): **LongMemEval ≥ 0.97** (long-tail bump), **LoCoMo ≥ 0.80** (cross-session bump, well above floor)
- LongMemEval >7d-slice metric: **+0.05 minimum** (broken out separately in leaderboard)
- post accelerated (sidecar ON, bonus): ≥ +0.02 over intrinsic per metric
- regression budget: no metric drops > 0.02
- evidence: leaderboard regenerated with decay calibration sweep table

Plus:
- `cargo test -p memd-server` green for episodes + dedup
- Dedup dry-run on dogfood corpus shows non-trivial merge candidates without false positives
- Dream loop consolidation: idempotent (re-run produces zero new episodes for same window)

## Evidence

- Pre/post leaderboard with >7d slice broken out
- Decay calibration sweep table (half_life × reinforcement → score)
- Sample episode (narrative + linked facts) after consolidation
- Dedup dry-run report on dogfood data

## Product Win

- **Episodes read like human narratives.** A dogfooder opening a recent episode sees "we were debugging X, decided Y, shipped Z" in prose — not a mechanical event dump.
- **Working memory stays under ~50 items without manual pruning.** Dedup + episode consolidation do the work; operator intervention is rare.
- **Dedup is explainable.** Every merge names what merged, why (similarity score), and is restorable from the dry-run log. No silent data loss — never erase history.

Evidence:
- Sample episodes from dogfood corpus; hand to a reader outside the project, verify they can follow the narrative
- Working-memory size timeseries across a dogfood week; no manual-prune events
- Dedup report with explicit keep-vs-merge decisions + restorable log

## Fail Conditions

- LongMemEval >7d slice doesn't move — decay still uncalibrated; investigate before shipping
- Dedup false-positive rate > 5% on dogfood — raise threshold above 0.15
- Episode consolidation non-idempotent — diagnose before merge
- LoCoMo regression — episode boundaries breaking cross-session retrieval

## Donor Anchors

- **D3-D1**: mempalace storage-time dedup (group by source, 0.15 cosine, greedy keep-richest) — [[.memd/lanes/architecture/A2-04-dedup.md]]
- **D3-D2**: mempalace temporal freshness signals (rehearsal, last_accessed, verification decay) — [[.memd/lanes/architecture/A2-13-temporal-freshness.md]]
- **D3-D3**: Omegon-style decay with reinforcement extension (M2-evo overlap) — [[docs/theory/2026-04-14-donor-extraction-to-v2-phases.md]]

## Rollback

- `dedup.storage_time=false` disables storage-time dedup
- Decay constants default to current uncalibrated values; calibrated values behind `policy.decay.calibrated=true`
- Episodes table can stay empty; only consolidation writes to it
- Dream loop disabled by default until proven idempotent on dogfood

## Out of scope

- Atlas extraction (C3 already done by this phase)
- Reranker (B3)
- ConvoMem adapter (E3)
- Sidecar wiring (A3)
- M2-evo's broader overnight evolution scope (lives in M4, infra not bench)

## Relationship to M2-evo (M4)

M2-evo ships the overnight evolution **infrastructure** (worker loop, dream module, lifecycle hooks). D3 ships the **bench-gated consolidation behavior** that runs on top of that infra. If M2-evo is incomplete when D3 starts, D3 owns the missing infra pieces it depends on; otherwise D3 just wires consolidation into the existing loop.
