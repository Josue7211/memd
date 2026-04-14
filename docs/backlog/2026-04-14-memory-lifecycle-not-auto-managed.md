# Memory Lifecycle Not Auto-Managed

status: open
severity: critical
phase: V2 — needs own phase or fold into B2+C2+M2-evo
opened: 2026-04-14

## Problem

memd's memory lifecycle is entirely manual. Items are created, but nothing
automatically updates, expires, promotes, or demotes them. The `ttl_seconds`
field is stored but never enforced. `drain_expired()` only deletes items
already marked Expired — nothing marks them. Live truth doesn't update live.
Research items ingested into the DB never surface in wake's durable truth
tier because the ranking system doesn't reward them.

This is a compound failure across 5 subsystems.

## The 5 Gaps

### 1. TTL stored, never enforced

- `ttl_seconds` is a field on `MemoryItem` (schema lib.rs:237)
- `drain_expired()` (store.rs:1550) only deletes `status=Expired` rows
- Nothing checks `created_at + ttl_seconds < now()` to mark items expired
- Hive claims DO have auto-expiry (store_hive_lifecycle.rs:11-19) — memory items don't
- No background reaper, no retrieval-time check

### 2. No staleness detection

- Items never auto-transition to `Stale` status
- `last_verified_at` is stored but nothing uses it to compute staleness
- `context_score()` penalizes Stale items (-1.5) but nothing ever sets that status
- Result: dead items sit forever at full confidence

### 3. Live truth doesn't update live

- `capture-live` is a one-shot shell script (profile_runtime.rs:261-273)
- Calls `memd hook capture --tag live-capture` — no daemon, no file watcher
- Heartbeat (profile_runtime.rs:73) runs every 30s via `memd heartbeat --watch`
  but doesn't pull/refresh live truth items
- Items created via `capture-live` don't even set `kind=LiveTruth` —
  the hook doesn't translate tags into kinds

### 4. Research items don't rank into durable truth

- `durable_truth_rank_adjustment()` (helpers.rs:707-732) only rewards:
  - `tag == "correction"`: +1.4
  - `tag == "project_fact"`: +1.0
  - `source_system == "correction"`: +0.8
  - Content prefix `"Corrected fact:"`: +1.2
  - Content prefix `"Remembered project fact:"`: +0.8
- Ingested research items (`confidence=0.85`, tags `research`, `lane:*`) get +0.0
- They score ~2.1 vs corrections at ~3.5 — always ranked below
- `context_compact` returns them (position 3+) but wake only shows top 1-2

### 5. Wake durable truth section is too narrow

- wakeup.rs:134-141: `claude_strict` limits to 1 item, default to 2, verbose to 4
- With 600+ items in DB, 1-2 slots means research never surfaces at boot
- Context compact returns 8 items — wake discards 6-7 of them

## Dog-Food Proof

This session: 39 research items about the memd codebase were ingested into the
DB. In the same session, the agent dispatched sub-agents to re-read the same
files instead of using `memd lookup`. The ingested knowledge was invisible
because wake didn't surface it and the agent didn't reflexively lookup.

This is the exact failure pattern the backlog item
`2026-04-14-research-not-stored-as-shared-memory.md` was supposed to fix.
The storage fix worked. The surfacing and lifecycle fix did not.

## What Needs to Exist

1. **TTL reaper** — background job or retrieval-time check that marks items
   expired when `created_at + ttl_seconds < now()`. Run in worker cycle
   alongside existing `drain_expired()`.

2. **Staleness detector** — items not verified within N days of their kind's
   expected freshness window get auto-marked Stale. Use kind-based profiles
   (Status: 2d, Fact: 14d, LiveTruth: 1d).

3. **Live truth daemon** — either a file watcher on `.memd/lanes/` or a
   polling loop in the heartbeat that detects changes and re-ingests.
   `capture-live` should be continuous, not one-shot.

4. **Research rank boost** — `durable_truth_rank_adjustment()` should reward
   `tag == "research"` and `source_system == "lane-ingest"` items. Not as
   high as corrections (+1.4) but above default (+0.0). Suggest +0.6.

5. **Wake durable truth expansion** — wake should show more than 1-2 items,
   OR should show a summary line like "39 research items available via
   `memd lookup --tag research`" so the agent knows to check.

## Phase Mapping

| Gap | Natural Owner | Milestone |
| --- | --- | --- |
| TTL reaper | C2 Ghost Cleanup (#50) | M1 |
| Staleness detector | B2 Signal vs Noise (#66, #77) | M1 |
| Live truth daemon | B2 Signal vs Noise (#66) | M1 |
| Research rank boost | F2 Ingestion Pipeline (#37) | M1 |
| Wake expansion | B2 Signal vs Noise | M1 |

All 5 gaps map to M1 phases. This suggests M1 is the right place — not a
new phase. But the lifecycle concern cross-cuts B2, C2, and F2. Consider
a pre-M1 teardown that defines the lifecycle contract before implementing.

## Related Backlog

- #37 `no-source-ingestion-pipeline` — no compile step for lane docs
- #41 `no-change-detection-on-source-material` — no mtime/hash tracking
- #50 `ttl-enforcement-no-gc` — TTL never enforced
- #62 `no-decay-calibration` — decay hardcoded
- #66 `no-live-memory-contract` — live refresh unenforced
- #77 `stale-working-memory-cache` — corrections silently ignored
