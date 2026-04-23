---
version: v11
kind: integration-plan
status: ready-to-execute
opened: 2026-04-22
revised: 2026-04-22
scope: A11..G11 (outline only; phase-a11-plan.md etc. created by future executor)
depends_on: [../../verification/0.1.0-CONTRACT.md, ../../verification/0.1.0-AXIS-OWNERSHIP.md, ../../verification/milestones/MILESTONE-v11.md, ../../theory/MEMD-SOTA-THEORY.md, ../v10/V10-INTEGRATION.md]
---

# V11 Integration — Cross-Phase Plan

> Read after MILESTONE-v11.md. This doc covers what no single phase owns: shared fixtures, dynamic-compiler schema locks, the 3-project dogfood scenario G11 executes, the 10-STAR regeneration ritual, and the commit strategy for the plan-spec-land phase.

## 1. Execution-order discipline

Phase-level dependency (strict):

```
A11 ──► B11 ──┐
        │     │
        └──► D11 ──┐
             │      │
             └──► E11 ──┐
                  │      │
    C11 ────────┤
                 │
                 ▼
                 G11
```

Rules:

- A11 tasks 1–6 land first (project-isolation contract + metadata schema). B11 cannot start until A11 Task A11.5 (handoff contract doc) commits.
- D11 requires A11 project metadata (project_id foreign key). E11 requires D11 compiler (cost ledger).
- C11 can parallelize after A11; requires A11's metadata for silent-correction detection cross-project scope.
- G11 requires everything; executes 3-project scenario combining A11, B11, C11, D11, E11.

No phase may short-circuit a prior dependency to hit its own pass gate. If blocked, file a backlog item and surface in the next session's handoff.

## 2. Shared test fixtures

To avoid fixture drift across phases, the following live under a shared dir and are referenced by multiple phase plans:

| Fixture | Owner | Shared with |
| --- | --- | --- |
| `crates/memd-client/fixtures/shared/projects/3-project-scenario.jsonl` | G11 | A11 (project isolation), B11 (compaction survival), C11 (silent-correction cross-project) |
| `crates/memd-client/fixtures/shared/transcripts/silent-correction-triggers.jsonl` | C11 | G11 proof harness |
| `crates/memd-client/fixtures/shared/compiler/dynamic-depth-50-turn.jsonl` | D11 | E11 cost reporting, G11 token-count assertion |
| `crates/memd-client/fixtures/shared/compaction/heavy-post-project-switch.jsonl` | B11 | G11 recovery assertion |

Convention: each phase plan's `fixtures/<phase>/` dir contains **only** fixtures unique to that phase. Shared fixtures move to `fixtures/shared/` the moment a second phase references them, with a compat shim in the original dir (symlink or `pub use`).

First consolidation happens after A11 lands — harvest project-isolation fixtures into shared.

## 3. Schema locks (V11 compiler plumbing)

Three schema-level changes land in V11 to support dynamic compiler and project awareness. These are **structural**, not axis-credit-bearing. All three are A11 scope (metadata schema expansion).

### 3.1 Project ID foreign key + workspace isolation

Every `memory_items` row carries `project_id` (TEXT NOT NULL) and `workspace_id` (TEXT NOT NULL). Together they form the isolation boundary:
- Same project, same workspace, different harness → shared memory (CH requirement).
- Same workspace, different project → isolated memory (A11 SC requirement).
- Same project, different workspace → isolated memory (multi-workspace support for future).

Columns added (A11 Task A11.2 migration):
- `memory_items.project_id` TEXT NOT NULL (foreign key to projects table)
- `memory_items.workspace_id` TEXT NOT NULL

Rationale: enables project-aware wake without schema changes mid-V11. Donor: Letta session model + Supermemory workspace concept.

### 3.2 Compiler context + cost ledger schema

`compiler_context` table (new, A11 Task A11.3):
- `compiler_context_id` TEXT PRIMARY KEY
- `session_id` TEXT NOT NULL
- `turn_seq` INTEGER NOT NULL
- `intent_class` TEXT (e.g., "recall", "create", "refine", "synthesize")
- `target_token_budget` INTEGER
- `actual_tokens` INTEGER
- `depth_decision` TEXT (e.g., "immediate:4, procedural:2, background:0")
- `created_at` TIMESTAMP

`cost_ledger` table (new, E11 scope but schema lands in A11):
- `cost_ledger_id` TEXT PRIMARY KEY
- `turn_seq` INTEGER NOT NULL
- `project_id` TEXT NOT NULL
- `cost_cents` REAL
- `token_count` INTEGER
- `timestamp` TIMESTAMP

Rationale: cost tracking is post-hoc; ledger serves both per-turn compiler decisions (D11) and operator UI (E11). Donor: V4 D4 cost ledger concept, expanded.

### 3.3 Silent-correction detection state

`correction_flags` table (new, C11 scope but schema lands in A11):
- `correction_flag_id` TEXT PRIMARY KEY
- `memory_item_id` TEXT (the prior answer being flagged)
- `project_id` TEXT NOT NULL
- `rephrasing_count` INTEGER (how many times question was re-asked)
- `ignore_count` INTEGER (how many times suggestion was ignored)
- `flagged_at` TIMESTAMP
- `detection_latency_ms` INTEGER

Rationale: silent-correction detection needs persistent state across turns to count rephrasings and ignores. C11 reads and writes this table; G11 assertions verify latency ≤1000 ms.

### 3.4 Why V11 owns these

All three are metadata / schema expansions. A11 is the "project-aware wake" phase; plumbing lives where the schema migrations land. Later phases (B11, C11, D11, E11) consume the new columns without owning the migration, which keeps phase boundaries clean.

## 4. 3-project dogfood scenario G11 executes

Full canonical script; three projects run in sequence with compaction + silent-correction triggers.

### Session 1 — Project A (6 turns)

| Turn | Actor | Utterance | Phases exercised |
| --- | --- | --- | --- |
| T1 | user | "Focus: finish project A redesign" | A11 (project context set) |
| T2 | user | "Primary storage is PostgreSQL" | A11 + C11 (baseline fact) |
| T3 | agent | Read `A/schema.sql` | A11 (project-A ledger) |
| T4 | user | "No, I meant the cache is Redis." | C11 (correction on T2's prior claim) |
| T5 | user | "Summarize what you've learned about A." | D11 (dynamic compiler at recall intent) |
| T6 | system | SessionStop → PreCompact (compact post-T4 correction) | A11 + B11 |

Cut 1 assertions: project-A focus set, correction stored, compaction completes.

### Session 2 — Project B (6 turns — cross-project switch)

| Turn | Actor | Utterance | Phases |
| --- | --- | --- | --- |
| — | system | Wake for project B (empty session, different project) | A11: no project-A items visible |
| T7 | user | "Focus: debug project B API" | A11 (project-B context set) |
| T8 | user | "The API uses gRPC." | A11 + C11 |
| T9 | agent | Read `B/api.proto` | A11 (project-B ledger) |
| T10 | user | "What does the API use?" → `memd lookup --query "API protocol"` | D11 (lookup intent, immediate memory only) |
| T11 | user | "Correct, gRPC." (confirms prior answer) | C11 (no rephrasing, not flagged) |
| T12 | system | SessionStop → PreCompact | B11 (heavy compaction) |

Cut 2 assertions: project-B focus set, project-A items remain hidden, no false positives on C11 single-confirmation.

### Session 3 — Project A round-trip (8 turns — correction survival + rephrasing trigger)

| Turn | Actor | Utterance | Phases |
| --- | --- | --- | --- |
| — | system | Wake for project A (resume session 1 context) | A11: project-A focus restored, B items hidden, compaction-aware recall (B11) |
| T13 | user | "What did we store in the cache?" | D11 (recall intent, depth=immediate+procedural) |
| T14 | agent | Reads T4 correction: "Redis" | C11: correction from T4 (session 1) still active, **cross-session validity proven** |
| T15 | user | "What's the primary storage?" → `memd lookup` | D11 (lookup intent) |
| T16 | agent | Answers "PostgreSQL" | **C11 trigger**: this disagrees with T4 correction "Redis for cache" — potential confusion |
| T17 | user | "Wait, what's the cache backend?" (rephrase T13 from different angle) | C11: rephrasing detected (T13 rephrased as T17) |
| T18 | user | "Remind me the cache protocol." (another rephrase) | C11: **second rephrase triggers silent-correction flag** with latency ≤1000 ms surface-to-user |
| T19 | agent | Returns flag: "Prior T4 answer (Redis cache) flagged as potentially wrong on T17–T18 rephrasing pattern" | **C11 ≤1s latency assertion fires** |
| T20 | system | SessionStop | G11 close |

Cut 3 assertions:
- A11: project-A focus = session 1's focus (correcting project-B pollution).
- B11: compaction in T12 did not corrupt T4 correction; T14 reads it correctly.
- C11: silent-correction latency T17 → flag surface ≤1000 ms, T18 → confirmed latency ≤1000 ms.
- D11: turn-by-turn compiler decisions logged to `compiler_context` table; token counts per-turn stable.

### Scenario self-tests (harness fault-injection)

G11 Task G11.3 includes fault-injection variants: suppress A11 project isolation → wake includes B items (fail); drop C11 correction from T4 → T14 returns wrong value (fail); mute silent-correction detector → T17–T18 flags don't surface (fail). These are negative controls that prove the harness itself is honest.

## 5. 10-STAR scorecard regeneration template

G11 Task G11.4 writes to `docs/verification/MEMD-10-STAR.md`. The regenerator replaces the composite scorecard table in place and appends a dated evidence block.

Target table post-V11 (aligned to MILESTONE-v11.md axis contract):

```markdown
## 10-Star Composite Scorecard

| Axis | Weight | Score | Status |
|------|--------|-------|--------|
| Session continuity | 20% | 8/10 | A11 project-aware wake + B11 compaction-aware recall |
| Correction retention | 15% | 7/10 | C11 silent-correction detection ≤1s latency |
| Procedural reuse | 15% | 6/10 | unchanged — V12 owns PR +2 |
| Cross-harness continuity | 15% | 6/10 | unchanged — V12 owns CH +2 |
| Raw retrieval strength | 15% | 8/10 | unchanged — V13 owns RR +1 |
| Token efficiency | 10% | 7/10 | D11 dynamic compiler, E11 cost UI, F11 wake median ≤1500 tokens |
| Trust + provenance | 10% | 6/10 | unchanged — V12 owns TP +2 |

**Composite: 6.95 (V11 gate requirement) — regenerated YYYY-MM-DD by G11 harness run <id>**

Evidence: docs/verification/v11-proof-runs/YYYY-MM-DD.ndjson
```

Regeneration rules:
- Never score an axis higher than the harness evidence supports.
- If an axis has no V11 work, preserve its prior score verbatim.
- Always link to the proof-run NDJSON.
- Append a one-line delta history entry so prior scorecards are reconstructible.

## 6. Feature-flag graduation calendar

Dynamic compiler, project awareness, and silent-correction detection ship feature-flagged. Graduation order (each 7-day clean window):

1. `MEMD_A11_PROJECT_AWARE_WAKE` = 1 (Task A11.8)
2. `MEMD_D11_DYNAMIC_COMPILER` = 1 (Task D11.7)
3. `MEMD_C11_SILENT_CORRECTION_DETECT` = 1 (Task C11.6)

**Calendar spillover:** 3 graduations × 7 days = 21 days of post-G11 observation. V12 planning must account for the flag-ops work, but V12 phase A12 is **not** blocked on graduation completion — only on the handoff commit from V11's last code phase.

G11 runs with all flags at production defaults (on). A graduation rollback does not re-open V11 — file a recovery phase instead. If a flag flip surfaces a regression during the 7-day window, the recovery phase targets the specific axis that regressed and is named `v11-recovery-<axis>-<date>`.

## 7. Bench regression watch

V11 does not directly target public-bench numbers, but dynamic-compiler rewrites can affect LME / ConvoMem / MemBench. Mandatory checkpoints:

- Post-D11 Task D11.6: run canonical regression suite on LME, ConvoMem, MemBench, LoCoMo. Document delta in D11 plan's compiler-correctness-review doc.
- Post-E11 Task E11.5: same suite on cost-ledger changes.
- Post-G11 Task G11.6: full public + substrate bench sweep; publish in MILESTONE-v11.md.

If any public bench regresses >3% canonical score, hold the flag flip and root-cause.

## 8. Commit strategy

### Plan-spec land phase (this task)

Two atomic commits on `research/mining`:

1. `docs(v11): MILESTONE-v11 + V11-INTEGRATION cross-phase plan`

After that, one handoff commit:

2. `docs(handoff): V11 plan spec landed, next agent outlines phases A11–G11 (no specs yet)`

### Phase planning and outline

Future executor outlines phases A11–G11 in `docs/phases/v11/` (outline only, no implementation specs at milestone-land time). This happens before code execution starts.

### Execution commits per phase

Each phase plan (once created) has its own internal task list that commits per task. A11 = ~8 tasks, B11 = ~6, C11 = ~7, D11 = ~9, E11 = ~6, F11 = ~4, G11 = ~7. Those execution commits are produced by future agents — **not** by the plan-spec-land task. The spec-land task produces only the 2 docs commits + 1 handoff commit.

### Handoff commit

After the 1 plan-spec commit, one handoff commit:

```
docs(handoff): V11 plan spec landed, next agent outlines phases A11–G11
```

Content: new file `docs/handoff/YYYY-MM-DD-v11-plan-spec-complete-next-outline-phases.md`. Includes calendar note: 3 flag-graduation windows (21 days total) starting after G11 close, overlapping V12 planning.

## 9. Cross-phase API surface summary

| Introduced in | Symbol / Path | Consumed by |
| --- | --- | --- |
| A11 | `memory_items.project_id` + `workspace_id` columns | all (read filter) |
| A11 | `memd_core::isolation::ProjectScope` | B11 (compaction filter), C11 (detection scope), D11 (compiler decision context), G11 |
| A11 | `docs/contracts/project-isolation.md` | B11, C11, D11 (read handoff rules) |
| B11 | `memd_core::compaction::recovery::*` | G11 |
| B11 | `.memd/logs/compaction-recovery.ndjson` | G11 |
| C11 | `correction_flags` table + detection state | D11 (compiler priority), E11 (cost impact), G11 |
| C11 | `.memd/logs/silent-correction-detections.ndjson` | G11 |
| D11 | `compiler_context` table + intent-class routing | E11 (per-turn cost), F11 (token counts), G11 |
| D11 | `memd_core::runtime::resume::compiler_v2::*` | all (replaces compiler_v1) |
| D11 | `.memd/logs/compiler-decisions.ndjson` | G11 |
| E11 | `cost_ledger` table | operator CLI via `memd configure`, G11 |
| E11 | `.memd/logs/cost-ledger.ndjson` | G11 |
| F11 | `.memd/logs/wake-tokens.ndjson` | G11 |
| G11 | `docs/verification/v11-proof-runs/*.ndjson` | V12 entry gate |

## 10. Open questions for next executor

Surface these in TodoWrite or phase kickoff notes; do not silently assume:

- Project-scope enforcement strategy: all read paths need to filter by project_id? Or one central filter in wake flow? Investigate before A11 Task A11.1.
- Silent-correction latency measurement: wall-clock time from user input to flag surface, or turn-processing time? Clarify with G11 latency assertion before C11 Task C11.1.
- Intent classification for D11 compiler: use LLM classifier or rule-based? Cost/quality tradeoff — investigate token cost vs recall quality before D11 Task D11.1.
- Compaction recovery: how much state re-materialization is acceptable? Fast but lossy vs slow but perfect? Clarify budget before B11 Task B11.1.
- Cost tuning UI: `memd configure cost_target=0.5` (cents per turn)? Or per-session? Per-workspace? Clarify scope with E11 Task E11.1.

## 11. Schema + ordering locks (V11 compiler plumbing)

All three are A11 scope (metadata schema expansion, read-state plumbing). Details in section 3 above.

## 12. Exit criteria for V11 as a milestone

All seven phase exit criteria met AND G11 exit criteria met AND:

- 10-STAR composite ≥ 6.95 written to `docs/verification/MEMD-10-STAR.md` by the G11 scorecard regenerator.
- No axis score above the targets in MILESTONE-v11.md (regenerator fails loud on over-claim).
- `docs/verification/milestones/MILESTONE-v11.md` filled in with evidence paths.
- `ROADMAP.md` V11 → closed, V12 → in progress.
- No open backlog items tagged `axis: session_continuity`, `axis: correction_retention`, or `axis: token_efficiency` at severity `blocker`.
- 3-project scenario passing (project A → project B → project A round-trip with compaction + silent-correction triggers).
- Silent-correction detection latency ≤1000 ms proven in G11 harness.
- Dynamic compiler correctness proven (per-turn depth decisions logged, token counts stable within budget).
- Project isolation proven (wake does not leak items between projects in same workspace).
- Compaction-aware recall proven (post-compaction wake recovers full context without truncation).
- Schema locks A11.1–A11.3 landed and covered by unit tests.
- Feature-flag calendar set for V12 planning window (3 × 7-day graduations starting post-G11).
- Cost ledger operational via `memd configure` (E11 surface).
- Final handoff doc points at `docs/phases/v12/` (to be created in the V12 plan-spec phase).

## 13. Changelog

- 2026-04-22 initial spec. V11 is the first SOTA-push milestone after V10 production-floor close. Composite 6.40 → 6.95. SC +1 (project awareness), CR +1 (silent-correction detection ≤1s), TE +2 (dynamic compiler). Phase letters a11-g11 outlined (outline only, no implementation specs created in milestone-land phase). Non-goals explicit (PR, CH, RR, TP). Feature-flag calendar: 3 graduations over 21 days, spillover into V12 planning window. Shared fixtures consolidated in section 2. Schema locks detailed in section 3 (project ID, compiler context + cost ledger, silent-correction state). 3-project dogfood scenario documented in section 4 with fault-injection negatives. Composite math verified at 6.95 exactly. Theory alignment explicit.
