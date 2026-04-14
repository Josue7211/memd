# Memory Lifecycle Contract

status: draft
ref: backlog #82 (memory-lifecycle-not-auto-managed)
date: 2026-04-14
consumers: B2 (Signal vs Noise), C2 (Ghost Cleanup), F2 (Ingestion Pipeline)

## Purpose

Define the guarantees each phase must uphold so memory items
move through their lifecycle automatically. No manual intervention.
This contract resolves all 5 gaps from backlog #82.

---

## 1. State Machine

### Valid Transitions

```
                 ┌──── verify ────┐
                 │                │
  Candidate ──promote──► Active ──┤──► Stale ──► Expired ──► DELETED
                           │      │       │
                           │      └───────┘  (verify resets to Active)
                           │
                           ├──► Superseded ──► Expired ──► DELETED
                           └──► Contested
```

| From | To | Trigger | Reversible |
|------|-----|---------|-----------|
| Candidate | Active | `promote` (explicit) | no |
| Active | Stale | staleness detector (auto) | yes — via `verify` |
| Active | Superseded | `supersede` repair (explicit) | no |
| Active | Contested | `contest` repair (explicit) | yes — via `prefer_branch` |
| Active | Expired | TTL expiry (auto) or `expire` repair | no |
| Stale | Active | `verify` repair (explicit) | — |
| Stale | Expired | TTL expiry (auto) or staleness exceeds 2× window | no |
| Superseded | Expired | grace period (auto, 7d) | no |
| Expired | DELETED | `drain_expired` reaper (auto) | no |

### Invariants

- Only `Active` and `Stale` items appear in retrieval results.
- `Expired` items never appear in any retrieval path. Period.
- `Candidate` items only appear in candidate-specific queries.
- Transitions record a `memory_event` with timestamp and reason.

---

## 2. TTL Enforcement (Gap #1)

**Dual-path. Both required.**

### Path A — Retrieval-time filter (correctness)

**Owner: B2**

Every retrieval path (`context`, `context/compact`, `working`, `search`,
`wakeup`) MUST filter out items where:

```
ttl_seconds IS NOT NULL
AND created_at + ttl_seconds < now()
```

These items are treated as `Expired` in results regardless of their
stored `status` field. The filter applies BEFORE scoring, not after.

**Guarantee:** An expired-by-TTL item is never returned to any consumer.

### Path B — Background reaper (hygiene)

**Owner: C2**

A maintenance job (called during `memd heartbeat` or `memd maintenance`)
that:

1. Scans items where `ttl_seconds IS NOT NULL AND created_at + ttl_seconds < now()`
2. Sets `status = Expired` on matching items
3. Records a `memory_event` with `event_type = "expired"`, `reason = "ttl"`
4. Calls existing `drain_expired()` to hard-delete items that have been
   `Expired` for longer than the grace period (default: 24h)

**Guarantee:** DB does not accumulate dead-weight rows indefinitely.

### Default TTL by Kind

Items ingested without an explicit `ttl_seconds` inherit the kind default:

| Kind | Default TTL | Rationale |
|------|------------|-----------|
| Status | 48h | Ephemeral by nature |
| LiveTruth | none (managed by refresh) | Freshness, not expiry |
| All others | none | Persist until superseded or stale-expired |

Phases SHOULD NOT invent new defaults beyond this table.

---

## 3. Staleness Detection (Gap #2)

**Owner: B2**

### Freshness Windows by Kind

| Kind | Freshness Window | Double-window (auto-expire) |
|------|-----------------|---------------------------|
| LiveTruth | 1 day | 2 days |
| Status | 2 days | 4 days |
| Pattern | 7 days | 14 days |
| Fact | 14 days | 28 days |
| Decision | 14 days | 28 days |
| Procedural | 14 days | 28 days |
| Preference | 30 days | 60 days |
| Constraint | 30 days | 60 days |
| Runbook | 30 days | 60 days |
| SelfModel | 30 days | 60 days |
| Topology | 30 days | 60 days |

### Formula

```
age = now() - coalesce(last_verified_at, updated_at)

if age > freshness_window AND status == Active:
    status ← Stale
    record event(type="status_change", reason="staleness_auto")

if age > 2 × freshness_window AND status == Stale:
    status ← Expired
    record event(type="expired", reason="staleness_double_window")
```

**Grace period for double-window expirations:** Items expired via
double-window MUST use a 7-day `drain_expired()` grace period (not the
default 24h). A user who doesn't touch memd for 2 months should find
their Preferences in Expired state (recoverable via verify) rather than
hard-deleted. The 7-day window allows re-verification after returning.

### Reversal

`MemoryRepairMode::Verify` already sets `last_verified_at = Utc::now()`
and resets status to Active (repair/mod.rs:74-76). This is the reversal
mechanism. No new code needed for reversal — only for detection.

### Execution

Runs as part of the heartbeat cycle or a dedicated `memd maintenance staleness` pass.
Batch size: configurable, default 256 items per cycle.

---

## 4. Live Truth Refresh (Gap #3)

**Owner: B2**

### Contract

Items with `kind = LiveTruth` represent facts that change during a session.
The system MUST detect changes and re-ingest within one heartbeat cycle (30s).

### Requirements

1. `capture-live` MUST set `kind = LiveTruth` on created items (currently missing).
2. Heartbeat MUST include a `refresh_live_truth()` step that:
   - Checks live truth sources (`.memd/lanes/`, runtime state) for changes
   - Re-ingests changed items via the existing store pipeline
   - Uses content-hash dedup to avoid duplicates (reinforce, don't insert)
3. LiveTruth items not refreshed within their freshness window (1d) transition
   to Stale via the staleness detector (section 3).

### What "live" means

"Live" = the system automatically detects changes and updates the item.
"One-shot" = the current behavior where `capture-live` runs once and never again.
This contract requires "live", not "one-shot".

---

## 5. Research Rank Boost (Gap #4)

**Owner: F2**

### Problem

`durable_truth_rank_adjustment()` (helpers.rs:707-732) gives +0.0 to research
items. Corrections get +1.4 to +3.4. Research items score ~2.1 vs corrections
at ~3.5 and never surface in wake.

### Contract

Add to `durable_truth_rank_adjustment()`:

| Condition | Boost | Stacks |
|-----------|-------|--------|
| `tag == "research"` | +0.6 | yes |
| `source_system == "lane-ingest"` | +0.4 | yes |

Cap: total `durable_truth_rank_adjustment()` return value MUST NOT exceed 2.0
for non-correction items. Corrections retain their existing boosts uncapped
(they represent user corrections and must always rank highest).

### Scoring outcome (approximate, same-project canonical Fact)

After this change:
- Correction items: highest rank (unchanged)
- Research items: boosted ~1.0 above baseline (now visible in top slots)
- Untagged items: baseline (unchanged)

Exact scores depend on kind, project match, age, and other scoring factors.
These are directional, not spec targets.

---

## 6. Wake Durable Truth Budget (Gap #5)

**Owner: B2**

### Current limits (wakeup.rs:134-141)

| Mode | Current | New |
|------|---------|-----|
| `claude_strict` | 1 | 1 (unchanged — token budget constraint) |
| default | 2 | 4 |
| verbose | 4 | 6 |

### Overflow hint

When available items exceed the display limit, append a hint line:

```
- + N more via `memd lookup`
```

The overflow hint appears in **all modes including `claude_strict`**.
It is one short line with negligible token cost, and without it the
agent in strict mode has zero signal that research items exist.

### Char budget per item

| Mode | Current | New |
|------|---------|-----|
| `claude_strict` | 120 | 120 (unchanged) |
| default | 160 | 140 (tighter — more items, less space each) |
| verbose | 160 | 160 (unchanged) |

---

## Phase Ownership Map

| Contract Section | Gap # | Owner | Prerequisite |
|-----------------|-------|-------|-------------|
| 2A. Retrieval-time TTL filter | 1 | B2 | none |
| 2B. Background TTL reaper | 1 | C2 | B2 (retrieval filter exists) |
| 3. Staleness detection | 2 | B2 | none |
| 4. Live truth refresh | 3 | B2 | none |
| 5. Research rank boost | 4 | F2 | none |
| 6. Wake budget expansion | 5 | B2 | none |

### Dependency chain

```
B2 delivers: retrieval TTL filter, staleness detector, live truth refresh, wake budget
    ↓
C2 delivers: background reaper, drain pipeline, ghost ref cleanup
F2 delivers: research rank boost, ingestion pipeline
```

C2 depends on B2 (retrieval filter must exist before reaper matters).
F2 depends on B2 (wake budget must be expanded before rank boost is visible).

---

## Verification

Each phase MUST prove its contract sections hold:

| Section | Test |
|---------|------|
| 2A | Store item with `ttl_seconds=1`, wait 2s, query → not returned |
| 2B | Store expired item, run reaper, verify row deleted |
| 3 | Store fact, advance clock 15d, run staleness pass → status=Stale |
| 3 (reversal) | Verify stale item → status=Active, `last_verified_at` updated |
| 3 (double) | Store fact, advance clock 29d without verify → status=Expired |
| 4 | Modify live source, wait one heartbeat cycle → item updated in DB |
| 5 | Store research item, query durable truth → appears in top 4 |
| 6 | Store 10 items, wake default → shows 4 + overflow hint |

---

## Out of Scope

- Implementation details (SQL queries, function signatures, thread model)
- Decay calibration (#62) — separate concern, existing exponential decay
- Entity lifecycle — entities have their own decay in `decay_entities()`
- Hive session lifecycle — already has auto-retirement
