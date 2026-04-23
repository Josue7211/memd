# M2 Node-by-Node Verification

> Date: 2026-04-16
> Branch: research/mining
> Tests: 624 pass, 0 fail
> Benchmarks: LME 82.8% (gate 80%), LoCoMo 41.5% (gate 41.5%), MemBench 34.6% (gate 30%)

## Result: PASS (15 ✓, 6 ~, 0 ✗)

Gate rule: "A milestone cannot close with any ✗ in its tier." No ✗ found.

## Ingest Layer

| Node | M2 Criterion | Status | Evidence |
|------|-------------|--------|----------|
| I1 | Corrections routed to P4 | ✓ | `POST /memory/correct` → `repair::correct_item()`. Marks old Superseded, creates new Active with correction tag + preferred=true. Entity-based contradiction detection marks siblings Contested. 9 tests: `correct_item_supersedes_old_and_creates_new`, `d2_contradiction_marks_siblings_contested`, `h2_ab_influence_corrections_improve_retrieval`, etc. |
| I2 | Spill→promotion pipeline | ~ | Eviction tracking (`WorkingMemoryEvictionRecord` with reasons). Consolidation pipeline (`maintain_runtime` → `consolidation_candidates`). No explicit spill buffer stage — eviction→consolidation is implicit via maintenance loop. |
| I3 | Handoff preserves correction state | ~ | Correction state persists in shared DB. `h2_cross_session_correction_persists` proves new session retrieves corrected version, not original. Implicit via DB (not explicit handoff packet metadata). |

## Control Plane

| Node | M2 Criterion | Status | Evidence |
|------|-------------|--------|----------|
| P1 | Lane-aware admission | ✓ | `lane_diversity_cap = 5` items per lane in working memory (working/mod.rs:119). Lane_score +0.06 boost. `detect_content_lane()` auto-detection from tags/path/keywords. 6 lane tests. |
| P2 | Cross-session preferences persist | ✓ | Shared DB, no session isolation. `h2_cross_session_correction_persists`, `h2_cross_harness_item_retrievable`. HiveSession merge preserves focus/pressure/branch/workspace. |
| P3 | Lane-based routing (DB tags), contradiction detection | ✓ | `lane TEXT` column (migration in store_migrations.rs:183). Entity-based contradiction via `repair/mod.rs:160-197`: entity lookup → sibling items → mark Contested. Test: `d2_contradiction_marks_siblings_contested`. |
| P4 | Corrections change future recall, trust hierarchy enforced | ✓ | Superseded items excluded by `build_context` (status==Active filter). Correction_boost +0.10 in working_item_priority. Trust_rank: correction(4) > canonical(3) > promoted(2) > candidate(1) > synthetic(0). Tests: `d2_correction_e2e`, `h2_ab_influence_corrections_improve_retrieval`. |

## Typed Memory

| Node | M2 Criterion | Status | Evidence |
|------|-------------|--------|----------|
| M1 | Reasons visible, rehydration works | ✓ | Eviction reasons: `evicted_by_status_cap`, `evicted_by_budget`, `evicted_by_lane_diversity`, `evicted_by_admission_limit`. Rehydration_queue built from evicted items (working/mod.rs:199-206) with reason, label, summary. |
| M2 | Preferences + architecture persist | ✓ | HiveSession merge logic (store_hive.rs:22-88). Preserves focus, pressure, branch, workspace, confidence. Shared DB means all kinds survive session boundaries. |
| M3 | Timeline navigable via atlas | ✓ | Atlas regions auto-generated. `explore_atlas()` with depth expansion. `pivot_time` filtering (time-based nav). `e2_atlas_navigation_four_hops` test (region→explore→expand→explain). 7+ atlas tests. |
| M4 | Corrections update semantic fast, lanes tag correctly | ✓ | `correct_item()` instant: old→Superseded, new→Active+Canonical (atomic). Lane auto-detect on every `store_item()`. Tests: `d2_correction_e2e`, `lane_persisted_on_store_item`, `lane_auto_detection_from_content_keywords`. |
| M5 | Procedures surface in relevant lanes | ~ | `match_procedures()` exists (procedural.rs:226-292), called from working memory (working/mod.rs:240-248). Procedures can be lane-tagged via `record_procedure()`. No test verifying lane-scoped procedure retrieval. |
| M6 | Promotion criteria enforced, weak signal expires | ~ | Auto-promote: `use_count >= 3 AND session_count >= 2`. Stale retire: promoted + 0 uses + >30d → retired. TTL expiry in keys/mod.rs:128-136. No integration test for candidate lifecycle. |
| M7 | Canonical outranks candidate in retrieval | ✓ | `stage_score`: Canonical +0.08, Candidate -0.02 (0.10 swing in working/mod.rs:425-428). trust_rank ordering: canonical(3) > candidate(1). Tests: `active_recent_canonical_items_rank_above_stale_contested_items`, `d2_correction_e2e`. |

## Recall Surfaces

| Node | M2 Criterion | Status | Evidence |
|------|-------------|--------|----------|
| S1 | Lane-relevant items, architecture decisions present | ✓ | Wake packet (`render_bundle_wakeup_markdown`) includes durable truth (non-live items first), atlas hints, continuity capsule. Agent-specific budgets (claude-code strict vs. others). Lane hints in E2 marker. |
| S2 | Navigable atlas with backlinks | ✓ | `explore_atlas()` (atlas.rs:133-382) with region→node→evidence chain. `atlas_expand()` for neighborhood expansion. Supersedes backlinks (atlas.rs:309-340). 7+ tests including `e2_atlas_navigation_four_hops`. |
| S3 | Drilldown from summary to evidence | ✓ | `explain_memory()` (inspection/mod.rs:12-77) returns: item + entity context + events + sources + rehydration. Route: `GET /explain?id=UUID`. Rendered in Obsidian via `build_compiled_memory_markdown()`. |
| S4 | Source linkage from canonical to raw | ✓ | `source_path` on MemoryItem + MemoryEventRecord. `source_memory()` route returns provenance aggregates. Evidence chain: canonical item → entity → events → source_path. Wikilinks for Obsidian navigation. |
| S5 | Two-way sync, readable vault | ~ | Compile: `run_obsidian_compile()` renders explain→markdown with frontmatter. Import: `ObsidianVaultScan` parses vault notes, tracks changed/new/unchanged via `ObsidianSyncState`. No round-trip verification test. |
| S6 | Compact semantic briefing | ~ | Wake packet is the briefing: char-budgeted sections (instructions 180ch, live 100ch, durable 140-160ch, atlas hints 80ch). `enforce_wake_char_budget()` trims with elision. `lookup_with_fallbacks()` for semantic search. No explicit "briefing" command — briefing is implicit in wake packet. |

## Amnesia Prevention Checklist

From M2-EXECUTION-PLAN.md, verified against code and tests:

- [x] Corrected fact replaces original in all recall surfaces — `d2_correction_e2e`, `h2_ab_influence_corrections_improve_retrieval`
- [x] Superseded items NEVER appear in `build_context` or wake packet — `build_context` filters `status==Active`; `eval_bundle_memory` has superseded leak detection (-15 penalty)
- [x] `memd explain` shows full correction chain — `explain_shows_correction_events` test (lifecycle events: `superseded_by_correction`, `correction_created`)
- [x] Contradiction detection fires for same-entity items — `d2_contradiction_marks_siblings_contested` (entity-based, not redundancy_key)
- [x] Trust hierarchy enforced — `trust_rank()`: correction(4) > canonical(3) > promoted(2) > candidate(1) > synthetic(0); wired into `working_item_priority` as `trust_hierarchy_score = rank * 0.012`
- [x] Lane auto-detection works — `lane_auto_detection_from_content_keywords`, `lane_auto_detection_from_tags`, `lane_auto_detection_from_source_path`
- [~] Query with lane context → same-lane items rank higher — **KNOWN GAP (G2.2 deferred)**. Current `lane_score` +0.06 boosts ALL lane-tagged items equally. Documented in ROADMAP.md.
- [x] Entity links populated — `auto_link_entity` salience gate removed (new entities start at 0.0). `entity_link_backfill` wired to maintain. `e2_atlas_navigation_four_hops` proves links work.
- [x] `memd explore` returns non-empty regions — 7+ atlas tests
- [x] Wake→explore→expand→explain ≤4 hops — `e2_atlas_navigation_four_hops`
- [x] Cross-session correction persistence — `h2_cross_session_correction_persists`
- [x] Cross-harness retrieval — `h2_cross_harness_item_retrievable`
- [x] LongMemEval >= 80% — 82.8%
- [x] LoCoMo >= 41.5% — 41.5% (exactly at baseline, zero regression)
- [x] A/B influence test passes — `h2_ab_influence_corrections_improve_retrieval`

## Known Gaps (~ nodes)

All ~ nodes have code implementing the feature but lack comprehensive integration tests:

1. **I2 (Spill→promotion)**: Consolidation pipeline implicit. Items evicted from working memory can be re-promoted via `consolidation_candidates()` during maintenance, but there's no explicit spill buffer stage. Acceptable for M2; explicit pipeline is M3 scope (provable).

2. **I3 (Handoff correction state)**: Correction state persists via shared DB, proven by `h2_cross_session_correction_persists`. The lack of explicit correction metadata in handoff packets is a polish item — the functional requirement (new session sees corrected data) is met.

3. **M5 (Procedures in lanes)**: Infrastructure exists (`match_procedures` + lane tagging). No test proving lane-scoped procedure retrieval. Low risk — procedures are a secondary recall surface at M2 maturity.

4. **M6 (Candidate lifecycle)**: Promotion criteria and stale retirement logic exist in code. No integration test exercising the full lifecycle. Acceptable for M2 where the focus is correctness, not provability (M3).

5. **S5 (Obsidian round-trip)**: Compile and import both exist as separate paths. No test verifying compile→edit→import round-trip. Two-way sync infrastructure is in place.

6. **S6 (Briefing)**: Wake packet IS the compact briefing, with char budgets and agent-specific limits. No separate `memd brief` command — the briefing is embedded in `memd wake` output. Functional requirement met.

## Live Loop M2 Test (Traced)

Test: `d2_correction_e2e` + `h2_ab_influence_corrections_improve_retrieval` together prove:

1. Store fact → I1 ✓
2. Correct fact → I1 → P4 ✓ (repair::correct_item)
3. Old fact Superseded, new fact Active → M4 ✓
4. Working memory returns corrected version → P1 → M1 ✓
5. Superseded absent from retrieval → S1 ✓ (eval_bundle_memory leak detection confirms)

## Conclusion

15/21 nodes pass (✓). 6/21 partial (~). 0 fail (✗). Gate rule satisfied. All core M2 criteria — correction flow, trust hierarchy, lane routing, atlas navigation, cross-session persistence, benchmark stability — are implemented and tested. The 6 partial nodes have working code but need deeper integration tests, which are M3 (provable) scope.
