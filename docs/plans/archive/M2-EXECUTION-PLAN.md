---
status: superseded
milestone: M2
superseded_by: ROADMAP.md + individual phase docs
superseded_date: 2026-04-17
---

# M2 Execution Plan: Make It Correct

> This plan tells the next code session exactly what to do. No theory reading needed.
> No "figure out the architecture" needed. Just follow the steps.

## Context

M1 is verified (eval 95). The live loop works: store→wake surfaces facts, ghosts cleaned,
status capped at 2/8. M2 fixes **architectural correctness** — the system stores and retrieves,
but doesn't correct, navigate, route by lane, or prove recall stability.

Four phases. 10-STAR gaps 10–17. 618 existing tests at session start.

## Root Cause

Agents can't self-correct because:
1. Contradiction detection uses `redundancy_key` (sorted content tokens) — two items with opposite
   claims about the same topic produce DIFFERENT keys. Detection never fires for real contradictions.
2. Trust hierarchy is statistical aggregates (`source_trust_score`), not human > canonical > promoted > candidate.
3. Correction items get no scoring boost — `preferred: false` always, no tag-based bonus for `correction` tag.
4. Entity auto-linking runs at ingest but `entity_links` table is empty in production — `auto_link_entity`
   gates on `salience_score > 0.1`, which new entities never reach.
5. Lane detection fires at ingest (`detect_content_lane` in `store_item`) but existing items from M0/M1
   were stored before lane detection existed — they have `lane: NULL`.
6. FTS5 + RRF already landed (commit `d6e3402`) but no E2E proof that corrections change future recall
   or that lanes improve retrieval relevance.

## Dependency Graph

```
M1 (verified) ──┬── D2 (Correction Flow)  ──┐
                 ├── G2 (Lane Architecture)  ──┼── H2 (Recall Proof)
                 └── E2 (Atlas Activation)     ┘
```

D2, G2, E2 can start in parallel — all depend only on M1.
H2 depends on D2 + G2 (two independent proof tracks: correction retention, lane relevance).

## Execution Order

Steps 1–3 (D2, G2, E2) can execute in any order — all depend only on M1.
Step 4 (H2) blocks on D2 + G2. Recommended serial order for a single-developer session:
D2 first (H2 blocks on it and contradiction detection informs E2 entity work), then G2, E2, H2.

### Step 1: D2 — Correction Flow

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-d2-correction-flow.md]]
**Depends on**: M1 (verified)
**Fixes**: gaps 10, 15, 16

**What already exists** (verify, don't rebuild):

- `correct_item()` at `crates/memd-server/src/repair/mod.rs:64` — marks old Superseded,
  creates new Active with `supersedes` vec and `correction` tag. Works.
- `build_context()` at `crates/memd-server/src/helpers.rs:27` — filters `status == Active` only.
  Superseded items already excluded from recall. Works.
- `explain_memory()` at `crates/memd-server/src/inspection/mod.rs:12` — shows lifecycle events
  including `superseded_by_correction` and `correction_created`. Works.
- 6 existing correction tests at `crates/memd-server/src/tests/mod.rs:4310–4589`.
- `working_item_priority` at `crates/memd-server/src/working/mod.rs:332` — Superseded gets
  `status_score: -0.20` and `contradiction_score: -0.12`. Already penalized. Works.

**What to build**:

1. **Entity-based contradiction detection** — replace redundancy_key matching at
   `repair/mod.rs:146–181`. Current code: checks `redundancy_key` match → never fires for
   real contradictions (different content = different sorted tokens = different key).
   Fix: after creating the correction item, get the entity for the new item via
   `store.entity_for_item(new_item.id)`. Then find all OTHER items linked to that same
   entity — **no `items_for_entity` method exists yet; add one to the store** that queries
   `SELECT mi.* FROM memory_items mi JOIN memory_entities me ON me.item_id = mi.id
   WHERE me.entity_id = ?`. Filter those to: same kind, same scope, same project,
   `status == Active`, different content. Mark matches Contested.
   This is the only structural way to detect "two items claiming different things
   about the same topic."
   - Donor: D2-D2 (temporal fact invalidation)
   - File: `crates/memd-server/src/repair/mod.rs:146–181`
   - File: `crates/memd-server/src/store.rs` (add `items_for_entity` query method)

2. **Correction tag scoring boost** — items with `correction` tag should rank higher in
   working memory admission. Add a `correction_boost` to `working_item_priority`:
   `if item.tags.contains("correction") { +0.10 }`. This ensures corrected facts
   outrank uncorrected versions even when other scores are tied.
   - Donor: D2-D1 (Lamport versioning — version ordering → correction wins recency)
   - File: `crates/memd-server/src/working/mod.rs:332` (add after `lane_score` calculation)

3. **Set `preferred: true` on correction items** — `correct_item` creates new items with
   `preferred: false` (hardcoded in `store_item`). The repair path at `repair/mod.rs:237–283`
   already handles `preferred` for belief branches. Correction items should also get
   `preferred: true` so they outrank the original in retrieval. Set it on the new item
   immediately after `store_item` returns.
   - File: `crates/memd-server/src/repair/mod.rs:129–131` (after `store_item` call)

4. **Trust hierarchy enforcement** — `source_trust_score` at `store_entities.rs:251` uses
   statistical ratios. Add a `trust_rank` helper that returns a hard ordering:
   `human_correction > canonical > promoted > candidate > synthetic`.
   Wire into `working_item_priority` as a tiebreaker: when two items score within 0.05,
   higher trust_rank wins.
   - Donor: D2-D4 (immutable checkpoints — authority chain)
   - File: `crates/memd-server/src/store_entities.rs` (new function)
   - File: `crates/memd-server/src/working/mod.rs:332` (consume trust_rank)

5. **E2E correction test** — store a fact, correct it, verify:
   (a) old item is Superseded, (b) new item is Active with `correction` tag and `preferred: true`,
   (c) `build_context` returns corrected version only,
   (d) `explain_memory` shows correction chain,
   (e) working memory admission scores corrected item higher than original.
   - File: `crates/memd-server/src/tests/mod.rs` (new test)

**Pass gate**:
- E2E: store → correct → recall returns corrected version only
- `explain_memory` shows correction history (original + corrected + lifecycle events)
- Contradiction detection fires when two items claim different things about same entity
- Corrected item scores higher than uncorrected in working memory
- Selective reset: correct one item, others unaffected

**Verify nodes**: P4 (corrections change future recall, trust hierarchy enforced),
I1 (corrections routed to P4), M4 (corrections update semantic fast)

---

### Step 2: G2 — Lane Architecture

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-g2-lane-architecture.md]]
**Depends on**: M1 (verified)
**Fixes**: gap 17

**What already exists** (verify, don't rebuild):

- `detect_content_lane()` at `crates/memd-server/src/helpers.rs:335` — 3-step detection:
  explicit `lane:X` tag → source path component → content keyword scan. Covers 6 lanes:
  architecture, decisions, constraints, patterns, design, operations.
- `store_item()` at `crates/memd-server/src/main.rs:99` — calls `detect_content_lane` and
  sets `lane` column on every new item.
- `lane_score` at `crates/memd-server/src/working/mod.rs:430` — `+0.06` for items with lane,
  `0.0` for items without. Already boosts lane-tagged items.
- Lane column exists in DB schema.
- Lane source material exists in `.memd/lanes/` (seeded during F2).
- 10 lane detection tests at `crates/memd-server/src/tests/mod.rs:4827–4871`.

**What to build**:

1. **Backfill lanes on existing items** — items stored before lane detection existed have
   `lane: NULL`. Add a `backfill_lanes` maintenance task that scans all items with `lane IS NULL`,
   runs `detect_content_lane(content, source_path, tags)` on each, and updates the lane column.
   Wire into `memd maintain --mode full`.
   - File: `crates/memd-server/src/main.rs` or `repair/mod.rs` (new function)
   - SQL: `SELECT id, content, source_path, tags FROM memory_items WHERE lane IS NULL`

2. **Lane-aware retrieval routing** — `build_context` at `helpers.rs:27` returns items filtered
   by status but not by lane. Add optional lane filter to retrieval:
   when the query context has a detectable lane (run `detect_content_lane` on the query text),
   boost items from that lane in scoring. This is NOT hard filtering — it's a relevance boost.
   Add `query_lane_boost` to `working_item_priority`: `+0.08` when item lane matches query lane.
   - File: `crates/memd-server/src/working/mod.rs` (add query_lane parameter, boost)
   - Donor: G2-D1 (Omegon lane-indexed retrieval)

3. **Lane tag in wake packet** — wake packet compiler should include the lane tag in each item's
   metadata so consuming agents know which lane the item came from. Verify this is already
   surfaced; if not, add it.
   - File: `crates/memd-client/src/runtime/resume/wakeup.rs` (`render_bundle_wakeup_markdown`)

4. **Lane-gated admission** — extend the B2 kind-based quota in working memory with a lane
   diversity hint: if all 8 slots are from the same lane, prefer admitting items from
   underrepresented lanes (soft penalty, not hard cap). Adds `lane_diversity_penalty` when
   the item's lane already holds >= 5 of 8 slots.
   - File: `crates/memd-server/src/working/mod.rs` (admission function)

**Pass gate**:
- Schema has `lane` column (already exists — verify)
- All 6 lanes have source material (already seeded — verify items in DB)
- Auto-detection fires on new stores (`memd remember "system architecture uses event sourcing"` → lane = architecture)
- Lane-tagged items rank higher than untagged in same-query retrieval
- Existing NULL-lane items backfilled after `memd maintain --mode full`
- New memory gets lane tag automatically without explicit tag

**Verify nodes**: P1 (lane-aware admission), P3 (lane-based routing — DB tags not grep),
S1 (lane-relevant items included), M4 (lanes tag correctly)

---

### Step 3: E2 — Atlas Activation

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-e2-atlas-activation.md]]
**Depends on**: M1 (verified)
**Fixes**: gaps 13, 14

**What already exists** (verify, don't rebuild):

- `auto_link_entity()` at `crates/memd-server/src/main.rs:255` — runs during `store_item`,
  links entities with same project. Gates on `salience_score > 0.1`.
- `create_wiki_links()` at `crates/memd-server/src/main.rs:300` — parses `[[entity]]` in content,
  creates entity links to matching aliases.
- Atlas routes at `crates/memd-server/src/routes.rs:1662+`:
  `get_atlas_regions`, `post_atlas_explore`, `post_atlas_trail_save`, `get_atlas_trails`,
  `post_atlas_rename`, `post_atlas_expand`, `post_atlas_generate`.
- Entity table exists. Entity link table exists.
- Dashboard has atlas graph component (from prior work).

**Root cause: entity_links table empty in production**:
- `auto_link_entity` filters `salience_score > 0.1` — new entities start at 0 salience
  (or very low). By the time the second entity arrives, the first still has low salience.
  Neither passes the filter. No links ever form.

**What to build**:

1. **Fix salience threshold for auto-linking** — lower the threshold from `> 0.1` to `> 0.0`
   OR remove the gate entirely (entity links should form on co-occurrence, not salience).
   Any entity with items should be linkable. Test: store 3 items in same project →
   entities auto-linked.
   - File: `crates/memd-server/src/main.rs:270` (change `0.1` to `0.0` or remove filter)

2. **Backfill entity links for existing items** — run a one-time relink pass: for each
   entity, re-run `auto_link_entity` logic against all other entities in the same project.
   Wire into `memd maintain --mode full`.
   - File: `crates/memd-server/src/main.rs` or new maintenance function

3. **Atlas region generation** — `post_atlas_generate` exists but may not fire automatically.
   Verify: after storing 10+ items, call `atlas_generate` → regions should be non-empty.
   If generation doesn't cluster well, this is a tuning pass, not a rebuild.
   - File: `crates/memd-server/src/routes.rs:1722`

4. **Wake packet atlas hints** — wake packet should include a "what's in the atlas" hint
   so agents know navigation is available. Add a brief atlas summary (region count,
   entity count) to the wake packet metadata.
   - File: `crates/memd-client/src/runtime/resume/wakeup.rs` (`render_bundle_wakeup_markdown`)

5. **Navigation test: wake → evidence in ≤4 hops** — E2E test:
   (a) store 5+ items in same project,
   (b) verify entities auto-created with links,
   (c) call `atlas_explore` → non-empty regions,
   (d) pick a region → expand → items,
   (e) pick an item → `explain_memory` → provenance with sources.
   That's 4 hops: wake → explore → expand → explain.
   - File: `crates/memd-server/src/tests/mod.rs` (new test)

**Pass gate**:
- `memd explore` returns non-empty regions
- Entity links populated in production DB (not just tests)
- Wake packet includes atlas hints
- Wiki link `[[entity]]` creates entity link
- Navigate: wake → region → node → evidence in ≤4 hops

**Verify nodes**: S2 (navigable atlas with backlinks), M3 (timeline navigable via atlas),
S4 (source linkage from canonical to raw)

---

### Step 4: H2 — Recall Proof

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-h2-recall-proof.md]]
**Depends on**: D2, G2
**Fixes**: gaps 11, 12

**What already exists** (verify, don't rebuild):

- FTS5 + RRF landed in commit `d6e3402`: `fts_memory_items` virtual table, RRF scoring
  for hybrid semantic+keyword retrieval.
- `memd eval` framework with eval harness and scoring.
- Benchmark baseline: LongMemEval 82.8%, LoCoMo 41.5%, MemBench 34.6%, ConvoMem 0.0%.
- Post-M1: LongMemEval 96.0% (+13.2%).
- Evidence tables for retrieval feedback.

**Two independent proof tracks**:

**Track A — Correction Retention (needs D2)**:
- Store a fact → correct it → query for that topic → verify corrected version returned.
- Score: how often does the corrected version outrank or completely replace the original
  in retrieval results? Target: 100% (superseded should NEVER appear in recall).
- Cross-session: correct in session 1, query in session 2 → still corrected.

**Track B — Lane Relevance (needs G2)**:
- Store items across multiple lanes → query with lane-detectable context →
  verify same-lane items rank higher than cross-lane items.
- Score: lane-matched items should appear in top 3 of retrieval results.

**What to build**:

1. **Correction retention eval** — add eval scenarios to `memd eval` that:
   (a) store a fact, (b) correct it, (c) query for it, (d) assert corrected version returned,
   (e) assert superseded version NOT in results.
   - File: `crates/memd-client/src/evaluation/eval_report_runtime.rs` (add scenarios to `eval_bundle_memory`)

2. **Cross-session correction persistence test** — E2E test:
   (a) store + correct in one "session" context,
   (b) reset working memory,
   (c) rebuild working context (simulates new session),
   (d) verify corrected fact appears, original does not.
   - File: `crates/memd-server/src/tests/mod.rs` (new test)

3. **Lane relevance eval** — add eval scenarios that:
   (a) store 3 architecture items + 3 operations items,
   (b) query "system architecture" → verify architecture-lane items rank top 3,
   (c) query "deployment pipeline" → verify operations-lane items rank top 3.
   - File: `crates/memd-client/src/evaluation/eval_report_runtime.rs`

4. **Cross-harness continuity proof** (gap 12) — prove that items stored by one agent/system
   are retrievable by another. E2E test:
   (a) store item with `source_agent: "agent-A"`, `source_system: "system-A"`,
   (b) query from context of `agent-B` / `system-B`,
   (c) verify item returned (not filtered by source).
   - File: `crates/memd-server/src/tests/mod.rs` (new test)

5. **A/B influence test** — compare retrieval quality with corrections vs. without:
   (a) baseline: store 5 items, query, measure result quality,
   (b) treatment: store 5 items, correct 2, query, measure result quality,
   (c) assert treatment quality >= baseline (corrections improve, not degrade).
   - File: `crates/memd-client/src/evaluation/eval_report_runtime.rs`

6. **Benchmark re-run** — re-run LongMemEval, LoCoMo, MemBench after D2+G2 changes.
   Record numbers. Regression = stop (do not pass gate if any benchmark drops).
   - Protocol: [[docs/verification/PUBLIC_BENCHMARKS.md]]

**Pass gate**:
- LongMemEval >= 80% (currently 96% — watch for regression)
- LoCoMo above 41.5% baseline
- A/B influence test: corrections improve retrieval (not degrade)
- Cross-session: corrected fact persists across session boundary
- Cross-harness: item stored by agent-A retrievable by agent-B
- Superseded items NEVER appear in recall results

**Verify nodes**: S3 (drilldown from summary to evidence),
M4 (cross-session stability proven), M7 (canonical outranks candidate in retrieval),
P3 (contradiction detection works)

## Node-by-Node M2 E2E Tests

Every node in the architecture graph must pass its M2 criterion. Phase gates (D2/G2/E2/H2)
cover some nodes. The tests below cover ALL nodes. Run after all four phases pass.

### Ingest Layer

| Node | M2 Criterion | E2E Test |
|------|-------------|----------|
| I1 | Corrections routed to P4 | `memd correct --id <uuid>` → verify lifecycle event `correction_created` fired, P4 node (explain) shows correction chain |
| I2 | Spill→promotion pipeline | Accumulate spill → run `memd maintain --mode full` → verify spill items promoted to candidate or expired (not stuck) |
| I3 | Handoff preserves correction state | Correct an item → `memd handoff` → verify handoff packet includes the correction (supersedes chain intact) |

### Control Plane

| Node | M2 Criterion | E2E Test |
|------|-------------|----------|
| P1 | Lane-aware admission | (G2 gate) Store 8 items across 3 lanes → working memory admits mix of lanes, not all from one lane |
| P2 | Cross-session preferences persist | Store a preference → close session → open new session → `memd wake` → preference appears in wake packet |
| P3 | Lane-based routing, contradiction detection | (G2+D2 gates) Query with lane context → same-lane items rank higher. Correct an item → sibling with same entity gets Contested |
| P4 | Corrections change future recall, trust hierarchy enforced | (D2 gate) Correct an item → next retrieval returns corrected version. Human correction outranks synthetic |

### Typed Memory

| Node | M2 Criterion | E2E Test |
|------|-------------|----------|
| M1 | Reasons visible, rehydration works | `memd explain <id>` → shows admission reasons. Evicted item → rehydration via `rehydration_queue` in working context response → item returns to working memory |
| M2 | Preferences + architecture persist | After resume: architecture decisions and user preferences present without re-reading source docs |
| M3 | Timeline navigable via atlas | Store 5+ items → atlas regions generated → timeline visible via `atlas_explore` |
| M4 | Corrections update semantic fast, lanes tag correctly | (D2+G2 gates) Correct a fact → immediate retrieval returns corrected version. New items auto-tagged with detected lane |
| M5 | Procedures surface in relevant lanes | Store a procedure with ops content → auto-tagged `operations` lane → query for "deploy" → procedure returned |
| M6 | Promotion criteria enforced | Store weak candidate (low confidence, 0 rehearsals) → `memd maintain` → expires. Store strong candidate → maintain → promoted |
| M7 | Canonical outranks candidate in retrieval | Store identical item as both candidate and canonical → query → canonical version ranks higher |

### Recall Surfaces

| Node | M2 Criterion | E2E Test |
|------|-------------|----------|
| S1 | Lane-relevant items included, architecture decisions present | (G2 gate) Wake packet includes items from matching lane. Architecture decisions in wake packet |
| S2 | Navigable atlas with backlinks | (E2 gate) Wake → explore → expand → explain in ≤4 hops. Backlinks visible |
| S3 | Drilldown from summary to evidence | Pick a canonical item → `memd explain` → shows sources, lifecycle events, correction chain, raw evidence link |
| S4 | Source linkage from canonical to raw | For a canonical item created via correction, trace: canonical → superseded original → raw creation event |
| S5 | Two-way sync, readable vault | `memd obsidian compile` → vault has corrected items (not superseded). `memd obsidian import` → vault edits flow back to memd (import exists at `crates/memd-client/src/obsidian/import_runtime.rs`) |
| S6 | Compact semantic briefing | `memd brief` → briefing includes corrections and lane-relevant items, not just status |

### Live Loop M2 Test

Run the loop with a correction:
1. Store a fact → I1
2. Correct the fact → I1 → P4
3. Verify old fact Superseded, new fact Active → M4
4. Working memory updated → P1 → M1 (corrected version in, original evicted)
5. Compile wake packet → S1 (corrected fact present, original absent)

### Consolidation Loop M2 Test

1. Store a correction → P4
2. Run `memd maintain --mode full` → P3
3. Verify: contradiction detection runs, siblings marked Contested
4. Verify: promoted corrections survive, weak signals expire → M6 → M7

### Resume Loop M2 Test

1. Store 3 facts across 2 lanes + 1 preference + correct 1 fact
2. Close session context
3. Resume → load session continuity → P2 → M2
4. Merge with semantic memory → M4 (corrected fact present, original absent)
5. Pull relevant procedures → M5
6. Compile working context → P1 → M1 (lane-aware, correction-boosted)
7. Continue without big reread → S1

**Test**: After resume, agent has corrected facts + lane-tagged architecture knowledge
without re-reading source docs. Superseded items absent. Preferences persist.

## What NOT To Do

- Do not touch M3 phases (J2) or M4 phases (K2/L2/M2-evo/N2/I2) — they depend on M2
- Do not rebuild the correction pipeline — `correct_item` works. Fix contradiction detection.
- Do not hard-filter by lane — use scoring boosts. Hard filters break cross-lane recall.
- Do not remove the `redundancy_key` system — it's used for dedup. Fix contradiction detection
  by using entity-based matching INSTEAD, not by changing what redundancy_key computes.
- Do not skip the backfill steps — existing items with `lane: NULL` and empty `entity_links`
  are the whole point of G2 and E2.
- Do not merge M2 work until ALL four phases pass gates.

## Amnesia Prevention Checklist

After M2 passes, before declaring done:

- [ ] Corrected fact replaces original in all recall surfaces (wake, lookup, context)
- [ ] Superseded items NEVER appear in `build_context` or wake packet
- [ ] `memd explain <corrected_id>` shows full correction chain (original → superseded → corrected)
- [ ] Contradiction detection fires for two items claiming different things about same entity
- [ ] Trust hierarchy enforced: human correction outranks synthetic in working memory scoring
- [ ] All existing items have `lane` column populated (backfill ran)
- [ ] `memd remember "system architecture uses event sourcing"` → auto-tagged `lane: architecture`
- [~] Query with lane context → same-lane items rank higher in retrieval — **G2.2 deferred** (see ROADMAP.md decision log). Lane-tagged items rank above untagged (+0.06), but same-lane vs different-lane differential boost requires `query: Option<String>` on WorkingMemoryRequest. Accepted as known gap for M2.
- [ ] Entity links populated in production DB (not empty table)
- [ ] `memd explore` returns non-empty regions with navigable entities
- [ ] Wake → explore → expand → explain works in ≤4 hops
- [ ] Cross-session: correct in session 1, query in session 2 → corrected version returned
- [ ] Cross-harness: item stored by agent-A retrievable by agent-B
- [ ] LongMemEval >= 80% (no regression from 96%)
- [ ] LoCoMo above 41.5% baseline
- [ ] A/B influence test: corrections improve retrieval, not degrade
- [ ] `memd eval` score >= 65 (no regression from M1's 95)

If any of these fail, M2 is not done. Period. Items marked `[~]` are deferred with documented rationale — they represent known gaps accepted at gate review, not failures.

## Key Files

| What | Where |
|------|-------|
| Correction pipeline | `crates/memd-server/src/repair/mod.rs` |
| Contradiction detection (broken) | `crates/memd-server/src/repair/mod.rs:146–181` |
| Working memory scoring | `crates/memd-server/src/working/mod.rs:332` |
| Lane detection | `crates/memd-server/src/helpers.rs:335` |
| Entity auto-linking | `crates/memd-server/src/main.rs:255` |
| Wiki link parsing | `crates/memd-server/src/main.rs:300` |
| Source trust scoring | `crates/memd-server/src/store_entities.rs:251` |
| Explain/inspect | `crates/memd-server/src/inspection/mod.rs:12` |
| Atlas routes | `crates/memd-server/src/routes.rs:1662+` |
| Build context (retrieval) | `crates/memd-server/src/helpers.rs:27` |
| Redundancy key generation | `crates/memd-server/src/keys/mod.rs:53` |
| Store item (ingest) | `crates/memd-server/src/main.rs:92` |
| Existing tests | `crates/memd-server/src/tests/mod.rs` |
| Eval harness | `crates/memd-client/src/evaluation/eval_report_runtime.rs` (`eval_bundle_memory`) |
| Eval CLI entry | `crates/memd-client/src/cli/cli_analysis_runtime.rs` (`run_eval_command`) |
| Wake packet compiler | `crates/memd-client/src/runtime/resume/wakeup.rs` (`render_bundle_wakeup_markdown`) |
| Obsidian import (two-way sync) | `crates/memd-client/src/obsidian/import_runtime.rs` (`run_obsidian_import`) |
| Benchmarks | `crates/memd-client/src/benchmark/public_benchmark.rs` |
| Lane source material | `.memd/lanes/*/` |
| Store migrations | `crates/memd-server/src/store_migrations.rs` |

## Phase Docs (read before starting each step)

- [[docs/phases/phase-d2-correction-flow.md]]
- [[docs/phases/phase-g2-lane-architecture.md]]
- [[docs/phases/phase-e2-atlas-activation.md]]
- [[docs/phases/phase-h2-recall-proof.md]]
- [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- [[docs/verification/MEMD-10-STAR.md]]

## Theory Locks (read only if stuck on a design decision)

- [[memd-canonical-promotion-theory-lock-v1]] (D2)
- [[memd-lane-theory-lock-v1]] (G2)
- [[memd-atlas-theory-lock-v1]] (E2)
- [[memd-evaluation-theory-lock-v1]] (H2)
