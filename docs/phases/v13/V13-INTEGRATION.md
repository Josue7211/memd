---
version: v13
kind: integration-plan
status: closed
opened: 2026-04-22
revised: 2026-05-05
scope: A13..G13
depends_on: [../../verification/0.1.0-CONTRACT.md, ../../verification/0.1.0-AXIS-OWNERSHIP.md, ../../verification/milestones/MILESTONE-v13.md, ../../theory/MEMD-SOTA-THEORY.md]
---

# V13 Integration - Full 0.1.0 Release Harness

> Read after `docs/verification/milestones/MILESTONE-v13.md`. This doc covers the release harness architecture, proof-run directory structure, zero-generosity scorecard regenerator, TE zero-margin contingency, and commit strategy for 0.1.0 tag.

## Close Status

V13 closed on 2026-05-05. The release proof is rerunnable with:

```bash
RUN_DATE=2026-05-05 scripts/verify/v13-release-suite.sh
```

Evidence lives in `docs/verification/release-0-1-0/`; the strict scorecard
regenerated to composite `8.50/10`.

## 1. Execution-order discipline

Phase-level dependency (strict):

```
A13 ──► B13 ──────┐
                  │
C13 ──► D13 ──► E13 ┤
                  │
F13 ──────────────┤
                  │
                  ▼
                  G13
```

Rules:

- A13 (cross-device sync) must land Task A13.6 (sync conflict-resolution contract doc) before G13 harness executes.
- B13 (compaction-aware perf) is independent and can parallelize with A13 after shared fixtures settle (Task A13.4).
- C13 (multi-hop chains) and D13/E13 (routine composition + sharing) parallelize after C13.3 lands (correction graph mutation surface).
- F13 (domain-tuned retrieval) can start once C13 lands (to consume corrected retrieval index).
- G13 executes last, requires all A–F outputs plus the full release harness specification.

No phase may short-circuit a prior dependency to hit its own pass gate. If blocked, file a backlog item and surface in the next session's handoff.

## 2. Full 0.1.0 Release Harness Architecture

### 2.1 Harness scope (G13)

The release harness is a composite of seven per-axis subscenarios executed in sequence on the same memory workspace, with shared fixtures and shared session state. Unlike V4–V12 harnesses that focus on a single axis or 2–3 axes, G13 is a **full axis battery** — all 7 axes simultaneously, with strict assertions for every axis.

**Memory workspace**: Single persistent workspace (`test-workspace-0.1.0-release`). All sessions (G13.A through G13.G) write to the same `memory_items` table. State from one scenario feeds into the next. Compaction events are synthetic but realistic (triggered at 500-turn boundaries).

**Harness phases (7 subscenarios)**:

1. **G13.A** (SC + integration): Dormant-project recovery. Session 1: establish project A context (10 turns). Hibernation: 30-day clock jump. Session 2: wake project A; assert focus re-hydration (5 turns). Assert: wake produces prior context without cold-start quality loss.

2. **G13.B** (SC continued): Compaction-aware long-session perf. Continuation session 3: 200 turns across 4 synthetic compaction cycles (at turns 50, 100, 150, 200). Assert: wake context stays <1500 tokens; compaction markers visible in trace; session state survives intact.

3. **G13.C** (CR): Multi-hop correction chains. Session 4: corrections to value X (turn 15) and values Y, Z that derived from X (turns 25–30). Turn 40: query "what is Y?"; assert it reflects X's correction. Next session (session 5): wake; turn 45: confirm Y still reflects X's correction. Provenance graph shows correction lineage.

4. **G13.D** (PR): Routine composition. Session 6: three same-day file-read sequences (A file, B file, C file) in turns 50, 55, 60. Turn 61: memd surfaces suggestion "A+B+C = ReadMigrationSequence routine". Turn 62: user confirms composition. Turn 65: user invokes routine; assert execution trace shows composed routine, not individual file-reads.

5. **G13.E** (PR continued, TP): Cross-workspace routine sharing. Session 7: export ReadMigrationSequence routine to workspace `test-workspace-xws-share`. Switch to workspace `test-workspace-xws-share`; session 8: user queries `memd routine list --shared`; routine appears with origin metadata (authored in test-workspace-0.1.0-release, usage count, provenance). Session 8, turn 10: invoke shared routine; assert execution produces same behavior as original.

6. **G13.F** (RR): Public-bench sweep. Four independent subscenarios against LoCoMo, LongMemEval, MemBench, ConvoMem benchmarks. Each runs memd's retrieval against the benchmark's eval set. Measure: token F1 (LoCoMo), judged accuracy (LongMemEval), MC accuracy (MemBench), accuracy (ConvoMem). Compare to published SOTA baselines. Assert: all four ≥5pp margin.

7. **G13.G** (TP): Export + third-party replay. Session 9: full 20-turn interaction with 5 corrections, 3 routine uses. Task G13.G.1: export provenance snapshot (JSON file with memory state, correction graph, routine definitions, session trace). Task G13.G.2: third-party harness (standalone Rust binary or Python script) ingests the export and replays turns 1–20 without access to memd runtime. Assert: replay harness answers match original memd answers on all 20 turns. Audit-trail compliance format documented (not certified).

### 2.2 Harness assertion fixtures

Shared fixture directory: `crates/memd-client/fixtures/shared/release-0-1-0/`

| Fixture | Phase | Purpose |
| --- | --- | --- |
| `sessions/dormant-30d.jsonl` | G13.A | Session 1 + 30-day gap + session 2 wake scenario |
| `sessions/long-session-200t-4c.jsonl` | G13.B | 200-turn session with 4 synthetic compaction events (50T, 100T, 150T, 200T boundaries) |
| `corrections/multihop-x-to-y-z.jsonl` | G13.C | Correction chain: X corrected in turn 15 → Y, Z derived from X corrected/asserted in turns 25–30 → next-session confirmation |
| `routines/composition-abc.jsonl` | G13.D | Three same-day file-read sequences; auto-composition suggestion at turn 61 |
| `routines/xws-share-manifest.json` | G13.E | Cross-workspace routine export; second workspace ingestion scenario |
| `benches/locomo-test-set.jsonl` | G13.F | LoCoMo benchmark eval set (curated subset; full set if <10MB) |
| `benches/longmemeval-test-set.jsonl` | G13.F | LongMemEval benchmark eval set |
| `benches/membench-test-set.jsonl` | G13.F | MemBench benchmark eval set |
| `benches/convomem-test-set.jsonl` | G13.F | ConvoMem benchmark eval set |
| `export/full-session-9-export.json` | G13.G | Provenance snapshot from session 9 (20 turns, 5 corrections, 3 routines) |
| `replay/third-party-harness.rs` or `.py` | G13.G | Standalone third-party replay implementation; ingests export and re-answers all turns |

Fixture consolidation: All seven subscenarios share the same workspace. Session state carries forward (sessions 1→2→3→...→9 sequential, same `workspace_id`). No reset between subscenarios.

## 3. Proof-run directory structure

Directory: `docs/verification/release-0-1-0/`. Populated at G13 close by scorecard regenerator.

```
docs/verification/release-0-1-0/
├── YYYY-MM-DD-g13-harness.ndjson              # full G13 run log (all phases in NDJSON)
├── YYYY-MM-DD-axis-session_continuity.ndjson  # per-axis harness run (subscenario G13.A/B)
├── YYYY-MM-DD-axis-session_continuity-review.md
├── YYYY-MM-DD-axis-correction_retention.ndjson
├── YYYY-MM-DD-axis-correction_retention-review.md
├── YYYY-MM-DD-axis-procedural_reuse.ndjson
├── YYYY-MM-DD-axis-procedural_reuse-review.md
├── YYYY-MM-DD-axis-raw_retrieval.ndjson      # four bench runs: LoCoMo, LME, MemBench, ConvoMem
├── YYYY-MM-DD-axis-raw_retrieval-review.md    # margin table: LoCoMo +5.2pp, LME +4.8pp, etc.
├── YYYY-MM-DD-axis-trust_provenance.ndjson
├── YYYY-MM-DD-axis-trust_provenance-review.md
├── YYYY-MM-DD-margin-targets.md                # public-bench comparison table
├── YYYY-MM-DD-te-integration-check.ndjson     # TE regression check (non-owned, must ≥7)
├── YYYY-MM-DD-ch-integration-check.ndjson     # CH regression check (non-owned, must ≥8)
└── YYYY-MM-DD-0-1-0-release-ready.txt         # final gate: all conditions met, safe to tag
```

### 3.1 Per-axis review documents

Example: `YYYY-MM-DD-axis-raw_retrieval-review.md`:

```markdown
# Raw Retrieval (RR) — V13 Release Proof

## Assertion status

✓ LoCoMo (token F1): 0.77 vs SOTA 0.72, margin +5pp
✓ LongMemEval (judged acc): 0.735 vs SOTA 0.68, margin +5.5pp
✓ MemBench (MC acc): 0.805 vs SOTA 0.75, margin +5.5pp
✓ ConvoMem (accuracy): 0.752 vs SOTA 0.70, margin +5.2pp

All four benches ≥5pp margin. RR axis lifts from 8 to 9.

## Proof transcript

NDJSON path: docs/verification/release-0-1-0/YYYY-MM-DD-axis-raw_retrieval.ndjson

Entries show per-query:
- Query ID
- Query text
- Retrieved item IDs (top-k by rank)
- Ground truth label
- F1 / accuracy computed

Summary stats (computed from NDJSON):
- LoCoMo F1: 0.77 (n=200 queries)
- LongMemEval accuracy: 0.735 (n=150 queries)
- MemBench accuracy: 0.805 (n=100 queries)
- ConvoMem accuracy: 0.752 (n=150 queries)

Human review: All metrics stable. No outlier queries. Domain-tuned retrieval (code/docs/conversational) correctly weighted per query intent. Drift detection from V13.E not affecting retrieval quality.

## Timestamp

Run completed: YYYY-MM-DD HH:MM:SS UTC by agent <agent-id>
Human reviewed by: <reviewer-name> on YYYY-MM-DD
```

### 3.2 Margin targets table

File: `YYYY-MM-DD-margin-targets.md`

```markdown
# V13 Release — Public Benchmark Margin Targets

| Benchmark | Axis | SOTA baseline | V13 measured | Margin | Status |
|-----------|------|---------------|--------------|--------|--------|
| LoCoMo (token F1) | RR | 0.72 | 0.77 | +5pp | ✓ PASS |
| LongMemEval (judged acc) | RR | 0.68 | 0.735 | +5.5pp | ✓ PASS |
| MemBench (MC acc) | RR | 0.75 | 0.805 | +5.5pp | ✓ PASS |
| ConvoMem (accuracy) | RR | 0.70 | 0.752 | +5.2pp | ✓ PASS |
| LongMemEval multi-session | SC | 0.65 | 0.638 | parity (−1.2pp) | ✓ PARITY |
| LoCoMo multi-turn | CR | 0.58 | 0.572 | parity (−0.8pp) | ✓ PARITY |

Aggregate: All target margins met or matched as parity-acceptable. RR dominates (≥5pp on all four). SC and CR maintain parity. Release gate condition 5 satisfied.
```

## 4. Zero-generosity scorecard regenerator (strict mode)

The regenerator is a CLI tool invoked at G13.8 (Task G13.8: scorecard-regeneration-lock). Signature: `memd scorecard regenerate --strict --lock-at-8-50`.

### 4.1 Strict-mode rules

1. **No margin of error on axis scores.** If harness evidence supports 8.7/10 on an axis, regenerator writes 8.7, not 9. No rounding up.
2. **Per-axis floor enforcement.** If harness produces axis score < floor value in MILESTONE-v13.md, regenerator aborts with error (does not write).
3. **Composite floor enforcement.** If weighted composite < 8.00, regenerator aborts. V13 target is 8.50; if evidence yields 7.98 composite, regenerator refuses to write and stages rollback to V12 scores.
4. **TE zero-margin trap.** If TE score < 7 in regeneration, regenerator aborts immediately and surfaces TE-contingency-trigger log. Does not regenerate other axes. See Contingency Plan below.
5. **No credit for unowned axes.** Regenerator enforces per 0.1.0-AXIS-OWNERSHIP.md: only SC +1, CR +1, PR +1, RR +1, TP +1 allowed. CH and TE must be ≥8 and ≥7 respectively (inherited from V12/V11) but no lift claimed.

### 4.2 Invocation and lockdown

G13.8 invocation:

```bash
memd scorecard regenerate \
  --strict \
  --lock-at-8-50 \
  --harness-run docs/verification/release-0-1-0/YYYY-MM-DD-g13-harness.ndjson \
  --output docs/verification/MEMD-10-STAR.md \
  --evidence-dir docs/verification/release-0-1-0/
```

Regenerator output:

1. Reads harness NDJSON and extracts per-axis assertion pass/fail.
2. Computes axis scores (integer 0–10, basis from assertion evidence).
3. Applies strict-mode checks (1–5 above).
4. If all checks pass: writes new scorecard table in `docs/verification/MEMD-10-STAR.md` (overwrites composite row only, preserves per-axis narrative). Appends dated evidence block.
5. If any check fails: logs error, stages rollback file `docs/verification/MEMD-10-STAR.md.rollback` (V12 state), exits with non-zero code.

Example abort output (TE regression):

```
ERROR: TE score 6.8 < floor 7.0. V13 composite cannot achieve 8.50 with TE <7.
ABORT: Invoking TE-contingency-trigger. File v13.5-recovery phase. Do not tag 0.1.0.
Rollback state staged: docs/verification/MEMD-10-STAR.md.rollback
```

### 4.3 Scorecard write-back format

The regenerator replaces the composite table (and per-axis rows) in `MEMD-10-STAR.md` with the new evidence-backed values. Example:

```markdown
## 10-Star Composite Scorecard

| Axis | Weight | Score | V13 Basis |
|------|--------|-------|-----------|
| Session continuity | 20% | 9/10 | A13 dormant-project wake (30d gap, focus re-hydrated) + B13 1000-turn compaction-aware session survival |
| Correction retention | 15% | 8/10 | C13 multi-hop correction chains: correction to X affects Y, Z downstream; provenance graph shows lineage |
| Procedural reuse | 15% | 9/10 | D13/E13 routine composition (A+B=C auto-suggested, curated, shared) + cross-workspace provenance |
| Cross-harness continuity | 15% | 8/10 | CH integration: multi-harness live session (claude-code + codex simultaneous) same memory view, zero regression from V12 |
| Raw retrieval strength | 15% | 9/10 | F13 domain-tuned: LoCoMo F1 +5pp, LongMemEval +5.5pp, MemBench +5.5pp, ConvoMem +5.2pp |
| Token efficiency | 10% | 7/10 | TE integration: dynamic compiler functional, wake median ≤1500 tokens (V11 locked), zero regression |
| Trust + provenance | 10% | 9/10 | G13 export + third-party replay: memory state portable, audit-trail compliance-ready |

**Composite: 8.50** (0.20×9 + 0.15×8 + 0.15×9 + 0.15×8 + 0.15×9 + 0.10×7 + 0.10×9 = 8.50)

**Regenerated**: YYYY-MM-DD HH:MM:SS by G13 scorecard regenerator (strict mode).
**Evidence**: docs/verification/release-0-1-0/YYYY-MM-DD-*.ndjson

**0.1.0 Release Gate**: ALL CONDITIONS MET
1. ✓ Composite 8.50 ≥ 8.0
2. ✓ Every axis ≥ 7 (SC 9, CR 8, PR 9, CH 8, RR 9, TE 7, TP 9)
3. ✓ Zero blocker-severity backlog
4. ✓ Reproducible proof run in docs/verification/release-0-1-0/
5. ✓ Head-to-head SOTA: RR all four benches +5pp; SC/CR parity-with-margin

**SAFE TO TAG 0.1.0**
```

## 5. TE Zero-Margin Contingency Plan

**Condition**: G13 scorecard regenerator (strict mode) produces TE score < 7.

**Trigger**: At regeneration time, if TE <7, regenerator aborts with error `TE-CONTINGENCY-TRIGGER`.

**Action sequence**:

1. **Do not tag 0.1.0.** The release gate condition 2 (every axis ≥7) is violated.
2. **Roll back V13 axis credits.** MEMD-10-STAR.md stays at V12 state (SC 8, CR 7, PR 8, RR 8, TP 8; composite 7.75).
3. **Create v13.5 milestone (TE recovery).** File `docs/verification/milestones/MILESTONE-v13.5.md` with:
   - Status: planned
   - Scope: single phase C13.5 (TE recovery)
   - Composite: 7.75 (no change; only TE touched, all others stay at V12 values)
   - Goal: root-cause TE <7 regression and restore ≥7 within C13.5 scope
4. **Single recovery phase**: C13.5 (TE recovery). Scope:
   - Debug dynamic compiler (D11) — check if recent V13 changes (A13 sync overhead, E13 routine-curation metadata) added unexpected tokens.
   - Options: (a) optimize D11 dynamic compiler to shed tokens, or (b) revert breaking change in A/D/E that caused regression.
   - Exit gate: TE ≥7 confirmed by G13.5 harness re-run (one-axis assertion only).
5. **C13.5 exit commits**: Typical phase commits (1 per task).
6. **V13.5 handoff and tag**: After C13.5 closes with TE ≥7, regenerate scorecard again (G13.5 harness). If ≥7, proceed to tag 0.1.0.

**Rationale**: TE is the tightest axis at release (zero margin over floor). Rather than accept 0.1.0 at 7.99 composite (below 8.0 gate), we recover TE to ≥7 in a focused recovery phase. This preserves the "SOTA, not production floor" theory and maintains release gate integrity.

## 6. Feature-flag graduation calendar

V13 introduces four new features. Graduation order (each 7-day clean window):

1. `MEMD_A13_CROSS_DEVICE_SYNC` = 1 (after A13 task A13.9)
2. `MEMD_D13_ROUTINE_COMPOSITION` = 1 (after D13 task D13.8)
3. `MEMD_F13_DOMAIN_TUNED_RETRIEVAL` = 1 (after F13 task F13.8)
4. `MEMD_G13_EXPORT_THIRD_PARTY_REPLAY` = 1 (after G13 task G13.7)

**Calendar spillover**: 4 graduations × 7 days = 28 days of post-G13 observation. V13 code-complete and G13 harness pass are the milestone-close bar; flag-graduation runs post-0.1.0-tag into operational monitoring (no block on tagging).

## 7. Commit strategy

### Plan-spec land phase (this task)

Two atomic commits on `research/mining`:

1. `docs(v13): MILESTONE-v13 release gate + axis targets`
2. `docs(v13): V13-INTEGRATION full 0.1.0 release harness architecture`

### Execution commits per phase

Each phase plan (A13–G13) has its own internal task list that commits per task. Those execution commits are produced by future agents — **not** by the plan-spec-land task.

### Release tag commit (after G13 harness passes)

After G13 closes and scorecard regenerator confirms all conditions met, a single tag commit:

```
release: 0.1.0 (Evidence + Release milestone V13 closed)
```

Content: tag annotation includes summary of release conditions (composite 8.50, every axis ≥7, reproducible proof run, SOTA margins).

Example tag message:

```
memd 0.1.0 — SOTA Memory OS for Any Harness

Release gated by 0.1.0-CONTRACT.md; all five conditions met:
1. Composite 8.50 ≥ 8.0 ✓
2. Every axis ≥7: SC 9, CR 8, PR 9, CH 8, RR 9, TE 7, TP 9 ✓
3. Zero blocker-severity backlog ✓
4. Reproducible proof run in docs/verification/release-0-1-0/ ✓
5. SOTA head-to-head: RR all four benches +5pp; SC/CR parity ✓

Regenerated 0.1.0 at 2026-MM-DD by G13 scorecard regenerator (strict mode).

Axes lifted in V13:
- SC 8→9: dormant-project recovery (30d wake), CRDT cross-device sync
- CR 7→8: multi-hop correction chains (A affects Y, Z downstream)
- PR 8→9: routine composition (auto A+B=C) + cross-workspace sharing
- RR 8→9: domain-tuned retrieval; LoCoMo/LME/MemBench/ConvoMem all +5pp
- TP 8→9: export + third-party replay harness; compliance-ready audit trails

Axes integrated with zero regression:
- CH 8: multi-harness live session parity (claude-code + codex simultaneous)
- TE 7: dynamic compiler (V11) functional; wake median ≤1500 tokens

Theory: "Best SOTA memory OS for any harness" (MEMD-SOTA-THEORY.md).
See docs/verification/0.1.0-CONTRACT.md for full release gate and proof-run evidence.
```

## 8. Cross-phase API surface summary

| Introduced in | Symbol / Path | Consumed by |
| --- | --- | --- |
| A13 | `memd_core::sync::crdt::*` | B13 (conflict resolution), E13 (workspace merge), G13 (harness) |
| A13 | `docs/contracts/sync-conflict-resolution.md` | G13 (proof harness sync scenarios) |
| C13 | `memory_item.correction_graph: Graph<ItemId, CorrectionEdge>` | D13 (query lineage), G13 (export) |
| D13 | `memd_core::routine::composition::*` | E13 (sharing), G13 (harness) |
| D13 | `.memd/logs/routine-suggestions.ndjson` | G13 |
| E13 | `memd_core::workspace::xws_sharing::*` | G13 (export + cross-workspace ingestion) |
| E13 | `.memd/state/shared-routines-manifest.json` | G13 (xws-share verification) |
| F13 | `memd_core::retrieval::domain_tuned::*` | G13 (public bench runs) |
| G13 | `memd_core::export::provenance_snapshot::*` | G13 (export + third-party replay) |
| G13 | `docs/verification/release-0-1-0/` | release commit (evidence directory) |

## 9. Open questions for next executor

Surface in phase kickoff notes; do not silently assume:

- Third-party replay harness language: Rust (same as memd) or Python (accessibility)? Decide before G13.G task design.
- Fixture anonymization for benches: do published bench datasets contain PII? Check LoCoMo, LongMemEval, MemBench, ConvoMem licensing + anonymization status.
- CI substrate for G13 full harness: requires parallel bench runs (4 benches simultaneously) or sequential (28+ hours per full harness run)? Confirm capacity at G13 task G13.1.
- Release documentation: does 0.1.0 land with `docs/RELEASE-0-1-0.md` user guide, or is that 0.2.0 scope? Check ROADMAP.md post-V13 handoff notes.

## 10. Changelog

- 2026-04-22 opened. V13 release harness integrates all 7 axes and 5 owned deltas. Full harness architecture: G13.A (dormant-project SC), G13.B (long-session compaction SC), G13.C (multi-hop CR), G13.D (routine composition PR), G13.E (xws sharing PR+TP), G13.F (public-bench RR), G13.G (export+replay TP). Proof-run directory: per-axis NDJSON + dated human review + margin targets table. Zero-generosity regenerator in strict mode with TE zero-margin abort trap. Contingency plan: if TE <7, file v13.5 recovery phase before 0.1.0 tag. Feature-flag graduation: 4 flags over 28 days (no block on tagging). Two plan-spec commits, release tag commit after G13 closes. All conditions from 0.1.0-CONTRACT.md embedded in milestone doc. Cross-phase API surface and open questions documented.
