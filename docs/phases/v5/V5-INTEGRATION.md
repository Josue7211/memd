---
version: v5
kind: integration-plan
status: ready-to-execute
opened: 2026-04-22
revised: 2026-04-22
scope: A5..G5
depends_on: [../../verification/0.1.0-CONTRACT.md, ../../verification/milestones/MILESTONE-v5.md]
---

# V5 Integration — Cross-Phase Plan

> Read after all seven `phase-{a5..g5}-plan.md` specs. This doc covers what no single phase plan owns: suite-wide module layout, shared fixtures, benchmark runner architecture, CI publishing flow, `SUBSTRATE_BENCHMARKS.md` regeneration ritual, competitor-card template policy, schema/ordering locks landed in V5, flag-graduation calendar, scorecard regenerator strict-mode check, commit strategy, cross-phase API surface, and V5 milestone exit criteria.

## 1. Execution-order discipline

Phase-level dependency:

```
A5 (runtime) ─┬─► B5
              ├─► C5
              ├─► D5
              ├─► E5
              └─► F5
                       │
                       ▼
                       G5 (gate + aggregator)
```

Rules:

- A5 Tasks A5.1–A5.4 land the shared substrate module tree (`benchmark/substrate/mod.rs`, `session_driver.rs`, scorer base, CLI dispatcher). No sibling suite phase may start a PR before A5.4 commits.
- B5–F5 parallelize after A5.4. Each of B5/C5/D5/E5/F5 is a separate agent session; no phase touches another's runner file.
- G5 is strictly sequential — requires all five sibling suites plus A5 closed.
- V5 milestone closes when G5 Task G5.7 writes `MILESTONE-v5.md` and flips ROADMAP.

No phase may short-circuit a prior dependency to hit its own pass gate. If blocked, file a backlog item and surface in the next session's handoff.

## 2. Substrate module tree (post-V5)

Owner of layout: A5. Every sibling phase adds exactly one suite file + its tests under the same parent.

```
crates/memd-client/src/benchmark/
├── mod.rs                          # pre-V5: public-bench dispatcher; V5 adds `substrate` child
├── baselines.rs                    # unchanged
├── full_eval.rs                    # unchanged
├── public_benchmark.rs             # unchanged
├── runtime.rs                      # unchanged
├── scorers.rs                      # unchanged
└── substrate/                      # NEW (A5)
    ├── mod.rs                      # dispatcher: --suite <name> | --all
    ├── session_driver.rs           # A5 — shared scripted-session engine
    ├── scorers.rs                  # A5 — recall@K, type-correctness, completeness, leak-count, truth-conservation
    ├── report.rs                   # A5 — NDJSON emitter + markdown section renderer
    ├── cross_session_recall.rs     # A5 runner
    ├── correction_propagation.rs   # B5
    ├── cross_harness.rs            # C5
    ├── harness_adapter/            # C5
    │   ├── mod.rs
    │   ├── claude_code.rs
    │   └── codex.rs
    ├── progressive_depth.rs        # D5
    ├── provenance_integrity.rs     # E5
    ├── provenance_auditor.rs       # E5
    ├── typed_retrieval.rs          # F5
    ├── adversarial_noise.rs        # G5
    └── aggregator.rs               # G5 — suite-of-suites runner + markdown regenerator
```

Test mirror under `crates/memd-client/src/main_tests/substrate_{a5..g5}_tests/mod.rs`.

Fixture root: `.memd/benchmarks/substrate/fixtures/{a5..g5}/` plus `.memd/benchmarks/substrate/fixtures/shared/` (see §3).

## 3. Shared fixtures

To prevent drift across seven suites:

| Fixture | Owner | Shared with |
| --- | --- | --- |
| `.memd/benchmarks/substrate/fixtures/shared/taxonomy-card.json` | F5 | A5, B5, D5, E5, G5 (every suite needs kind enumeration) |
| `.memd/benchmarks/substrate/fixtures/shared/sessions/session-30t.jsonl` | A5 | B5 (correction turns overlay), D5 (depth queries) |
| `.memd/benchmarks/substrate/fixtures/shared/harness-scripts/write.memd.sh` | C5 | G5 (adversarial noise seeded via same harness driver) |
| `.memd/benchmarks/substrate/fixtures/shared/provenance-template.json` | E5 | B5, G5 (provenance chain audits) |

Convention: each phase plan's `fixtures/<phase>/` dir contains **only** fixtures unique to that phase. Shared fixtures move to `fixtures/shared/` the moment a second phase references them.

Consolidation pass: after F5 lands, A5/B5/D5/E5/G5 replace local taxonomy copies with the shared card via symlink (Linux) or `include_bytes!` at test compile time.

## 4. Bench runner architecture

A5 owns the shared skeleton. Contract every suite runner honors:

```rust
pub trait SubstrateSuite {
    fn name(&self) -> &'static str;          // "cross-session-recall", "correction-propagation", …
    fn spec_path(&self) -> &'static str;     // .memd/benchmarks/substrate/<name>.yaml
    fn run(&self, ctx: &SubstrateCtx) -> Result<SuiteOutcome>;
}

pub struct SuiteOutcome {
    pub metrics: BTreeMap<String, f64>,
    pub pass_gate_hit: bool,
    pub ndjson_events: Vec<JsonValue>,
    pub markdown_section: String,
}
```

`SubstrateCtx` holds: seed, fixture-root, output-dir, harness availability map, timeout budget, feature flags.

CLI entry (A5 Task A5.3):

```
memd bench substrate --suite <name> [--seed N] [--output DIR]
memd bench substrate --all         [--seed-base N] [--fail-fast] [--regenerate-report] [--regenerate-10star]
memd bench substrate --list
```

The dispatcher in `substrate/mod.rs` registers suites via `inventory::submit!` so later phases add one line.

## 5. `SUBSTRATE_BENCHMARKS.md` regeneration ritual

G5 Task G5.3 writes the aggregator + markdown regenerator. File lives at `docs/verification/SUBSTRATE_BENCHMARKS.md` and is rewritten in place per `--all` run.

Structure emitted by regenerator:

```markdown
# memd Substrate Benchmarks

> Regenerated YYYY-MM-DD by `memd bench substrate --all` (run <id>). Do not hand-edit.

## Composite
…score table…

## Suites
### Cross-Session Recall (A5)
…metrics + pass/fail…
### Correction Propagation (B5)
…
### Cross-Harness Continuity (C5)
…
### Progressive Depth (D5)
…
### Provenance Integrity (E5)
…
### Typed Retrieval (F5)
…
### Adversarial Noise (G5)
…

## Evidence
- NDJSON: docs/verification/v5-runs/YYYY-MM-DD.ndjson
- Seed base: N
- Reproducibility script: scripts/substrate-bench-reproduce.sh
```

Regeneration rules:
- Never score a suite better than its NDJSON supports.
- If a suite was skipped (e.g., C5 harness unavailable in CI), emit explicit `skipped: <reason>` block — never forge metrics.
- Preserve the prior file's section ordering.
- Append a one-line delta history under `## History` (date → composite → suite pass-count).

## 5b. Schema + ordering locks (V5 substrate sealing)

Three schema-level locks land in V5 to support substrate bench scale-out and cross-phase determinism. All three are **A5 scope** (in the shared substrate module initialization).

### 5b.1 Bench run determinism via fixed-seed fixtures

Every substrate suite fixtures (sessions, preferences, transcripts, noise) are versioned and locked post-landing. No runtime regeneration of fixture seeds after bench publication. Rationale: substrate competitors must run exact same scenarios; re-seeding between versions breaks reproducibility.

### 5b.2 Suite execution order lock

G5 aggregator runs suites in fixed order (A5 → B5 → C5 → D5 → E5 → F5 → G5) to prevent cross-suite contamination if state is carried forward. Each suite produces isolated NDJSON; aggregator merges post-run. Rationale: suite_N results must not depend on suite_{N-1} execution order or side effects.

### 5b.3 Harness adapter contract lock

C5 HarnessAdapter trait finalizes the shape that both claude-code and codex drivers must implement. No runtime discovery of harness capabilities; explicit preset availability map in SubstrateCtx. Rationale: G5 adversarial-noise suite must seed facts cross-harness deterministically; loose harness adaptation breaks reproducibility.

### Why V5 owns these

All three are substrate plumbing, not per-phase. A5 is the "build shared substrate" phase; locks live in the shared skeleton. Later phases consume the locked interface without owning the locks.

## 6. Competitor-card policy

`docs/verification/SUBSTRATE_COMPETITOR.md` is a template only. Rules (enforced by G5 Task G5.5 test 12):

- Never fill competitor numbers from secondary sources; must link primary bench output or authoritative repo.
- Any competitor comparison carries a `collected_on:` date and `methodology:` link.
- Template ships with `<< PLACEHOLDER — FILL FROM PRIMARY SOURCE >>` sentinels. Tests assert sentinels exist in the committed template.

## 7. CI publishing flow

G5 Task G5.6 wires nightly CI job:

1. Checkout `research/mining`.
2. `cargo build -p memd-client --release` with `CARGO_TARGET_DIR=/tmp/memd-target` (NFS rule).
3. `MEMD_RATE_LIMIT_DISABLED=1 memd bench substrate --all --regenerate-report --regenerate-10star`.
4. Upload `docs/verification/v5-runs/<date>.ndjson` as artifact.
5. Diff `docs/verification/SUBSTRATE_BENCHMARKS.md`; open PR if changed; fail job if composite < 5.5 unless `MEMD_SUBSTRATE_ALLOW_BELOW_TARGET=1`.
6. If CI env lacks `claude-code` or `codex`, C5 records a skip instead of hard-failing.

CI substrate to confirm at G5 Task G5.6: check `.github/workflows/` first; fall back to whichever runner the repo actually uses (Forgejo / Woodpecker).

## 8. Feature-flag graduation calendar

V5 uses fewer flags than V4; most substrate behavior is always-on once wired. Only one flag owns milestone closure:

1. `MEMD_SUBSTRATE_AGG_PARALLEL` = 1 default after G5 Task G5.7 7-day clean window.

   This is window **1 of 5 total windows** needed for F4 corrections + F4.7 seed to graduate out of feature flags (V4 → V5 → V6 → ... → flag-gate completion). V5 owns this single window; V6 owns the next. See V4-INTEGRATION.md §6 for full 5-window calendar.

Supporting flags (no graduation requirement):
- `MEMD_SUBSTRATE_C5_HARNESS_ALLOW_SKIP` = 1 in CI only; stays off locally (C5 Task C5.4).
- `MEMD_LOOKUP_EXPLAIN_ROUTE` = 1 default once F5 lands (F5 Task F5.1); intrinsic behavior, not guarded by graduation.
- `MEMD_SUBSTRATE_ALLOW_BELOW_TARGET` = 0 at all times in main; set only locally while iterating.

A graduation rollback does not re-open V5 — file a recovery phase.

## 9. Public-bench regression watch

V5 does not directly move LME/LoCoMo/MemBench/ConvoMem, but the shared runtime + new lookup `--explain-route` path cross substrate and public benches. Mandatory checkpoints:

- Post-A5 Task A5.9: run canonical regression suite (`memd bench public --full`) on all four public benches. No regression >1% allowed.
- Post-F5 Task F5.6: same — `--explain-route` overhead must not move public-bench scores.
- Post-G5 Task G5.6: full public + substrate sweep published in `MILESTONE-v5.md`.

If any public bench regresses >3% canonical, hold the phase close and root-cause.

## 9b. Scorecard regenerator strict-mode check (G5 Task G5.4)

G5 Task G5.4 wires the SUBSTRATE_BENCHMARKS.md regenerator. Strict-mode rules (enforced by test harness):

1. **Axis credit ceiling:** No axis in SUBSTRATE_BENCHMARKS may score higher than MILESTONE-v5.md contract allows. Regenerator fails loud if any axis exceeds ceiling.
   - SC, TE, TP must stay at V4 post (4, 4, 3 respectively).
   - CR must stay at 4 (B5 integrates, no credit).
   - PR must not exceed 4. RR must not exceed 6. CH must not exceed 4.

2. **Harness proof requirement:** Every axis lift claim must be backed by a harness fixture assertion in MILESTONE-v5.md per-axis table. G5 Task G5.5 verification gate runs per-axis assertions against regenerated scores; if any assertion fails, regenerator aborts and surfaces the failing fixture path.

3. **Owned-axis-only regeneration:** G5 regenerator writes only axes V5 owns (PR, CH, RR). All other axes are preserved verbatim from the prior MEMD-10-STAR.md entry.

Regeneration template post-G5 (aligned to MILESTONE-v5.md contract):

```markdown
## 10-Star Composite Scorecard

| Axis | Weight | Score | Status |
|------|--------|-------|--------|
| Session continuity | 20% | 4/10 | V4 complete; no V5 work (non-goal per AXIS-OWNERSHIP.md) |
| Correction retention | 15% | 4/10 | B5 measures C4 work; integrates only, no V5 credit |
| Procedural reuse | 15% | 4/10 | F5 live-fire: routine invocation token savings ≥ baseline vs F4.7 observation-only |
| Cross-harness continuity | 15% | 4/10 | C5 bench: claude-code writes stable in codex + round-trip |
| Raw retrieval strength | 15% | 6/10 | 7-suite substrate aggregate (A5+D5+E5+F5+G5): recall@K coverage |
| Token efficiency | 10% | 4/10 | V4 complete; no V5 work (non-goal per AXIS-OWNERSHIP.md) |
| Trust + provenance | 10% | 3/10 | V4 complete; no V5 work (non-goal per AXIS-OWNERSHIP.md) |

**Composite: 4.20 (V5 gate requirement) — regenerated YYYY-MM-DD by G5 substrate aggregator run <id>**

Evidence: docs/verification/v5-runs/YYYY-MM-DD.ndjson
```

Regeneration rules (as per V4-INTEGRATION.md §5):
- Never score an axis higher than the harness evidence supports.
- If an axis has no V5 work, preserve its prior score verbatim.
- Always link to the proof-run NDJSON.
- Append a one-line delta history entry.

## 10. Commit strategy

### Plan-spec land phase (this task)

Fifteen atomic commits on `research/mining`, one per file:

1. `docs(v5): phase-a5-cross-session-recall spec`
2. `docs(v5): phase-b5-correction-propagation spec`
3. `docs(v5): phase-c5-cross-harness-continuity spec`
4. `docs(v5): phase-d5-progressive-depth spec`
5. `docs(v5): phase-e5-provenance-integrity spec`
6. `docs(v5): phase-f5-typed-retrieval spec`
7. `docs(v5): phase-g5-adversarial-noise spec`
8. `docs(v5): phase-a5-plan implementation spec`
9. `docs(v5): phase-b5-plan implementation spec`
10. `docs(v5): phase-c5-plan implementation spec`
11. `docs(v5): phase-d5-plan implementation spec`
12. `docs(v5): phase-e5-plan implementation spec`
13. `docs(v5): phase-f5-plan implementation spec`
14. `docs(v5): phase-g5-plan implementation spec`
15. `docs(v5): V5-INTEGRATION cross-phase plan`

### Execution commits per phase

Each phase plan has its own task list that commits per task (A5 = 9, B5 = 7, C5 = 6, D5 = 7, E5 = 5, F5 = 6, G5 = 7 → 47 execution commits across V5). Spec-land does **not** produce those.

### Handoff commit

One more commit after the 15 docs commits:

```
docs(handoff): V5+V6 plan specs landed, next agent executes A5
```

Content: new file `docs/handoff/YYYY-MM-DD-v5-v6-plan-spec-complete-next-execute.md`.

## 11. Cross-phase API surface summary

| Introduced in | Symbol / Path | Consumed by |
| --- | --- | --- |
| A5 | `benchmark::substrate::session_driver::*` | B5, C5, D5, E5, F5, G5 |
| A5 | `benchmark::substrate::scorers::*` (recall@K, etc.) | B5 (propagation), D5 (completeness), E5 (completeness), F5 (type-correct), G5 (wins-rate) |
| A5 | `benchmark::substrate::report::NdjsonEmitter` | all |
| A5 | `SubstrateSuite` trait | B5..G5 runners |
| A5 | `.memd/benchmarks/substrate/fixtures/shared/*` | any suite |
| B5 | `ProvenanceChainScorer` | E5 (chain length check), G5 (tie-break) |
| C5 | `HarnessAdapter` trait + claude_code + codex drivers | G5 (noise can be seeded cross-harness) |
| D5 | per-depth completeness fixtures | G5 (noise also tested under depth) |
| E5 | `provenance_auditor::audit_record` | B5 (reuse), G5 (aggregator-level spot-check) |
| F5 | `memd lookup --explain-route` JSON shape + `docs/contracts/type-taxonomy.md` | G5 (aggregator classifies noise wins by type), V6 ingest adapter |
| G5 | `docs/verification/SUBSTRATE_BENCHMARKS.md` | V6 entry gate |
| G5 | `scripts/substrate-bench-reproduce.sh` | external reproducibility claims |
| G5 | `docs/verification/SUBSTRATE_COMPETITOR.md` template | V6 competitor comparisons |
| G5 | `docs/verification/v5-runs/*.ndjson` | V6 regression baseline |

## 12. Open questions for next executor

Surface in TodoWrite or phase kickoff — do not silently assume:

- Where does `public_benchmark.rs` emit its scorecard? Confirm before A5.3 so `substrate/mod.rs` mirrors rather than diverges.
- `inventory` crate already in deps? If not, A5.1 adds; else reuse existing dispatcher pattern.
- `memd lookup --json` current shape: does it already include `routed_kinds`? If yes, F5.1 is just making it explicit + stable.
- Harness CI availability: confirm runner image ships claude-code + codex CLIs, or decide to rely on skip path.
- Taxonomy card format: JSON vs YAML vs markdown — F5 Task F5.2 decides; pick one before A5 references it.

## 13. Exit criteria for V5 as a milestone

All seven phase exit criteria met AND G5 exit criteria met AND:

- **No axis credit without harness proof** (per 0.1.0-CONTRACT.md): Every axis lift in MILESTONE-v5.md per-axis assertions table has a passing harness fixture. G5 Task G5.5 verification gate runs all three assertions (PR invocation, CH round-trip, RR recall@K aggregate); if any fails, milestone cannot close.
- 10-STAR composite ≥ 4.20 written to `docs/verification/MEMD-10-STAR.md` by the G5 aggregator via strict-mode regenerator (§9b).
- `docs/verification/SUBSTRATE_BENCHMARKS.md` regenerated, current run ≤ 7 days old.
- `docs/verification/milestones/MILESTONE-v5.md` filled with per-axis basis + assertion fixtures.
- `ROADMAP.md` V5 → closed, V6 → in progress.
- `scripts/substrate-bench-reproduce.sh` passes on fresh clone (±0.03 per metric).
- `docs/verification/SUBSTRATE_COMPETITOR.md` template committed with sentinels intact.
- No open backlog items tagged `axis: procedural_reuse`, `axis: cross_harness`, or `axis: raw_retrieval` at severity `blocker`.
- V5 owns 1 of 5 windows required for F4 flag graduation (§8): `MEMD_SUBSTRATE_AGG_PARALLEL` clean window runs post-G5, documented in flag-calendar notes.
- Substrate schema/ordering locks (§5b) landed and covered by A5 unit tests.
- Final handoff doc points at `docs/phases/v6/` (already created in this plan-spec phase).
