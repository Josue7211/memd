---
phase: D4
name: Working-Context Compiler
version: v4
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [B4]
phase_doc: docs/phases/v4/phase-d4-working-context-compiler.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: token_efficiency
axis_delta_target: "+3 (1 → 4) with zero continuity regression"
---

# Phase D4 — Implementation Plan

> Depends on B4 (trace). Reads C4 corrections as a priority bucket once C4 Task C4.1 lands (schema). Can scaffold in parallel against placeholder correction kind.

## 0. Executive summary

`memd wake` currently renders via `crates/memd-client/src/runtime/resume/wakeup.rs` (~1200 LOC: `render_bundle_wakeup_markdown`, `render_preferences_block`, `compute_wake_token_metrics`, `wake_budget_agent_name`). The existing render is a near-linear dump of retrieved sections. D4 inserts a **compiler** between retrieval and render: it takes typed buckets, applies priority, dedupes across buckets, enforces a hard token budget, and demotes overflow to a `memd lookup` hint.

No storage changes. No new hooks. Compiler is a pure transform.

---

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/runtime/resume/compiler/mod.rs` | `compile_wake(buckets, budget) -> CompiledWake` pure-function entry. |
| `crates/memd-client/src/runtime/resume/compiler/buckets.rs` | Typed buckets: `CanonicalBucket`, `PreferenceBucket`, `FocusBucket`, `EpisodicBucket`, `SemanticBucket`, `CorrectionBucket`, `CandidateBucket`. |
| `crates/memd-client/src/runtime/resume/compiler/priority.rs` | Priority rule implementation per phase-doc §2. |
| `crates/memd-client/src/runtime/resume/compiler/dedupe.rs` | Cross-bucket dedupe on canonicalized content hash + provenance merge. |
| `crates/memd-client/src/runtime/resume/compiler/budget.rs` | Token counter + overflow-demotion logic. |
| `crates/memd-client/src/runtime/resume/compiler/render.rs` | Markdown section emitter: sections in priority order, demotion hints. |
| `crates/memd-client/src/runtime/resume/compiler/tests.rs` | Unit tests. |
| `crates/memd-client/src/main_tests/wake_continuity_tests/mod.rs` | Continuity-loss scenario runner: 20 scripted pre/post comparisons. |
| `crates/memd-client/fixtures/d4/` | 20 pre-recorded session states + expected-answer assertions. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/runtime/resume/wakeup.rs` | `render_bundle_wakeup_markdown` gains a branch: when `MEMD_D4_COMPILER=1`, route through `compiler::compile_wake`. Existing render kept as `--raw` fallback. |
| `crates/memd-client/src/cli/args.rs` | Extend `WakeArgs` (existing ~line 3056) with `--raw` bool + `--budget-tokens <N>` override. |
| `docs/phases/v4/phase-d4-working-context-compiler.md` | Add `plan_spec:` line. |

### Crates affected

- `memd-client` only.
- No `memd-core`, no `memd-schema`, no `memd-server` change.

---

## 2. Schema changes

None. D4 is a pure transform over already-typed records.

### One logging artifact

`.memd/logs/wake-budget.ndjson`:

```json
{"ts_ms":…,"session_id":"…","raw_tokens":4820,"compiled_tokens":1743,"bucket_sizes":{"canonical":4,"preference":3,"focus":1,"episodic":3,"semantic":12,"correction":0,"candidate":2},"demoted":{"semantic":5}}
```

One line per wake call. Used for the pre/post histogram.

---

## 3. API shape

### Extended `memd wake`

```
memd wake \
  [--output .memd] \
  [--raw]                          # bypass compiler, return raw render
  [--budget-tokens <N>]            # override MEMD_WAKE_BUDGET_TOKENS
  [--include-bucket <name>]…       # force-include overflow bucket
  [--exclude-bucket <name>]…       # force-exclude a bucket
```

### Compiler public entry

```rust
pub fn compile_wake(
    input: CompilerInput,
    budget: WakeBudget,
) -> CompiledWake;

pub struct CompilerInput {
    pub canonical: Vec<MemoryRecord>,
    pub preferences: Vec<MemoryRecord>,
    pub focus: Vec<MemoryRecord>,
    pub episodic: Vec<MemoryRecord>,
    pub semantic: Vec<MemoryRecord>,
    pub corrections: Vec<MemoryRecord>,
    pub candidates: Vec<MemoryRecord>,
}

pub struct WakeBudget {
    pub tokens: usize,          // default 2000
    pub per_bucket_floor: HashMap<BucketKind, usize>, // canonical: 4, prefs: 3…
}

pub struct CompiledWake {
    pub markdown: String,
    pub tokens: usize,
    pub bucket_report: HashMap<BucketKind, BucketReport>,
    pub demotion_hints: Vec<DemotionHint>,
}
```

---

## 4. Test matrix

### Unit (runtime/resume/compiler/tests.rs)

1. `priority_order_canonical_first`
2. `priority_order_preferences_after_canonical_before_focus`
3. `priority_order_corrections_before_semantic` — corrections trump plain facts on equal topic.
4. `dedupe_merges_same_content_across_buckets_with_provenance`
5. `dedupe_preserves_highest_priority_source`
6. `budget_enforces_hard_cap`
7. `budget_respects_per_bucket_floor` — canonical never demoted below floor.
8. `budget_demotes_overflow_to_lookup_hint`
9. `render_emits_section_headers_in_priority_order`
10. `render_includes_demotion_hint_section_when_overflow`
11. `render_is_markdown_and_round_trips_token_count`
12. `token_counter_matches_compute_wake_token_metrics`

### Integration

13. `cli_wake_compiler_on_produces_under_budget_on_fat_fixture`
14. `cli_wake_raw_matches_pre_d4_render`
15. `cli_wake_budget_override_respected`
16. `cli_wake_include_bucket_forces_inclusion_even_over_budget` — documented tradeoff.
17. `compiler_writes_wake_budget_ndjson_line`

### Continuity-loss scenarios (wake_continuity_tests)

18. `continuity_loss_20_scenarios_pass` — each scenario: load prerecorded state, run both `--raw` and compiled wake, run a list of queries against both, assert compiled answers ⊇ raw answers on the tested dimensions: "what was I doing", "what did I learn", "what does user prefer". Pass = 20/20.
19. `continuity_loss_regression_catch` — plant a deliberate bug in priority rules, test 18 catches it.

### Bench-sized sanity

20. `wake_size_histogram_pre_post_on_fixture_set` — runs over 20 fixtures, asserts median compiled tokens ≤ 2000, p95 ≤ 2500.

### Rebuild + smoke

```
cargo build --release --target-dir /tmp/memd-target -p memd-client
cargo test --target-dir /tmp/memd-target -p memd-client compiler::
cargo test --target-dir /tmp/memd-target -p memd-client wake_continuity
```

---

## 5. Fixtures

`crates/memd-client/fixtures/d4/`:

| Dir | Contents |
| --- | --- |
| `state/session-<N>/` (×20) | Pre-recorded memd state snapshot: canonical records JSON, preferences JSON, focus, episodic, semantic, candidates. No secrets. |
| `queries.jsonl` | 3 queries per scenario: `doing`, `learned`, `prefers`. |
| `expected-answers.jsonl` | Expected substring match per query (human-authored). |
| `fat-fixture.json` | One oversized single-session state used by test 13. |

Regen procedure: anonymize a real dogfood session via `scripts/dev/anonymize-session.sh` (create if missing — Task D4.7 bundles this).

---

## 6. Telemetry

| Signal | Path |
| --- | --- |
| Wake budget + demotion | `.memd/logs/wake-budget.ndjson` |
| Counters | `memd_wake_tokens_bucket`, `memd_wake_demotion_total{bucket}` — log-line only. |

---

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_D4_COMPILER` | `0` → `1` post continuity test | Route wake through compiler. |
| `MEMD_WAKE_BUDGET_TOKENS` | `2000` | Hard cap. |
| `MEMD_WAKE_PER_BUCKET_FLOOR` | `"canonical=4,preferences=3,focus=1"` | Minimum entries per priority bucket. |

---

## 8. Task list (executable)

### Task D4.1 — scaffold compiler module + bucket types

- [ ] Read `crates/memd-client/src/runtime/resume/wakeup.rs` to inventory existing render helpers.
- [ ] Create module tree under `runtime/resume/compiler/`.
- [ ] Define `BucketKind` enum, `CompilerInput`, `CompiledWake`, `WakeBudget`.
- [ ] Compiles; no tests yet.
- [ ] Commit: `scaffold(memd-client/runtime/compiler): bucket types (D4)`.

### Task D4.2 — priority rules

- [ ] Tests 1–3 failing.
- [ ] Implement `priority::apply(input) -> OrderedBuckets`.
- [ ] Green.
- [ ] Commit: `feat(memd-client/compiler): priority rules (D4)`.

### Task D4.3 — dedupe

- [ ] Tests 4 + 5 failing.
- [ ] Implement content-hash dedupe with provenance merge.
- [ ] Green.
- [ ] Commit: `feat(memd-client/compiler): cross-bucket dedupe (D4)`.

### Task D4.4 — budget + demotion

- [ ] Tests 6–8 + 12 failing.
- [ ] Implement token counter reusing `compute_wake_token_metrics` if stable; else reimplement with identical tokenization.
- [ ] Per-bucket floor logic.
- [ ] Demotion hint generator.
- [ ] Green.
- [ ] Commit: `feat(memd-client/compiler): budget + demotion (D4)`.

### Task D4.5 — render

- [ ] Tests 9–11 failing.
- [ ] Implement markdown emission.
- [ ] Green.
- [ ] Commit: `feat(memd-client/compiler): markdown render (D4)`.

### Task D4.6 — CLI wiring

- [ ] Extend `WakeArgs`.
- [ ] Tests 13–17 failing.
- [ ] Wire `--raw` fallback to existing `render_bundle_wakeup_markdown`, default branch to compiler when `MEMD_D4_COMPILER=1`.
- [ ] Write `wake-budget.ndjson` line on every call.
- [ ] Green.
- [ ] Commit: `feat(memd-client/wake): compiler route + --raw fallback (D4)`.

### Task D4.7 — fixtures + continuity-loss harness

- [ ] Build `scripts/dev/anonymize-session.sh` (one-shot anonymizer — emails, tokens, absolute paths).
- [ ] Anonymize 20 dogfood sessions into `fixtures/d4/state/session-<N>/`.
- [ ] Author `queries.jsonl` + `expected-answers.jsonl`.
- [ ] Tests 18 + 19 + 20 failing.
- [ ] Run harness: assert 20/20 pass and histogram targets hit.
- [ ] Green.
- [ ] Commit: `test(memd-client): wake continuity-loss 20-scenario harness (D4)`.

### Task D4.8 — 7-day dogfood + graduate

- [ ] Enable `MEMD_D4_COMPILER=1` locally.
- [ ] Collect 7d of `wake-budget.ndjson`.
- [ ] Assert median ≤ 2000 tokens; confirm no user-facing continuity complaints.
- [ ] Flip default to `1`.
- [ ] Commit: `feat(d4): default MEMD_D4_COMPILER=1 after dogfood`.

### Task D4.9 — 10-STAR rescoring

- [ ] Bump token_efficiency axis in `MEMD-10-STAR.md` with evidence pointer.
- [ ] Commit: `docs(10-star): token_efficiency rescored after D4`.

---

## 9. Bench impact

- **V5 D5 (Token-Efficiency Bench).** Unblocks the bench's measurement premise (compiled wake is now the canonical input).
- **V6 public benches.** Generator receives tighter wake → fewer distractors → lift on LME / ConvoMem memory-update turns. Run canonical regression after Task D4.8.
- Public-bench regression watch: raw retrieval axis must not drop. D4 only changes presentation, not retrieval set — if it drops, the compiler is stripping returned records.

---

## 10. Dependency graph

- Requires: B4 trace infra (for wake-budget log). C4 Task C4.1 (Correction MemoryKind) — D4 compiler has a CorrectionBucket. If C4.1 not yet landed, bucket is a no-op placeholder.
- Blocks: E4 progressive depth (lookup/resume depths build on compiled wake contract), F4 preference drift (D4 renders "Preferences" section F4 validates against).
- Parallelizable with A4 (no code overlap), with C4 past Task C4.1.

## Exit criteria

1. Tests 1–20 green 10/10.
2. Median compiled wake ≤ 2000 tokens on 20-fixture set.
3. Continuity-loss 20/20 pass.
4. 7-day dogfood clean.
5. `MEMD_D4_COMPILER=1` default.
6. 10-STAR token_efficiency rescored.
7. Atomic commits on `research/mining`.
