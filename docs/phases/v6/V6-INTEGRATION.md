---
version: v6
kind: integration-plan
status: ready-to-execute
opened: 2026-04-22
scope: A6..F6
---

# V6 Integration — Cross-Phase Plan

> Read after all six `phase-{a6..f6}-plan.md` specs. Covers what no single phase owns: typed-ingest adapter layer architecture, shared per-bench fixtures, bench rerun matrix, method-card ritual, judge-cost governance, flag-graduation calendar, commit strategy, cross-phase API surface, V6 milestone exit criteria.

## 1. Execution-order discipline

Phase dependency (strict, linear — each lift builds on the prior):

```
A6 ──► B6 ──► C6 ──► D6 ──► E6 ──► F6
```

Rules:

- Every V6 phase is sequential. Parallel branches are not safe because each phase's baseline lift is measured vs the prior phase's baseline; reordering poisons the delta.
- A6 must fully close (including the 7-day flag graduation) before B6 opens.
- F6 is the gate phase — runs full canonical sweep, regenerates PUBLIC_BENCHMARKS, writes 10-STAR composite ≥7.0.

No phase may short-circuit its predecessor to hit its own pass gate. If blocked, file a backlog item + surface in handoff.

## 2. Typed-ingest adapter layer

Owner: A6. Surface mutated by every subsequent phase. Module tree after F6:

```
crates/memd-client/src/benchmark/
├── mod.rs                          # V5 substrate + V6 typed_ingest dispatcher
├── (V3/V5 files unchanged)
├── substrate/                      # V5 (closed)
└── typed_ingest/                   # NEW (V6)
    ├── mod.rs                      # A6 — flag parsing + dispatcher
    ├── bench_loaders/              # A6
    │   ├── lme.rs
    │   ├── locomo.rs
    │   ├── membench.rs
    │   └── convomem.rs
    ├── episodic.rs                 # A6 — episodic adapter
    ├── distiller.rs                # B6 — LLM extractor
    ├── dedupe.rs                   # B6
    ├── candidate_store.rs          # B6
    ├── promotion.rs                # C6
    ├── canonical_index.rs          # C6
    ├── compiler.rs                 # D6 — wraps V4 runtime::resume::compiler
    ├── depth_router.rs             # E6
    ├── depth_policy.rs             # E6
    ├── reasoning.rs                # F6
    ├── report_aggregator.rs        # F6
    └── star_regen.rs               # F6
```

Test mirror under `crates/memd-client/src/main_tests/typed_ingest_{a6..f6}_tests/mod.rs`.

## 3. Shared fixtures

| Fixture | Owner | Shared with |
| --- | --- | --- |
| `.memd/benchmarks/public/cache/distill/*.json` | B6 | C6 (candidate source), D6 (compiler input), E6, F6 |
| `.memd/benchmarks/public/compiler-budgets.yaml` | D6 | E6 (uses same budgets for depth calls), F6 |
| `.memd/benchmarks/public/fixtures/shared/canonical-identity-rules.json` | C6 | D6 (priority), F6 |
| `tests/fixtures/typed_ingest/shared/multihop-samples.jsonl` | E6 | F6 reasoning fixtures |

Consolidation pass: after D6 lands, C6 canonical index + D6 budgets move to `public/fixtures/shared/` if any sibling phase references.

## 4. Bench rerun matrix

Every V6 phase re-runs all four public benches at close. Matrix:

| Phase | LME | LoCoMo | MemBench | ConvoMem | Expected ingest flags |
| --- | --- | --- | --- | --- | --- |
| A6 close | ±1% | ±1% | ±1% | ±1% | `--typed-ingest=episodic` |
| B6 close | +0.02 | ≤1% regress ok | ≤1% regress ok | ≤1% regress ok | `--typed-ingest=episodic+semantic` |
| C6 close | +0.04 cum | ≤1% regress ok | +0.03 | +0.03 allowed-if | `…+canonical` |
| D6 close | +0.04 held; tokens –25% | +0.03 | +0.06 cum | held | `…+canonical --compiler=on` |
| E6 close | +0.07 cum | +0.07 cum | +0.06 held | +0.03 | `… --depth-routing=on` |
| F6 close | ≥0.85 canonical | ≥0.75 | ≥0.75 | ≥0.90 | full V6 + `--reasoning=on` |

Each phase's Task .6 (or .8 for F6) runs the matrix; deviations investigated, not tuned.

## 5. Method-card ritual

F6 Task F6.3 writes four method cards at `docs/verification/method-cards/{bench}-v6.md`. Per I3 rules, every card has:

```markdown
# <Bench> v6 Method Card

## Upstream scorer
- repo + commit
- exact invocation line

## memd ingest path
- flags: `--typed-ingest=…`
- budgets: link to compiler-budgets.yaml
- reasoning: on/off

## Seeds
- distill seed
- promotion seed
- reasoning seed

## Hardware/env
- cargo target: /tmp/memd-target
- judge: codex-lb gpt-5.4-mini at 127.0.0.1:2455
- rate-limit env: MEMD_RATE_LIMIT_DISABLED=1

## Canonical numbers
- metric: value (sidecar OFF)
- retrieval diagnostic: value

## Judge cost
- milli-USD per 1k turns

## Reproducibility
- `bash scripts/public-bench-reproduce.sh --bench <name>`
- tolerance: ±0.03
```

Rules:
- No forecasted numbers. Fill from the actual canonical run.
- Every non-memd number (competitor) has a primary-source link or stays empty.
- One card per bench. Never merged.
- Regenerate on every phase close; prior cards archived under `method-cards/archive/v6/`.

## 6. `PUBLIC_BENCHMARKS.md` regeneration ritual

F6 owns the regenerator. Rules mirror V5's SUBSTRATE regen but tighter:

- File path: `docs/verification/PUBLIC_BENCHMARKS.md`.
- Regenerator emits header with run-date, seed-base, memd commit hash.
- Per-bench section: current canonical number, delta vs V5 baseline, method-card link, NDJSON link.
- History section: one line per regeneration (date → LME/LoCoMo/MemBench/ConvoMem quad).
- Never hand-edited.
- A regen that would fail a canonical gate aborts and emits reason to stderr — does not write.

## 7. Judge-cost governance

B6, C6, F6 call codex-lb. Cost discipline (enforced in each plan's Task .7 CI step):

- Cache is canonical. Re-runs at same prompt_version cost zero.
- Per-bench budget ceiling per full run: LME ≤ 500 milli-USD; LoCoMo ≤ 300; MemBench ≤ 200; ConvoMem ≤ 400.
- If a run exceeds 1.5× ceiling, CI fails and surfaces the offending phase.
- A phase may only bump prompt_version if the prompt-card is re-committed and a cache-bust rationale is recorded in the commit message.

## 8. Feature-flag graduation calendar

Flag-flip ordering (each flip = own commit, each after a 7-day clean window):

1. `MEMD_V6_TYPED_INGEST = 1` (A6.9)
2. `MEMD_V6_DISTILL_CACHE = 1` (B6 — already default, confirm 7-day clean)
3. `MEMD_V6_PROMOTION_DRY_RUN = 0` (C6 — flip from any safety default)
4. `MEMD_V6_COMPILER = 1` (D6.7)
5. `MEMD_V6_DEPTH_ROUTING = 1` (E6.7)
6. `MEMD_V6_REASONING = 1` (F6.7)
7. `MEMD_V6_ALLOW_BELOW_TARGET = 0` — permanent; never flipped to 1 in main.

F6 runs with all flags at production defaults. Graduation rollback does not re-open V6.

## 9. Commit strategy

### Plan-spec land phase (this task)

Thirteen atomic commits on `research/mining`, one per file:

1. `docs(v6): phase-a6-episodic-ingest spec`
2. `docs(v6): phase-b6-semantic-distillation spec`
3. `docs(v6): phase-c6-canonical-promotion spec`
4. `docs(v6): phase-d6-compiler-on-bench spec`
5. `docs(v6): phase-e6-progressive-depth-routing spec`
6. `docs(v6): phase-f6-iterative-reasoning-harness spec`
7. `docs(v6): phase-a6-plan implementation spec`
8. `docs(v6): phase-b6-plan implementation spec`
9. `docs(v6): phase-c6-plan implementation spec`
10. `docs(v6): phase-d6-plan implementation spec`
11. `docs(v6): phase-e6-plan implementation spec`
12. `docs(v6): phase-f6-plan implementation spec`
13. `docs(v6): V6-INTEGRATION cross-phase plan`

### Execution commits per phase

Each phase plan has its own internal task list (A6 = 9, B6 = 7, C6 = 6, D6 = 7, E6 = 7, F6 = 8 → 44 execution commits across V6). Produced by future agents, not this plan-spec task.

### Handoff commit

One more commit after the 13 docs commits:

```
docs(handoff): V5+V6 plan specs landed, next agent executes A5
```

Content: `docs/handoff/YYYY-MM-DD-v5-v6-plan-spec-complete-next-execute.md`.

## 10. Cross-phase API surface summary

| Introduced in | Symbol / Path | Consumed by |
| --- | --- | --- |
| A6 | `typed_ingest::bench_loaders::*` | all V6 phases |
| A6 | `MemoryRecord{kind:Episodic}` provenance schema | B6, C6, D6, E6, F6 |
| A6 | `docs/contracts/public-bench-ingest.md` | B6 (prompt references) |
| B6 | `typed_ingest::distiller::*` + cache | C6 (candidate source), F6 (regen uses cache) |
| B6 | `docs/contracts/semantic-distillation.md` | C6 (rule engine references kinds) |
| C6 | `typed_ingest::promotion::*` + canonical index | D6 (priority input), E6 (canonical-only depth), F6 |
| C6 | `docs/contracts/canonical-promotion.md` | D6 (priority rationale), F6 |
| D6 | `typed_ingest::compiler::*` | E6 (depth-call output re-compiles), F6 |
| D6 | `.memd/benchmarks/public/compiler-budgets.yaml` | E6, F6 |
| E6 | `typed_ingest::depth_router::*` | F6 (reasoning chains depth calls) |
| E6 | `docs/contracts/bench-depth-routing.md` | F6 |
| F6 | `PUBLIC_BENCHMARKS.md` + method cards | V7 entry gate |
| F6 | `MEMD-10-STAR.md` composite ≥7.0 | V7 entry gate |
| F6 | `scripts/public-bench-reproduce.sh` | external reproducibility |

## 11. Open questions for next executor

- V5 F5 taxonomy card location + format: confirm before B6.1 so distiller emits valid kinds.
- V4 `runtime::resume::compiler` public surface: audit before D6.2 to know if shim adaptation is needed.
- V4 C4 correction-propagation reuse shape: confirm before C6.2 (contradiction check).
- codex-lb rate limits at 127.0.0.1:2455: test burst before B6.2 to size cache warm-up.
- `PUBLIC_BENCHMARKS.md` current structure: read before F6.3 to preserve any prior human-authored sections not owned by regenerator.

## 12. Exit criteria for V6 as a milestone

All six phase exit criteria met AND F6 exit criteria met AND:

- Canonical numbers (sidecar OFF):
  - LME `qa_accuracy` ≥ 0.85
  - LoCoMo `token_f1_avg` ≥ 0.75
  - MemBench `mc_accuracy` ≥ 0.75
  - ConvoMem LLM-judge `accuracy` ≥ 0.90
  - LME `session_recall_any@5` ≥ 0.95 (no regression)
- 10-STAR composite ≥ 7.0.
- All four method cards committed; PUBLIC_BENCHMARKS.md regenerated.
- `scripts/public-bench-reproduce.sh` passes on fresh clone (±0.03).
- `MILESTONE-v6.md` filled.
- `ROADMAP.md` V6 → closed, V7 → in progress.
- Judge-cost totals within per-bench ceilings.
- No open backlog items tagged `axis: raw_retrieval` or `axis: token_efficiency` at severity `blocker`.
- Handoff doc points at `docs/phases/v7/` (to be created in V7 plan-spec phase).
