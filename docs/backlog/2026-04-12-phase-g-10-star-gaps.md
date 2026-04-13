# Phase G 10-Star Gaps

Status: `in_progress` (8 closed, 4 deferred medium, 3 deferred as feature work, 1 deferred to Phase H)
Created: 2026-04-12
Phase: G (Procedural Learning) + cross-phase integration

## Phase G Internal Gaps

### G1. Auto-promote thresholds — `closed`
Auto-promote in `use_procedure()`: when use_count >= 3 AND session_count >= 2,
candidate auto-promotes to promoted. `ProcedureUseResponse.auto_promoted` flag
signals when it happens. Test: `auto_promote_after_threshold`.

### G2. Match scoring primitive — `deferred` (feature work)
Keyword overlap only. Fuzzy/embedding matching is a standalone feature,
not a gap fix. Defer to a future milestone.

### G3. Procedure versioning — `deferred` (feature work)
In-place step edits with version history is real engineering scope.
Confidence + supersedes serves as an interim versioning alternative.

### G4. Detect quality — noisy steps — `deferred` (medium)
Step normalization and deduplication across similar summaries.
Worth doing but not blocking Phase G verification.

### G5. Wake budget pressure — `closed`
Procedures are part of the `prefix` string in `render_bundle_wakeup_markdown`,
which feeds into `enforce_wake_char_budget()`. Under budget pressure,
procedure section (near the end of prefix) gets trimmed first. Correct behavior.

### G6. No auto-retire — `closed`
`auto_retire_stale_procedures()` retires promoted procedures with 0 uses
and updated_at older than 30 days. Called at the start of `match_procedures()`.
Test: `auto_retire_stale_procedures`.

### G7. No procedure conflict detection — `closed`
On `record_procedure()`, checks for existing promoted procedures with
overlapping trigger keywords (2+ shared words or 1 if trigger is single-word).
Returns `ProcedureRecordResponse.conflicts` with matching procedures.
Test: `conflict_detection_on_record`.

### G8. No procedure execution recording — `deferred` (medium)
Outcome tracking (worked/failed) beyond use_count. Worth doing but
not blocking Phase G verification.

### G9. No procedure export to Obsidian — `deferred` (feature work)
Atlas compiles to Obsidian but procedures don't. New integration,
not a gap fix. Defer to future milestone.

## Cross-Phase Integration Gaps

### X1. Procedures missing correction/supersedes integration (Phase D × G) — `closed`
`Procedure.supersedes: Option<Uuid>` added. `ProcedureRecordRequest` accepts
`supersedes` for explicit chaining. `detect_procedures` filters out events
whose source entities have `current_state == "superseded"`.
Test: `supersedes_field_persists`.

### X2. No procedure-aware atlas integration (Phase F × G) — `deferred` (medium)
Procedures table not yet in atlas region generation. Atlas already handles
`MemoryKind::Procedural` items. Full integration deferred.

### X3. Procedures not integrated with hive coordination (Phase G × H) — `deferred` (Phase H)
Cross-harness procedure sharing. Explicitly depends on Phase H landing.

### X4. No candidate-procedural lane (Phase C × G) — `deferred` (medium)
`ProcedureStatus::Candidate` exists but doesn't integrate with the main
candidate memory system. Deferred until candidate memory gets reworked.

## Documentation Gaps

### D1. Test count mismatch — `closed`
ROADMAP says 9 procedural tests. Verified: 8 in procedural.rs + 1 routing
test = 9 server procedural tests. Now 13 after gap closure (+4 new tests).

### D2. Phase F doc outdated counts — `closed`
Updated from "13 tests, 85 total server tests" to "18 atlas tests, 98 total
server tests".

### D3. ProcedureKind::Policy vs theory "preferences" — `closed`
Documented in Phase G doc: theory "operating preferences" maps to
`ProcedureKind::Policy`. Same lifecycle as Workflow/Recovery.
