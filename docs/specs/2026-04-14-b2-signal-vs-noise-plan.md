# B2: Signal vs Noise — Implementation Plan

status: ready
phase: B2
depends_on: A2 (verified)
lifecycle_contract: docs/specs/2026-04-14-memory-lifecycle-contract.md
branch: feature/b2-signal-vs-noise

## Goal

Facts, decisions, and procedures surface in wake packets. Status noise eliminated.

## Delivery Order

Six tasks, ordered by dependency. Each is independently testable and committable.

---

### Task 1: Retrieval-time TTL filter (lifecycle contract §2A)

**What:** Every retrieval path filters out TTL-expired items before scoring.

**Where:**
- `crates/memd-server/src/main.rs:303-316` — `snapshot()` calls `apply_lifecycle()` on each item. Add TTL expiry check here: if `ttl_seconds IS NOT NULL AND created_at + ttl_seconds < now()`, set `status = Expired` during the lifecycle pass.
- This is the single chokepoint — all retrieval paths (`context`, `search`, `working`, `inbox`) flow through `snapshot()`.

**Why here, not in each route:**
- `snapshot()` already runs `apply_lifecycle()` on every item. Adding TTL check to the same pass means zero per-route changes. One filter, all paths covered.

**Already works:**
- `context_score()` applies -1.5 for Stale, -2.0 for Contested (helpers.rs:588-593)
- `search_score()` applies -4.0 for Expired (helpers.rs:600+)
- Once status is set to Expired in snapshot, existing scoring already suppresses it
- `filter_items()` should also hard-exclude Expired from search results (helpers.rs:160) — add default Expired exclusion if no status filter specified

**Test:** Store item with `ttl_seconds=1`, wait 2s, query via `/memory/context` → not returned.

---

### Task 2: Staleness detector (lifecycle contract §3)

**What:** Background pass marks items Stale when `age > freshness_window`.

**Where:**
- New function: `apply_staleness()` in `crates/memd-server/src/helpers.rs` or a new `lifecycle.rs` module
- Called from: `snapshot()` in `main.rs:303-316` (same lifecycle pass as Task 1)
- OR: separate maintenance endpoint `POST /memory/maintenance/staleness` called by heartbeat

**Design choice — snapshot vs maintenance endpoint:**
- Snapshot pass: items go stale on read (lazy). Simple, no background job. But only triggers when someone queries.
- Maintenance endpoint: items go stale on schedule (eager). Requires heartbeat integration. But DB stays clean even when no one queries.
- **Recommendation: both.** Snapshot does lazy marking (correctness). Maintenance endpoint does eager batch marking (hygiene). Same as dual-path TTL.

**Freshness windows:** Use table from lifecycle contract §3 (LiveTruth: 1d, Status: 2d, Pattern: 7d, Fact/Decision/Procedural: 14d, Preference/Constraint/Runbook/SelfModel/Topology: 30d).

**Formula:** `age = now() - coalesce(last_verified_at, updated_at)`. If `age > window && status == Active` → Stale. If `age > 2× window && status == Stale` → Expired (with 7d drain grace).

**Reversal:** Already works — `MemoryRepairMode::Verify` sets `last_verified_at = Utc::now()` and resets to Active (repair/mod.rs:74-76). No new code for reversal.

**Test:** Store Fact, advance clock 15d, run staleness pass → status=Stale. Verify → status=Active. Advance 29d without verify → status=Expired.

---

### Task 3: Checkpoint dedup via content hash (donor B2-D3)

**What:** On checkpoint writes, compute content hash. If hash matches existing item, reinforce instead of inserting duplicate.

**Where:**
- `crates/memd-server/src/keys/mod.rs:39-61` — `redundancy_key()` already tokenizes content + metadata
- `crates/memd-server/src/store.rs:409-459` — `insert_or_get_duplicate()` already handles UNIQUE constraint violations
- `crates/memd-client/src/runtime/checkpoint.rs:171-191` — `checkpoint_with_bundle_defaults()` is the client entry point

**What exists vs what's missing:**
- `redundancy_key` computation exists and works ✓
- Duplicate detection on insert exists (UNIQUE constraint on redundancy_key_stage index) ✓
- **Missing:** When duplicate detected, reinforce existing item instead of silently returning the old one. Currently `store_item()` returns the existing duplicate but doesn't bump its `updated_at`, confidence, or record a "reinforced" event.

**Donor pattern (Omegon):** On duplicate, increment rehearsal count, bump updated_at, bump version. memd equivalent: bump `updated_at`, optionally increase confidence (cap 1.0), record `memory_event(type="reinforced")`.

**Test:** Store same content twice via checkpoint → only 1 item in DB, `updated_at` bumped, "reinforced" event recorded.

---

### Task 4: Status cap in working memory (donor B2-D2)

**What:** Cap Status items at 2 in working memory admission. Already partially done.

**Where:**
- `crates/memd-server/src/working/mod.rs:68-82` — `working_memory()` already has `max_status_items = 2` hardcoded

**What exists vs what's missing:**
- Status cap = 2 exists ✓
- Total cap at admission exists (via `build_context()` limit) ✓
- **Missing:** Eviction tracking — when a status item is evicted, record why. Currently items are silently skipped. Add eviction reason to response so agents know items were dropped, not missing.
- **Missing:** Character budget per item. Donor says 220 chars. Currently no per-item char budget in working memory (only in wake rendering). Consider adding to `working_memory()` response.

**Test:** Store 5 Status items + 3 Facts, query working memory → ≤ 2 Status items, all 3 Facts present.

---

### Task 5: Wake durable truth budget expansion (lifecycle contract §6)

**What:** Increase default durable truth slots from 2→4, verbose from 4→6. Add overflow hint in all modes.

**Where:**
- `crates/memd-client/src/runtime/resume/wakeup.rs:134-141` — the limit block
- After line 148 (end of durable truth rendering loop) — add overflow hint

**Changes:**
```
claude_strict: 1 → 1 (unchanged)
default:       2 → 4
verbose:       4 → 6
```

**Overflow hint:** After the durable truth items, if `snapshot.context.records.len() > limit`:
```
- + N more via `memd lookup`
```
Appears in ALL modes including claude_strict.

**Char budget adjustment:**
```
claude_strict: 120 → 120 (unchanged)
default:       160 → 140 (more items, tighter each)
verbose:       160 → 160 (unchanged)
```

**Test:** Store 10 items, wake in default mode → shows 4 items + "6 more via memd lookup" hint.

---

### Task 6: Live truth refresh in heartbeat (lifecycle contract §4)

**What:** Heartbeat cycle detects changes in live truth sources and re-ingests.

**Where:**
- `crates/memd-client/src/bundle/maintenance_runtime/mod.rs:704-723` — `write_bundle_heartbeat()` is the heartbeat entry point
- New function: `refresh_live_truth()` called from heartbeat before publishing

**Design:**
1. `capture-live` MUST set `kind = LiveTruth` on created items (currently doesn't — tags don't translate to kinds)
2. Heartbeat adds a `refresh_live_truth()` step:
   - Check `.memd/lanes/` for file changes (mtime or content hash)
   - For changed files, re-ingest via store pipeline with `kind = LiveTruth`
   - Content-hash dedup (Task 3) prevents duplicates — reinforce instead of insert
3. LiveTruth items not refreshed within 1d → Stale via staleness detector (Task 2)

**Dependency:** Depends on Task 2 (staleness) and Task 3 (dedup) being done first.

**Test:** Modify a file in `.memd/lanes/`, wait one heartbeat cycle (30s), verify item updated in DB.

---

## Execution Order & Dependencies

```
Task 1 (TTL filter) ── no deps, do first
    ↓
Task 2 (Staleness) ── builds on same lifecycle pass
    ↓
Task 3 (Checkpoint dedup) ── independent but staleness uses reinforcement pattern
    ↓
Task 4 (Status cap) ── independent, small
    ↓
Task 5 (Wake budget) ── independent, small
    ↓
Task 6 (Live truth) ── depends on Tasks 2 + 3
```

Tasks 3, 4, 5 are independent of each other. Can parallelize after Task 2.

## Pass Gate (from phase doc)

- [ ] `memd eval` score ≥ 65 (from ~35)
- [ ] Store a fact → next wake contains that fact (not buried under status)
- [ ] Working memory has ≤ 2 status items out of 8 slots
- [ ] Status noise reduction ≥ 80% by before/after item count

## Evidence Required

- E2E test: store fact, run wake, assert fact in packet
- Before/after working memory composition snapshots
- `memd eval` score regression test
- TTL expiry test (store + wait + query)
- Staleness lifecycle test (store + age + verify cycle)
- Checkpoint dedup test (store twice, assert reinforcement)

## Rollback

- Revert scoring changes if eval score drops below pre-fix baseline
- Each task is independently revertable (separate commits)
