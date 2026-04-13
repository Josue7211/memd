# Phase G 10-Star Gaps

Status: `open`
Created: 2026-04-12
Phase: G (Procedural Learning) + cross-phase integration

## Phase G Internal Gaps

### G1. Auto-promote thresholds
No automatic candidate → promoted based on use_count/session_count.
Promotion is manual only. A 10-star system would auto-promote after
N uses across M sessions.

### G2. Match scoring primitive
Keyword overlap only. No fuzzy matching, no synonym awareness.
"deploy" won't match "ship to production". Need TF-IDF or
lightweight embedding similarity.

### G3. Procedure versioning
No way to update steps on an existing procedure. Must retire + re-record.
Should support in-place step edits with version history.

### G4. Detect quality — noisy steps
Event summaries become steps directly. No deduplication across
similar summaries, no step normalization. Steps could be noisy
or redundant.

### G5. Wake budget pressure
Procedures section added to wake packet but no budget accounting.
Could push over budget on claude-strict agents. Need to count
procedure chars against the wake char budget.

### G6. No auto-retire
Procedures with 0 uses over N days should auto-decay or auto-retire.
Currently they persist forever as promoted.

### G7. No procedure conflict detection
Two procedures with overlapping triggers could both match.
No dedup or conflict resolution between competing procedures.

### G8. No procedure execution recording
After following a procedure, no way to record "this worked" or
"this failed". Use-count only tracks invocation, not outcome.

### G9. No procedure export to Obsidian
Atlas compiles to Obsidian. Procedures don't. Should have
`memd procedure compile` for vault surface.

## Cross-Phase Integration Gaps

### X1. Procedures missing correction/supersedes integration (Phase D × G)
Procedure struct has no `supersedes` field. Procedures built from
evidence that was later corrected remain valid. Promote doesn't
check if source_ids items are still active. Detect doesn't filter
out superseded source events.

### X2. No procedure-aware atlas integration (Phase F × G)
Procedures should appear in atlas regions under the "procedures"
lane. Currently atlas generates regions from memory_items with
kind=Procedural, but doesn't include the procedures table.

### X3. Procedures not integrated with hive coordination (Phase G × H)
Cross-harness procedure sharing not implemented. When Phase H
lands, procedures need to be shareable across harnesses with
ownership and freshness rules.

### X4. No candidate-procedural lane (Phase C × G)
Theory lock specifies `candidate-procedural` as a typed internal
lane of candidate memory. Current implementation uses ProcedureStatus::Candidate
but doesn't integrate with the main candidate memory system.

## Documentation Gaps

### D1. Test count mismatch
ROADMAP says 9 procedural tests. Actual count should be verified
after all changes land (routing test counts as one).

### D2. Phase F doc outdated counts
Phase F doc says "13 tests, 85 total server tests". Reality is
18 atlas tests, 98 server tests. Update to match.

### D3. ProcedureKind::Policy vs theory "preferences"
Theory lock names "operating preferences" explicitly. Code maps
this to ProcedureKind::Policy. Either rename or document the mapping.
