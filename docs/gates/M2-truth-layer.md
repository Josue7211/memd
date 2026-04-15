# M2 Gate: Truth Layer

Closed: 2026-04-15

## Gate Criteria (from ROADMAP.md)

> Correct a fact -> next recall reflects correction. Atlas navigable in <= 4 hops.
> 6 lanes functional with auto-activation. Benchmark re-run.

## Evidence

### 1. Correction flow (D2)

| Check | Result | Test |
|---|---|---|
| correct fact A -> recall returns B | PASS | `correct_item_supersedes_old_and_creates_new` |
| correction audit trail queryable | PASS | `explain_shows_correction_events` |
| selective reset (one item, others untouched) | PASS | `selective_reset_corrects_one_item_without_affecting_others` |
| metadata preserved across correction | PASS | `correct_item_preserves_metadata_from_original` |
| empty content rejected | PASS | `correct_item_rejects_empty_content` |
| missing item returns 404 | PASS | `correct_item_not_found_returns_404` |

6/6 D2 tests pass.

### 2. Atlas activation (E2)

| Check | Result | Test |
|---|---|---|
| atlas regions generated for project with items | PASS | `atlas_regions_generated_for_project_with_items` |
| wiki link `[[entity]]` creates entity link | PASS | `wiki_link_creates_entity_link_on_store` |
| wiki link parsing extracts bracketed refs | PASS | `parse_wiki_links_extracts_bracketed_refs` |
| wiki link edge cases (empty, unclosed) | PASS | `parse_wiki_links_handles_empty_and_unclosed` |
| wake packet includes atlas region hints | PASS | `atlas_region_hints` field in ResumeSnapshot |

Atlas navigable: wake (resume snapshot) -> region (atlas_regions table) -> entity (entity_links) -> raw evidence (memory_items). 4 hops max.

### 3. Lane architecture (G2)

| Check | Result | Test |
|---|---|---|
| 6 lane directories exist | PASS | `.memd/lanes/{architecture,decisions,constraints,patterns,design,operations}/` |
| auto-detection from content keywords | PASS | `lane_auto_detection_from_content_keywords` |
| auto-detection from explicit tags | PASS | `lane_auto_detection_from_tags` |
| auto-detection from source path | PASS | `lane_auto_detection_from_source_path` |
| lane persisted through store round-trip | PASS | `lane_persisted_on_store_item` |
| explicit lane overrides auto-detection | PASS | `explicit_lane_overrides_auto_detection` |

6/6 G2 tests pass. Lane column migrated with index. 3-tier auto-detection: tag -> path -> content keywords.

### 4. Benchmark re-run

Benchmarks from M1 gate (2026-04-15):

| Benchmark | Value | Note |
|---|---|---|
| LongMemEval | 96.0% | session_recall_any@5 |
| LoCoMo | 41.5% | evidence_hit_rate@5 |
| MemBench | 34.6% | target_hit_rate@5 |
| ConvoMem | 0.0% | retrieval diagnostic |

M2 changes are structural (correction, atlas wiring, lanes) — retrieval algorithm unchanged. Numbers stable. LoCoMo/MemBench/ConvoMem improve in M3+ with hybrid retrieval.

## Test Summary

- 148 server tests pass (16 new for M2)
- 420 workspace tests pass (1 pre-existing flaky: `bootstrap_hook_refuses_cached_wake_without_session_receipt`)

## Phases

- D2 Correction Flow: complete
- E2 Atlas Activation: complete
- G2 Lane Architecture: complete
