---
status: superseded
milestone: M3
superseded_by: ROADMAP.md + individual phase docs
superseded_date: 2026-04-17
---

# M3 Execution Plan: Make It Provable

> This plan tells the next code session exactly what to do. No theory reading needed.
> No "figure out the architecture" needed. Just follow the steps.

## Context

M2 is verified (624 tests, 0 failures, benchmarks zero regression, node verification 15✓/6~/0✗).
The system stores, corrects, navigates, and routes by lane. M3 fixes **measurement and enforcement** —
the system is architecturally correct but scope/visibility aren't enforced, decay constants are
hardcoded guesses, consolidation quality is unmeasured, and token efficiency is untracked.

Three phases. 10-STAR gaps 18–23. 624 existing tests at session start.

## Root Cause

The system can't prove it works because:
1. Visibility is stored in `payload_json` (a JSON blob), not a SQL column — DB-level filtering
   is impossible. `build_context` checks scope but visibility enforcement is cosmetic.
2. Working memory has no per-agent isolation — agent A's working context is visible to agent B
   if they share the same project.
3. Consolidation at `routes.rs:1395` hardcodes all consolidated items to `Workspace` visibility
   regardless of source item visibility — private items become visible after consolidation.
4. Decay constants are hardcoded at `store.rs:634–635` (21d inactive threshold, 0.12 max_decay,
   14.0 divisor). Never calibrated with production data. No sensitivity analysis exists.
5. Consolidation generates content via format string (`helpers.rs:415–433`) with zero quality
   metrics — no coherence scoring, no information preservation check, no post-consolidation
   recall comparison.
6. No per-kind token tracking — we can't tell if facts cost more budget than status items.
   No per-operation token tracking (wake vs recall vs handoff vs working memory).
7. Benchmark suite exists but has no CI gate, no trend tracking (git SHA indexed), no
   automated regression detection.

## Dependency Graph

```
M2 (verified) ──┬── J2 (Isolation + Trust)      ──┐
                 │                                  │
                 └── O2 (Lifecycle Calibration)  ──┼── P2 (Measurement Proof)
                                                    │
```

J2 and O2 can start in parallel — both depend only on M2.
P2 depends on J2 + O2 (measurement requires enforcement + calibration in place first).

## Execution Order

Steps 1–2 (J2, O2) can execute in any order — all depend only on M2.
Step 3 (P2) blocks on J2 + O2. Recommended serial order for a single-developer session:
J2 first (enforcement must exist before measurement), then O2 (calibration informs measurement
baselines), then P2 (measure everything).

### Step 1: J2 — Isolation + Trust

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-j2-isolation-trust.md]]
**Depends on**: D2 (verified), G2 (verified)
**Fixes**: gaps 20, 23
**Backlog items**: #47, #48, #68, #73

**What already exists** (verify, don't rebuild):

- `MemoryScope` enum at `crates/memd-schema/src/lib.rs:23–28` — Local, Synced, Project, Global.
  Schema field exists on all items. Works.
- `MemoryVisibility` enum at `crates/memd-schema/src/lib.rs:87–92` — Private, Workspace, Public.
  Schema field exists. Works.
- `build_context()` at `crates/memd-server/src/helpers.rs:27–113` — scope filter at line 43,
  visibility filter at lines 54–56. Checks scope. Visibility check exists but reads from
  deserialized payload_json, not a SQL column — can't filter at DB level.
- Agent identity scoring at `crates/memd-server/src/helpers.rs:713–717` — gives +0.75 boost
  to items matching `source_agent`. This is scoring, NOT access control. Agents can still
  see each other's items.
- `trust_rank()` at `crates/memd-server/src/store_entities.rs:279–289` — 5-level hierarchy:
  0=Synthetic, 1=Candidate, 2=Promoted, 3=Canonical, 4=HumanCorrection. Already wired into
  `working_item_priority`. Works.
- `matches_requested_project()` at `crates/memd-server/src/helpers.rs:916–928` — project
  isolation at retrieval. Items filtered by project match. Works.
- Working memory compiler at `crates/memd-server/src/working/mod.rs:15–272` — no per-agent
  isolation. All items from the project go into one working context.
- Consolidation at `crates/memd-server/src/routes.rs:1315–1502` — template-based content
  generation. Line 1395 hardcodes `MemoryVisibility::Workspace` on all consolidated items.
- Handoff packet builder — builds resume/handoff without quality scoring.
- `memory_policy_snapshot()` at `crates/memd-server/src/working/mod.rs:717–826` — all defaults,
  no per-project policy override.

**What to build**:

1. **Extract visibility to SQL column** — visibility is buried in `payload_json` (a TEXT column
   holding serialized JSON). This makes DB-level filtering impossible — every query deserializes
   the full payload to check visibility. Add a `visibility` column to `memory_items` table.
   Migration: `ALTER TABLE memory_items ADD COLUMN visibility TEXT NOT NULL DEFAULT 'workspace'`.
   Backfill: parse `payload_json` for existing items, write `visibility` value to new column.
   Update `store_item` to write both `payload_json` AND the new column.
   - File: `crates/memd-server/src/store_migrations.rs` (new migration)
   - File: `crates/memd-server/src/store.rs:93–107` (schema — add column)
   - File: `crates/memd-server/src/store.rs` (store_item — write visibility column)
   - Donor: J2-D3 (Omegon minds namespace — SQL-level scoping)

2. **Visibility enforcement on retrieval** — `build_context()` must filter by visibility at
   the SQL level, not after deserialization. Add `WHERE visibility IN (?)` to all retrieval
   queries. Rules: Private items → only the storing agent. Workspace → all agents in project.
   Public → all agents everywhere. The `source_agent` field on items identifies the owner.
   - File: `crates/memd-server/src/helpers.rs:27–113` (build_context — add SQL visibility filter)
   - File: `crates/memd-server/src/helpers.rs` (all other retrieval paths — working memory, wake)

3. **Per-agent working context isolation** — working memory compiler builds one context per
   project. Add agent-scoped partitioning: agent A's private items in A's working context only.
   Shared (Workspace/Public) items visible to all agents. Not a separate table — filter at
   admission time using the new visibility column + `source_agent` match.
   - File: `crates/memd-server/src/working/mod.rs:15–272` (add agent_id parameter, filter admission)

4. **Fix consolidation visibility preservation** — `consolidate_semantic_memory()` at
   `routes.rs:1395` hardcodes `MemoryVisibility::Workspace`. Fix: consolidated item inherits
   the MOST RESTRICTIVE visibility from its source items. If any source is Private, the
   consolidated item is Private. If all sources are Public, consolidated is Public.
   Otherwise Workspace.
   - File: `crates/memd-server/src/routes.rs:1395` (replace hardcoded Workspace)
   - BUG: this is a data leak — private items become Workspace-visible after consolidation

5. **Adversarial visibility test** — E2E test:
   (a) agent A stores Private item,
   (b) agent B queries same project,
   (c) assert B cannot retrieve A's Private item,
   (d) agent A stores Workspace item,
   (e) assert B CAN retrieve A's Workspace item.
   - File: `crates/memd-server/src/tests/mod.rs` (new test)
   - Donor: J2-D4 (Omegon secrets redaction — adversarial leak testing pattern)

6. **Multi-project isolation proof** — E2E test:
   (a) store item in project X,
   (b) query from project Y context,
   (c) assert item NOT returned,
   (d) query from project X context,
   (e) assert item IS returned.
   `matches_requested_project` already exists — this test proves it works under adversarial conditions.
   - File: `crates/memd-server/src/tests/mod.rs` (new test)

7. **Compaction quality scoring** (gap 20) — after compaction (working memory admission/eviction),
   measure what was lost. Add a `compaction_quality_report` function: compare pre-compaction
   item set vs post-compaction item set. Metrics: information preservation ratio (how many
   unique facts survive), kind coverage (all 6 kinds represented?), visibility preservation
   (no Private→Workspace leaks). Log the report; expose via `/api/diagnostics`.
   - File: `crates/memd-server/src/working/mod.rs` (new function after admission)

8. **Handoff quality scoring** (gap 23) — when building a handoff packet, score completeness.
   Metrics: fact count, decision count, procedure count, correction chain intact, trust rank
   distribution. Compare handoff contents to full semantic memory — what percentage of
   canonical items made it into the handoff? Log the score; expose via handoff response metadata.
   - File: `crates/memd-client/src/runtime/resume/` (handoff quality scorer)
   - Donor: J2-D2 (Smriti WorkClaim — intent-based quality measurement pattern)

**Pass gate**:
- Adversarial: agent A's Private item invisible to agent B
- Multi-project: project X items never appear in project Y retrieval
- Trust: canonical item outranks candidate in working memory (already works — regression test)
- Per-agent: agent A's working context excludes B's Private items
- Compaction quality report generates for every admission cycle
- Handoff quality score generated with every handoff packet
- Consolidation preserves source visibility (no Private→Workspace leak)
- Visibility column exists in DB schema (SQL-level filtering)

**Verify nodes**: P1 (budget efficiency measured), P2 (continuity quality scored),
P4 (correction retention scored), M1 (adversarial over-capacity tests),
I3 (handoff quality scored)

---

### Step 2: O2 — Lifecycle Calibration

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-o2-lifecycle-calibration.md]]
**Depends on**: M2 (verified)
**Fixes**: gaps 21, 22

**What already exists** (verify, don't rebuild):

- `decay_entities()` at `crates/memd-server/src/store.rs:629–746` — applies exponential decay
  to entity salience scores. Constants hardcoded at lines 634–635:
  `inactive_threshold: 21` days, `max_decay: 0.12`, `divisor: 14.0`.
- `memory_policy_snapshot()` at `crates/memd-server/src/working/mod.rs:717–826` — returns
  `MemoryPolicyDecay` with default values. Struct has `inactive_days_threshold`,
  `max_decay_rate`, `decay_divisor` fields but they're never read by `decay_entities()`.
- `consolidate_semantic_memory()` at `crates/memd-server/src/routes.rs:1315–1502` — merges
  related items into consolidated summaries. Template-based content at `helpers.rs:415–433`.
  No quality metrics on the output.
- `gc_expired_items()` at `crates/memd-server/src/store_runtime_maintenance.rs:283–302` —
  removes expired items. No metrics on what was removed or why.
- `maintain_runtime()` at `crates/memd-server/src/store_runtime_maintenance.rs:68–190` —
  orchestrates maintenance: GC, decay, consolidation, promotion. Runs in worker loop
  at `crates/memd-worker/src/main.rs:44–69` (300s interval).

**What to build**:

1. **Make decay constants configurable via MemoryPolicyDecay** — `decay_entities()` ignores
   the policy struct and uses hardcoded values. Wire it up: read `inactive_days_threshold`,
   `max_decay_rate`, `decay_divisor` from the policy snapshot. Pass the policy to
   `decay_entities()` as a parameter. Default values remain 21/0.12/14.0 — same behavior,
   now configurable.
   - File: `crates/memd-server/src/store.rs:629–746` (accept policy param, use its values)
   - File: `crates/memd-server/src/working/mod.rs:717–826` (ensure defaults match current hardcodes)

2. **Decay metric collection** — add instrumentation to `decay_entities()`:
   track items inspected, items decayed, items expired, decay amounts applied,
   age distribution (buckets: <7d, 7-14d, 14-21d, 21-30d, >30d), confidence curve
   (pre-decay vs post-decay salience distribution). Persist metrics per run with timestamp.
   Expose via `/api/diagnostics/decay`.
   - File: `crates/memd-server/src/store.rs:629–746` (add metric collection)
   - File: `crates/memd-server/src/routes.rs` (new diagnostics endpoint)

3. **Decay sensitivity analysis framework** — add a `decay_sensitivity` test/tool that runs
   `decay_entities()` with 5 different parameter sets:
   (a) current defaults (21/0.12/14.0),
   (b) aggressive (14/0.20/7.0),
   (c) conservative (30/0.06/21.0),
   (d) fast-decay (7/0.25/5.0),
   (e) slow-decay (45/0.04/30.0).
   For each: measure items retained, items expired, recall quality (run a test query set
   before and after decay). Output a comparison table. This is a CLI tool, not production code.
   - File: `crates/memd-server/src/tests/mod.rs` (sensitivity test)
   - File: `crates/memd-client/src/cli/` (optional CLI command `memd decay-sensitivity`)

4. **Consolidation quality scoring** — after `consolidate_semantic_memory()` generates a
   consolidated item, score it. Metrics:
   (a) information preservation: count distinct facts in source items vs consolidated content
       (simple: count sentences/clauses),
   (b) semantic coherence: consolidated content should reference the same topic as sources
       (check entity overlap),
   (c) kind preservation: consolidated item should keep the most important kind from sources,
   (d) visibility preservation: consolidated item should inherit correct visibility (ties into J2.4).
   Persist quality scores with the consolidated item's metadata.
   - File: `crates/memd-server/src/routes.rs:1315–1502` (add scoring after consolidation)
   - File: `crates/memd-server/src/helpers.rs:415–433` (quality check on generated content)

5. **Post-consolidation recall comparison (A/B)** — test that proves consolidation doesn't
   degrade retrieval:
   (a) store 10 items on same topic,
   (b) run 5 test queries, record retrieval scores (pre-consolidation baseline),
   (c) run consolidation,
   (d) run same 5 queries again, record retrieval scores (post-consolidation),
   (e) assert post >= pre (consolidation should not degrade recall quality).
   - File: `crates/memd-server/src/tests/mod.rs` (new test)

6. **Calibrated defaults with data justification** — after running decay sensitivity analysis
   on real data, document the chosen constants and WHY they were chosen. If current defaults
   (21/0.12/14.0) are justified by the data, keep them. If not, update the defaults in
   `MemoryPolicyDecay`. Either way, the decision is documented with supporting data.
   - File: `docs/phases/phase-o2-lifecycle-calibration.md` (evidence section)
   - File: `crates/memd-server/src/working/mod.rs:717–826` (update defaults if needed)

**Pass gate**:
- Decay constants configurable via MemoryPolicyDecay, wired into decay_entities()
- Decay sensitivity: comparison table shows impact of 5 parameter sets on retention and recall
- Consolidation quality: semantic coherence test passes (consolidated item preserves original meaning)
- Consolidation recall: post-consolidation retrieval quality >= pre-consolidation
- Calibrated defaults documented with data justification
- Decay metrics collected and exposed via diagnostics endpoint

**Verify nodes**: P3 (promotion quality measured, decay calibrated),
M6 (candidate→canonical conversion rate), M7 (canonical quality scored)

---

### Step 3: P2 — Measurement Proof

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-p2-measurement-proof.md]]
**Depends on**: J2, O2
**Fixes**: gaps 18, 19

**What already exists** (verify, don't rebuild):

- Wake packet character budget at `crates/memd-client/src/runtime/resume/wakeup.rs:44–83` —
  1600 total chars, 220/item. Budget enforced. No per-kind breakdown.
- Working memory admission at `crates/memd-server/src/working/mod.rs` — 8-slot budget with
  kind-based quotas (B2). No token counting per kind.
- Benchmark suite at `crates/memd-client/src/benchmark/public_benchmark.rs` (3199 lines) —
  LongMemEval, LoCoMo, MemBench, ConvoMem. Runs manually via CLI. No CI integration, no
  trend tracking, no regression gate.
- Benchmark scorers at `crates/memd-client/src/benchmark/scorers.rs` — F1 scoring, tokenization,
  MC accuracy.
- Full-eval harness at `crates/memd-client/src/benchmark/full_eval.rs` — LLM-graded evaluation.
- Eval bundle at `crates/memd-client/src/evaluation/eval_report_runtime.rs` — `eval_bundle_memory`
  function.
- Benchmark registry at `docs/verification/benchmark-registry.json` — tracks benchmark runs.
- Public benchmarks doc at `docs/verification/PUBLIC_BENCHMARKS.md` — protocol and cadence.
- M0 baseline: LongMemEval 82.8%, LoCoMo 41.5%, MemBench 34.6%, ConvoMem 0.0%.
- M2 benchmark re-run: LME 82.8%, LoCoMo 41.5%, MemBench 34.6% — zero regression.

**What to build**:

1. **Per-kind token counter** — instrument wake packet assembly, working memory admission,
   recall, and handoff to count characters (or tokens if tokenizer available) consumed by
   each memory kind (Fact, Decision, Preference, Procedure, Status, Topology). Track:
   total chars per kind, items per kind, avg chars/item per kind. Expose via
   `/api/diagnostics/token-efficiency`.
   - File: `crates/memd-client/src/runtime/resume/wakeup.rs` (add per-kind counting in render)
   - File: `crates/memd-server/src/working/mod.rs` (add per-kind counting in admission)

2. **Per-operation token efficiency tracking** — for each operation (wake, recall, handoff,
   working memory rebuild), measure total chars consumed, items included, budget utilization
   percentage, and per-kind breakdown. Persist per-operation metrics with timestamp.
   Expose via `/api/diagnostics/token-efficiency`.
   - File: `crates/memd-client/src/runtime/resume/wakeup.rs` (wake operation metrics)
   - File: `crates/memd-server/src/working/mod.rs` (working memory operation metrics)
   - File: `crates/memd-client/src/runtime/resume/` (handoff/recall operation metrics)

3. **Automated benchmark CI gate** — create a script/command that runs all 4 benchmarks
   in CI-compatible mode (no interactive prompts, deterministic seed, timeout per benchmark).
   Exit code 1 if any benchmark drops below threshold:
   LongMemEval >= 80%, LoCoMo >= 41.5%, MemBench >= 30%.
   Wire into `memd benchmark --ci` or a dedicated `memd benchmark-gate` command.
   - File: `crates/memd-client/src/benchmark/public_benchmark.rs` (add CI mode)
   - File: `crates/memd-client/src/cli/` (CLI entry for benchmark gate)

4. **Benchmark trend tracking with git SHA** — after each benchmark run, persist results
   to `docs/verification/benchmark-registry.json` with: git SHA, timestamp, benchmark name,
   score, threshold, pass/fail. Add a `--record` flag to the benchmark command that auto-appends.
   Include a `memd benchmark-trend` command that outputs a comparison table: current vs M2
   baseline vs M0 baseline.
   - File: `crates/memd-client/src/benchmark/public_benchmark.rs` (add result persistence)
   - File: `docs/verification/benchmark-registry.json` (append results)

5. **Full-eval pipeline run** — run all 4 benchmarks with full-eval mode (LLM-graded accuracy)
   for the M3 gate. LongMemEval must use `session_recall_any@10` (same metric as M1's full-eval).
   Record results. This is not a code change — it's a verification run using existing infrastructure.
   - Protocol: [[docs/verification/PUBLIC_BENCHMARKS.md]]
   - File: existing `memd benchmark` commands

6. **Measurement report generator** — add a `memd diagnostics report` command that outputs
   a combined report: token efficiency (per-kind, per-operation), decay metrics, compaction
   quality scores, consolidation quality scores, handoff quality scores, benchmark results
   with trend. This is the M3 evidence artifact.
   - File: `crates/memd-client/src/cli/` (new diagnostics report command)

**Pass gate**:
- Token efficiency: per-kind counters reporting for all 6 memory kinds
- Token efficiency: per-operation metrics for wake, recall, handoff, working memory
- LongMemEval >= 80% (regression gate, currently 82.8%)
- All 4 benchmarks run with full-eval pipeline
- Benchmark results stored with git SHA for trend tracking
- Regression gate: any benchmark drop below threshold = exit code 1
- Measurement report generates with all quality dimensions

**Verify nodes**: S1 (token efficiency measured), S6 (briefing latency measured),
I1 (capture rate measured), I2 (spill latency measured)

## Node-by-Node M3 E2E Tests

Every node in the architecture graph must pass its M3 criterion. Phase gates (J2/O2/P2)
cover some nodes. The tests below cover ALL nodes. Run after all three phases pass.

### Ingest Layer

| Node | M3 Criterion | E2E Test |
|------|-------------|----------|
| I1 | Capture rate measured | Ingest 100 items → measure successful captures vs attempts. Report capture rate %. Assert >= 99% (no silent loss) |
| I2 | Spill latency measured | Fill spill buffer → run `memd maintain` → measure time from spill-full to spill-drained. Record latency in diagnostics |
| I3 | Handoff quality scored | Build handoff packet → score completeness (fact count, decision count, correction chains intact, trust rank distribution). Assert score logged and exposed |

### Control Plane

| Node | M3 Criterion | E2E Test |
|------|-------------|----------|
| P1 | Budget efficiency measured | Build working context → measure chars used / chars available. Report per-kind utilization. Assert fact+decision use >= 50% of budget (not dominated by status) |
| P2 | Continuity quality scored | Build continuity → score completeness: what/where/changed/next all populated, no ghost refs, fields resolve to live items. Assert quality score >= 0.8 |
| P3 | Promotion quality measured, decay calibrated | Run promotion → measure candidate→canonical conversion rate. Run decay with configurable constants → verify policy params used (not hardcoded). Assert conversion rate tracked |
| P4 | Correction retention scored | Store 10 corrections → query for each → measure how many return the corrected version (not original). Assert retention rate = 100% |

### Typed Memory

| Node | M3 Criterion | E2E Test |
|------|-------------|----------|
| M1 | Adversarial over-capacity tests | Insert 20 items (2.5x budget) → verify working memory holds 8, evicts 12 with reasons. Verify highest-value items survive. Assert no Private item leaks to wrong agent |
| M2 | Resume quality scored | Resume from cold state → score: architecture knowledge present without re-reading source docs, preferences persist, corrections intact. Assert quality score generated |
| M3 | Episodic retrieval quality scored | Store 20 episodic events → query by time range → measure precision/recall of time-based retrieval. Assert score logged |
| M4 | Cross-session stability proven | Store 10 facts in session 1 → close → open session 2 → query all 10 → assert all 10 retrievable. Repeat for 3 sessions. Assert 0 fact loss across sessions |
| M5 | Reuse rate measured | Store 5 procedures → run 10 queries that should trigger procedure detection → measure how many times stored procedures surface vs raw re-explanation. Assert reuse rate tracked |
| M6 | Candidate→canonical conversion rate | Store 20 candidates with varying confidence/rehearsal levels → run `memd maintain --mode full` → measure how many promoted vs expired. Assert conversion rate = strong signals promoted, weak expired |
| M7 | Canonical quality scored | For each canonical item, score: has provenance chain, has entity links, has lane tag, has trust rank >= 3. Assert quality report generated |

### Recall Surfaces

| Node | M3 Criterion | E2E Test |
|------|-------------|----------|
| S1 | Token efficiency measured | Build wake packet → measure total chars, per-kind chars, budget utilization %. Assert per-kind breakdown logged. Assert facts+decisions use >= 50% of wake budget |
| S2 | Navigation coverage scored | Generate atlas regions → measure: what % of canonical items are reachable via atlas navigation? Assert coverage score logged. Target >= 80% items reachable |
| S3 | Deep-dive quality scored | Pick 5 canonical items → `memd explain` each → score: has sources, has lifecycle events, has correction chain (if applicable), has entity links. Assert quality score >= 0.8 |
| S4 | Evidence completeness scored | For 5 canonical items, trace back to raw evidence → measure: raw spine exists, creation event recorded, all intermediate lifecycle events present. Assert completeness score logged |
| S5 | Sync quality scored | `memd obsidian compile` → verify vault has corrected items (not superseded). Count items in vault vs canonical items in DB. Assert sync coverage >= 90% |
| S6 | Briefing latency measured | Run `memd brief` 10 times → measure p50, p95, p99 latency. Assert latency recorded in diagnostics. Assert p95 < 500ms |

### Live Loop M3 Test

Run the loop and measure token cost + quality at each step:

1. Capture raw event → I1 → measure capture latency
2. Update working context → P1 → M1 → measure budget efficiency (chars used/available)
3. Update session continuity → P2 → M2 → measure continuity quality score
4. Write episodic record → M3 → measure episodic write latency
5. Repair semantic truth → P3 → M4 → measure promotion rate
6. Update procedural memory → P3 → M5 → measure procedure reuse rate
7. Compile wake packet → S1 → measure token efficiency (per-kind breakdown)

**M3 gate test**: Run the full loop 3 times. All 7 metrics must be collected and logged.
Token efficiency report shows per-kind breakdown. No measurement gaps.

### Calibration Loop M3 Test

1. Run `decay_entities()` with default policy → record metrics (items inspected/decayed/expired)
2. Run with aggressive policy (14/0.20/7.0) → record metrics
3. Run with conservative policy (30/0.06/21.0) → record metrics
4. Compare: aggressive should expire more items, conservative fewer
5. Run test queries after each → measure recall quality delta
6. Output sensitivity comparison table

**M3 gate test**: Sensitivity table shows measurable differences between parameter sets.
Chosen defaults justified by data (not gut feeling).

### Measurement Completeness Loop M3 Test

1. Run all 4 benchmarks in CI mode → all pass thresholds
2. Token efficiency report covers all 6 kinds and 4 operations
3. Decay metrics collected (items inspected/decayed/expired/age distribution)
4. Compaction quality report generated (information preservation ratio)
5. Consolidation quality report generated (coherence score, recall comparison)
6. Handoff quality report generated (completeness score, trust distribution)
7. Combined diagnostics report generates without errors

**M3 gate test**: `memd diagnostics report` outputs all 7 measurement dimensions.
No "N/A" or "not measured" in any dimension.

## What NOT To Do

- Do not touch M4 phases (K2/L2/M2-evo/N2/I2) — they depend on M3
- Do not improve retrieval quality — M3 MEASURES, it doesn't optimize. If measurement
  reveals a problem, log it as a backlog item for M4, don't fix it now.
- Do not rebuild the consolidation pipeline — measure its quality, don't redesign it.
  If quality is poor, that's a finding, not a blocker.
- Do not optimize benchmark numbers — the goal is measurement infrastructure, not score improvement.
  If LongMemEval drops below 80%, that's a regression to investigate, not a score to game.
- Do not add new memory kinds or operations — measure what exists.
- Do not change the decay constants WITHOUT data justification — calibration means using data
  to choose, not guessing different numbers.
- Do not skip the sensitivity analysis — picking new constants without comparing alternatives
  is the same mistake as the original hardcoded values.
- Do not merge M3 work until ALL three phases pass gates.

## Amnesia Prevention Checklist

After M3 passes, before declaring done:

- [ ] Agent A's Private item invisible to Agent B (adversarial test passes)
- [ ] Project X items never appear in project Y retrieval (isolation test passes)
- [ ] Consolidated items inherit source visibility (no Private→Workspace leak)
- [ ] Visibility column exists in DB schema (SQL-level filtering, not payload_json deserialization)
- [ ] Per-agent working context isolation works (agent A's context excludes B's Private items)
- [ ] Decay constants read from MemoryPolicyDecay, not hardcoded in store.rs
- [ ] Decay sensitivity comparison table exists with 5 parameter sets
- [ ] Chosen decay defaults documented with data justification
- [ ] Consolidation quality score generated on every consolidation run
- [ ] Post-consolidation recall quality >= pre-consolidation (A/B test passes)
- [ ] Token efficiency: per-kind counters for all 6 memory kinds
- [ ] Token efficiency: per-operation metrics for wake, recall, handoff, working memory
- [ ] LongMemEval >= 80% (regression gate)
- [ ] All 4 benchmarks run with full-eval pipeline, results recorded with git SHA
- [ ] `memd diagnostics report` outputs all measurement dimensions without gaps

If any of these fail, M3 is not done. Period.

## Key Files

| What | Where |
|------|-------|
| Scope/visibility enums | `crates/memd-schema/src/lib.rs:23–28, 87–92` |
| Build context (retrieval) | `crates/memd-server/src/helpers.rs:27–113` |
| Agent identity scoring | `crates/memd-server/src/helpers.rs:713–717` |
| Trust rank function | `crates/memd-server/src/store_entities.rs:279–289` |
| Project isolation | `crates/memd-server/src/helpers.rs:916–928` |
| Working memory compiler | `crates/memd-server/src/working/mod.rs:15–272` |
| Memory policy snapshot | `crates/memd-server/src/working/mod.rs:717–826` |
| Decay entities | `crates/memd-server/src/store.rs:629–746` |
| DB schema | `crates/memd-server/src/store.rs:93–107` |
| Consolidation | `crates/memd-server/src/routes.rs:1315–1502` |
| Consolidation visibility bug | `crates/memd-server/src/routes.rs:1395` |
| Consolidation content | `crates/memd-server/src/helpers.rs:415–433` |
| Store migrations | `crates/memd-server/src/store_migrations.rs` |
| Maintenance runtime | `crates/memd-server/src/store_runtime_maintenance.rs:68–190` |
| GC expired items | `crates/memd-server/src/store_runtime_maintenance.rs:283–302` |
| Worker loop | `crates/memd-worker/src/main.rs:44–69` |
| Wake packet compiler | `crates/memd-client/src/runtime/resume/wakeup.rs:44–83` |
| Benchmark suite | `crates/memd-client/src/benchmark/public_benchmark.rs` |
| Benchmark scorers | `crates/memd-client/src/benchmark/scorers.rs` |
| Full-eval harness | `crates/memd-client/src/benchmark/full_eval.rs` |
| Eval bundle | `crates/memd-client/src/evaluation/eval_report_runtime.rs` |
| Existing tests | `crates/memd-server/src/tests/mod.rs` |
| Benchmark registry | `docs/verification/benchmark-registry.json` |

## Phase Docs (read before starting each step)

- [[docs/phases/phase-j2-isolation-trust.md]]
- [[docs/phases/phase-o2-lifecycle-calibration.md]]
- [[docs/phases/phase-p2-measurement-proof.md]]
- [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- [[docs/verification/MEMD-10-STAR.md]]

## Theory Locks (read only if stuck on a design decision)

- [[memd-theory-lock-v1]] (J2, general architecture)
- [[memd-evaluation-theory-lock-v1]] (P2, benchmark methodology)
- [[memd-canonical-promotion-theory-lock-v1]] (O2, promotion/decay)
