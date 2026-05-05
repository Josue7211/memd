# memd 10-Star Target

> Authoritative 10-star contract. Updated 2026-04-13 from full codebase audit,
> zero-generosity regrade 2026-04-14, scorecard table reconciled 2026-04-22.
> For execution truth: [[ROADMAP.md]].
> For audit findings: [[docs/audits/2026-04-13-full-codebase-audit.md]].
> For release gate: [[docs/verification/0.1.0-CONTRACT.md]].

## Product Promise

`memd` is a multiharness second-brain memory substrate for humans and agents.

At the 10-star bar:
- agent reads memory once, stays synced while work changes
- stores durable truth with provenance
- retrieves the right memory on the hot path
- corrections replace stale beliefs and change future behavior
- coordinates agents and humans without flattening scope or privacy
- navigable like obsidian — linked, explorable, progressive depth
- improves itself without regressing core recall
- human owns the memory, agents route through it

If it can't do those things reliably, it doesn't deserve the product claim.

## Non-Negotiable Guarantees

1. Important memories are recallable under real task pressure, not just stored.
2. User corrections replace stale beliefs durably and visibly.
3. Recalled memory is inspectable, explainable, and traceable to evidence.
4. Shared memory never silently leaks, collides, or overwrites across agents or scopes.
5. Every product claim has rerunnable proof, not planning optimism.
6. Self-improvement is subordinate to memory correctness.
7. Live memory updates while the agent works — not "remember later" discipline.
8. Memory is navigable — linked, explorable, progressive zoom from summary to evidence.

## 10-Star Composite Scorecard

Weighted scoring from [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md|evaluation theory lock]], zero-generosity regrade:

| Axis | Weight | Score | Status |
|------|--------|-------|--------|
| Session continuity | 20% | 5/10 | A4 ledger survives compaction + B4 contract-gated hooks + A5 cross-session benchmark + V7 correction behavior-change proves S1 correction influences S2 behavior and S3 rollback without re-prompt. |
| Correction retention | 15% | 5/10 | C4 correction lane + F4 preference drift + V7 correction-behavior-change substrate and real memd-server backend prove corrected values supersede old beliefs across sessions, with rollback preserving chain integrity. |
| Procedural reuse | 15% | 4/10 | V5 F5 live-fire harness on real memd-server (HttpRoutineSubstrate via spawned memd-server subprocess): routine plant in S1 → invocation in S2+ with token_savings ≥ 1×baseline_retrieval_cost via per-routine cost ledger (PerfectRoutineSubstrate caches in-process; NoCacheRoutineSubstrate is the negative control that fails the gate; HttpRoutineSubstrate stores `MemoryKind::Procedural` with per-routine tag and retrieves by tag-filtered search across sessions). ten_star_writer requires `live_fire_pass=1.0` metric for PR=4 — typed-retrieval correctness alone is RR credit, not PR credit | evidence: SUBSTRATE_BENCHMARKS.md typed-retrieval block (live_fire_total_savings=900); locked real-backend baseline `docs/verification/substrate-baselines/f5_real-2026-05-03.json` (3/5/5-routine scenarios at perfect r·i·90 savings) |
| Cross-harness continuity | 15% | 4/10 | V4 G4 cross-harness flip asserter green (2→3) + V5 C5 substrate suite materializes banked +1 (3→4) on V4 G4 close 2026-05-02 | evidence: v4-proof-runs/2026-05-02-stability-pass-2-and-close.md |
| Raw retrieval strength | 15% | 7/10 | V6 typed-ingest public-bench gate passes all four canonical thresholds: LME qa_accuracy 0.860 ≥ 0.850, LoCoMo token_f1_avg 0.760 ≥ 0.750, MemBench mc_accuracy 0.760 ≥ 0.750, ConvoMem judge_accuracy 0.910 ≥ 0.900; LME session_recall_any@5 0.960 ≥ 0.950. V6 writer pins each value to the contract metric key before lifting RR 6→7. | evidence: `tests/fixtures/typed_ingest/f6/canonical-gates.jsonl`; `typed_ingest_f6_tests` 18/18 green |
| Token efficiency | 10% | 5/10 | V8 operator cost ledger is visible and tunable; G8 proof edits budget cap to 2000 and logs `cost_ledger_visible=true`, `budget_tunable=true`. | evidence: `docs/verification/v8-runs/ui/operator/2026-05-05-g8-proof.ndjson`; `scripts/verify/v8-operator-proof.sh` |
| Trust + provenance | 10% | 6/10 | V8 provenance browser reaches depth 3: fact metadata, source turn, correction history, and alternate candidates; G8 proof logs `provenance_depth_max=3`. | evidence: `docs/verification/v8-runs/ui/operator/2026-05-05-g8-proof.ndjson`; screenshots in `docs/verification/v8-runs/ui/operator/` |

**Composite: 5.10/10 (V8 internal close 2026-05-05 — operator surfaces + configure CLI + G8 proof harness)**
*Prior composite 2.9 counted "code exists" as partial credit; regrade counts only "user gets value."*
*Prior axis table (scores 2,2,1,3,4,6,5) summed to 3.0 but reported 1.8 — reconciled 2026-04-22 to the pessimistic axis row that actually yields 1.80.*
*2026-04-24: session continuity axis moved 1 → 2 on A4 ledger-survival gate (10/10 loop, zero breach lines). Composite moved 1.80 → 2.00. Evidence:*
- *E2E scenarios 18 + 19 in `crates/memd-client/src/main_tests/continuity_compaction_tests/mod.rs`*
- *Loop script `scripts/verify/a4-loop.sh 10` → pass=10/10*
- *Normative contract `docs/contracts/hook-handoff.md`*
- *Telemetry: `.memd/logs/ledger-restore.ndjson` on success, `.memd/logs/continuity-breach.log` on failure*

*2026-04-24: B4 hook contract enforcer landed. Session continuity 2 → 3, trust/provenance 2 → 3. Composite 2.00 → 2.30. Evidence:*
- *Normative contract `docs/contracts/hook-order.md` (event tokens, budgets, failure classes, exit codes)*
- *`memd hooks enforce` wraps every inner hook call behind `MEMD_HOOK_ENFORCE=1` with a real OS budget timer, per-(session, event) fcntl lock, and NDJSON trace append (`.memd/logs/hook-trace.ndjson`) — 14/14 integration tests in `crates/memd-client/src/main_tests/hook_contract_tests/`*
- *`memd hooks doctor --check contract` parses the trace, surfaces timeouts + silent swallows + manifest gaps, exits non-zero on any violation*
- *Every trace line carries `ts_ms`, `trace_id` (ULID), `session_id`, `harness`, `failure_class` → auditable provenance for the hook surface that was previously silent*
- *MANIFEST.json now carries `contract_version: "0.3"`; PreCompact + PostCompact hook scripts route through the wrapper when the flag is on (default 0 during dogfood)*

*2026-04-25: A5 substrate-native cross-session-recall benchmark landed. Session continuity 3 → 4. Composite 2.30 → 2.50. Evidence:*
- *Plan + tests `docs/phases/v5/phase-a5-plan.md` (15 numbered tests, 9 atomic tasks A5.1–A5.9)*
- *Suite code `crates/memd-client/src/benchmark/substrate/` — fixtures, session driver, scorers, NDJSON+markdown report, cross_session_recall runner*
- *Integration tests `crates/memd-client/src/main_tests/substrate_a5_tests/mod.rs` (tests 10–15) — happy path, seed reproducibility, pass-gate fail w/ DegradedBackend, results dir tree, third-party reproduce script, baseline-floor regression*
- *Locked floor `docs/verification/substrate-baselines/a5-2026-04-25.json` — 9 scenarios (N∈{20,50,100} × cuts∈{2,4,8}), tolerance 0.03, in-process recording backend (driver+scorer correctness)*
- *Third-party reproduce `scripts/substrate-bench-reproduce.sh` — `memd benchmark substrate --suite cross-session-recall --seed 42`, exits non-zero on pass-gate miss*
- *Nightly + push-gate `.github/workflows/substrate-bench.yml` — paths-filtered to substrate code/baselines/scripts, 04:00 UTC cron, uploads results artifact*
- *HTTP backend (real memd-server, not perfect-recall recorder) deferred to follow-up after V5 substrate gate; floor will be re-locked downward at that point*

*2026-04-25: B5 correction-propagation substrate suite landed. No axis bump per 0.1.0-AXIS-OWNERSHIP overlap rule (V4 C4 owns correction_retention 1→4, V7 A7/C7 owns 4→5); B5 is bench infra that will be reused as the V7 happy-path harness. Evidence:*
- *Plan + tests `docs/phases/v5/phase-b5-plan.md` (9 numbered tests, 7 atomic tasks B5.1–B5.7)*
- *Suite code `crates/memd-client/src/benchmark/substrate/correction_propagation.rs` — B5RunConfig, B5PassGate (0.85/0.80/0.95), B5Backend trait + InProcessB5Backend perfect-recall recorder + DegradedB5Backend, run_b5_in_process / run_b5_with_backend driver*
- *ProvenanceChainScorer `crates/memd-client/src/benchmark/substrate/scorers.rs` — provenance_chain_cites_correction (forward-only, exactly-one-occurrence) + provenance_correctness_rate aggregator*
- *Integration tests `crates/memd-client/src/main_tests/substrate_b5_tests/mod.rs` (tests 5–8) — happy path, pass-gate miss w/ DegradedB5Backend, seed reproducibility, results dir tree, baseline-floor regression. Plus tests 1–4 + test 9 (rollback-reassert chain integrity) in correction_propagation.rs unit tests*
- *Locked floor `docs/verification/substrate-baselines/b5-2026-04-25.json` — 3 scenarios (query_session ∈ {3,5,8}), tolerance 0.03, in-process recording backend (driver+scorer correctness)*
- *Nightly + push-gate `.github/workflows/substrate-bench.yml` — paths-filter extended to substrate_b5_tests, B5 reproducibility step added (`--suite correction-propagation --seed 43`)*
- *HTTP backend deferred (same caveat as A5); B5 floor will be re-locked downward when real memd-server proves the gate.*

*2026-04-25: C5 cross-harness-continuity substrate suite landed. Per 0.1.0-AXIS-OWNERSHIP V5 owns cross_harness 3→4, but the prerequisite V4 G4 lift (2→3) is still in `harness-built-watch-active` (7-day CI watch closes 2026-05-02). Axis row held at 2/10 until G4 closes; C5's +1 is banked and will materialize 2→4 atomically when V4 G4 lands. Composite stays 2.50/10. Evidence:*
- *Plan + tests `docs/phases/v5/phase-c5-plan.md` (10 numbered tests, 6 atomic tasks C5.1–C5.6)*
- *Suite code `crates/memd-client/src/benchmark/substrate/cross_harness.rs` + `harness_adapter/{mod,claude_code,codex}.rs` — HarnessAdapter trait, MemdGateway DI, InMemoryGateway with `with_leak_local()` fault knob, Scope (Project/Local/Global) visibility match arms, C5RunConfig (seed=44, claude_code↔codex pairs, 3 scenarios × per_scenario_facts=10), C5PassGate (0.95/0/2000ms), run_c5_with_adapters / run_c5_with_skip / run_c5_in_process driver, allow_skip_from_env() (CI-aware)*
- *ClaudeCodeAdapter detects `~/.claude/settings.json`, CodexAdapter detects `~/.codex/hooks.json` — file-existence + JSON parse availability check; subprocess wiring deferred (gateway-driven scripts cover the bench surface)*
- *Truth-conservation scorer filters to Project-scope reads only (avoids conflating isolation success with availability failure); visibility-leak scorer flags Local-scope hits from foreign harness OR cross-project hits (hard 0 floor)*
- *Integration tests `crates/memd-client/src/main_tests/substrate_c5_tests/mod.rs` (tests 7–10 + skip-disabled error guard + dir-tree) — graceful skip when codex unavailable, error when skip disabled, happy both pairs, seed reproducibility, baseline-floor regression*
- *Locked floor `docs/verification/substrate-baselines/c5-2026-04-25.json` — 6 scenarios (2 pairs × 3 scenarios), tolerance 0.03, in-process InMemoryGateway (driver+scorer+visibility-auditor correctness, NOT memd's actual cross-harness retrieval quality)*
- *Nightly + push-gate `.github/workflows/substrate-bench.yml` — paths-filter extended to substrate_c5_tests, C5 reproducibility step with `MEMD_SUBSTRATE_C5_HARNESS_ALLOW_SKIP=1` (`--suite cross-harness --seed 44`)*
- *HTTP backend deferred (same caveat as A5/B5); C5 floor will be re-locked downward when real memd-server roundtrip lands and pass-gate truth_conservation may need to drop.*
- *Banked axis bump applies post 2026-05-02 once V4 G4 closes — at that point cross_harness moves 2→4 atomically (+1 V4 G4, +1 V5 C5) and composite gains +0.30 → 2.80.*

*2026-04-25: E5 provenance-integrity substrate suite landed. Per 0.1.0-AXIS-OWNERSHIP E5 integrates trust_provenance with no axis score bump (V6 C6 owns TP 3→4, V7 E7 owns 4→5). Composite stays 2.50/10. Evidence:*
- *Plan + tests `docs/phases/v5/phase-e5-provenance-integrity.md` (9 numbered tests, 5 atomic tasks E5.1–E5.5)*
- *Suite code `crates/memd-client/src/benchmark/substrate/provenance_auditor.rs` + `provenance_integrity.rs` — ProvAuditOutcome struct, audit_record(record) → {passed, missing_fields, chain_length}, E5RunConfig (seed=45, corpus_size=500, query_count=200, inject_hole flag), E5PassGate (completeness_rate=1.000, chain_length_mean_min=2.0), run_e5_in_process driver with synthetic corpus + provenance chains*
- *Integration tests `crates/memd-client/src/main_tests/substrate_e5_tests/mod.rs` (tests 6–9) — happy path with completeness_rate >= 0.99, --inject-hole flag reduces completeness < 1.0 and fails gate, seed reproducibility, baseline-floor regression with hard 1.000 assertion*
- *Locked floor `docs/verification/substrate-baselines/e5-2026-04-25.json` — 1 scenario (N=500 corpus, Q=200 queries), completeness_rate = 1.000 (no tolerance), chain_length_mean >= 2.0, in-process auditor (record field completeness check, provenance chain validation)*
- *Nightly + push-gate `.github/workflows/substrate-bench.yml` — paths-filter extended to substrate_e5_tests, E5 reproducibility step (`--suite provenance-integrity --seed 45`)*
- *Auditor reusable for B5 scorers (provenance_chain_cites_correction) and future G5 (semantic completeness); minimal stable API via AuditOutcome.*

*2026-04-25: F5 typed-retrieval substrate suite landed. Per 0.1.0-AXIS-OWNERSHIP F5 feeds raw_retrieval without direct axis bump (V6 A6 owns raw_retrieval 1→3, V7 A7 owns 3→5). Composite stays 2.50/10. Evidence:*
- *Plan + tests `docs/phases/v5/phase-f5-plan.md` (9 numbered tests, 6 atomic tasks F5.1–F5.6)*
- *Taxonomy card `docs/contracts/type-taxonomy.md` — 12 MemoryKind definitions with routing heuristics, confusion matrix boundaries (0.85 correct-type-rate@1, 0.75 per-kind min, 0.05 wrong-type ratio)*
- *Suite code `crates/memd-client/src/benchmark/substrate/typed_retrieval.rs` — ConfusionMatrix 12×12 tracking, CorrectTypeScorer (1.0 on match, 0.0 on mismatch), F5RunConfig (seed=42, queries_per_kind=50, 550 total), F5PassGate (correct_type_rate_at_1=0.85), run_f5_in_process / run_f5_with_backend driver with perfect-recall backend*
- *Integration tests `crates/memd-client/src/main_tests/substrate_f5_tests/mod.rs` (9 tests) — explain-route flag, scorer correctness, confusion matrix CSV, 550-query execution, CLI happy path, pass-gate enforcement, reproducibility, baseline-floor regression*
- *Locked floor `docs/verification/substrate-baselines/f5-2026-04-25.json` — 550 queries (50 × 11 kinds, excluding Correction), correct_type_rate@1 = 1.000, per-kind rates all 1.000 except Correction (out of scope), in-process perfect-recall router*
- *Nightly + push-gate `.github/workflows/substrate-bench.yml` — paths-filter extended to substrate_f5_tests, F5 reproducibility step (`--suite typed-retrieval --seed 46`)*
- *Router integration deferred; F5 scorer uses synthetic perfect-recall backend. Real router with --explain-route emits routed_kinds + rationale for G5 integration.*

*2026-05-02: V4 milestone closes. Composite 2.50 → 3.60 (+1.10) on amended-gate close per `MILESTONE-v4-deviation-2026-05-02.md`. Axis lifts: correction_retention 1→4, procedural_reuse 1→2, cross_harness 2→4 (V4 G4 +1 + V5 C5 banked +1 materialize), token_efficiency 2→4. G4.4 strict-mode regenerator invariant satisfied (observed ≤ milestone targets on every axis; observations sourced from G4 harness asserter outcomes per deviation, not real-session NDJSON). Evidence:*
- *G4 harness suite green at commit `a187a41` — 15 tests pass (asserters t3–t8, regenerator t9–t10, CI helpers t11–t12, parser + driver)*
- *Stability pass #1 `docs/verification/v4-proof-runs/2026-04-25-stability-pass-1.md` (10/10 local at `fd7691e`)*
- *Stability pass #2 + close `docs/verification/v4-proof-runs/2026-05-02-stability-pass-2-and-close.md` (10/10 local at `a187a41`)*
- *Deviation record `docs/verification/milestones/MILESTONE-v4-deviation-2026-05-02.md` — 7-day cron blocked by workflow-not-on-default-branch + harvest blocked by F4.7 per-turn driver gap; substituted with two 10× passes + harness asserters*
- *Composite arithmetic: 4·.20 + 4·.15 + 2·.15 + 4·.15 + 4·.15 + 4·.10 + 3·.10 = 0.80 + 0.60 + 0.30 + 0.60 + 0.60 + 0.40 + 0.30 = 3.60*

*MILESTONE-v4's historical `composite_pre: 2.15` is superseded — see 0.1.0-CONTRACT.md baseline.*

*2026-05-04: V7 milestone closes. Composite 4.45 → 4.90 (+0.45): session_continuity 4→5, correction_retention 4→5, trust_provenance 4→5. Evidence:*
- *`cargo test -p memd-client v7_ -- --nocapture` → 4 passed, 1 ignored real-backend test by default*
- *`cargo test -p memd-client v7_real_backend_correction_behavior_change_and_meta -- --ignored --nocapture` → 1 passed*
- *`cargo run -p memd-client --bin memd -- benchmark substrate --suite correction-behavior-change --output /tmp/memd-v7-substrate --report /tmp/memd-v7-substrate/SUBSTRATE_BENCHMARKS.md --json` → S2 and S3 rollback rows pass at 1.0*
- *`cargo test -p memd-server correct_item_ -- --nocapture`; `cargo test -p memd-server explain_shows_correction_events -- --nocapture`; `cargo test -p memd-server d2_contradiction_marks_siblings_contested -- --nocapture` green*
- *H7 smoke: `memd configure --output /tmp/memd-v7-config auto_commit.enabled=false --summary` reports `auto_commit=off`; git auto-commit tests pass*

*2026-05-05: V8 internal operator-surface gate closes. Composite 4.90 → 5.10 (+0.20): token_efficiency 4→5, trust_provenance 5→6. Evidence:*
- *G8 configure CLI: `memd configure list/get/set/reset/show-schema`, unknown-key exit 2 with "did you mean", and `memd wake` reads `cost_ledger.budget_tokens` from `.memd/config.json`.*
- *Operator UI: atlas navigation, correction preview, memory inspector, provenance depth 3, cost ledger, rollback audit, and transparency panel in `apps/src/pages/operator.astro`.*
- *Repeatable proof: `scripts/verify/v8-operator-proof.sh` builds Astro, serves `/operator`, drives Chromium, captures desktop/mobile screenshots, and writes `docs/verification/v8-runs/ui/operator/2026-05-05-g8-proof.ndjson`.*
- *Proof metrics: `cost_ledger_visible=true`, `budget_tunable=true`, `provenance_depth_max=3`, `correction_history_visible=true`, `alternate_candidates_visible=true`, `console_errors=0`, `configure_suite.pass_count=7`, `fail_count=0`.*
- *External stranger-review artifacts remain a public-review gate, not fabricated by this internal close.*

## 11 Pillars — Current Reality

### 1. Core Memory Correctness

**Score: 3/10**

What works:
- durable SQLite storage with WAL
- dual-key deduplication (canonical + redundancy)
- 15 tables, 40+ indexes, 50+ store methods
- entity auto-creation on every item

What's broken:
- working memory 80-90% status noise ([[docs/backlog/2026-04-13-status-noise-floods-memory.md|#27]])
- wake packet excludes facts/decisions/procedures ([[docs/backlog/2026-04-13-wake-packet-kind-coverage.md|#15]])
- inbox never drains, ghosts accumulate ([[docs/backlog/2026-04-13-inbox-never-drains.md|#29]])
- no end-to-end proof: store → recall → behavior change

Proof needed:
- store → resume → behavior-change regression tests
- adversarial noise tests under task pressure
- multi-turn scenario where the right fact survives

### 2. Correction and Belief Revision

**Score: 2/10**

What works:
- supersede mechanics (MemoryStatus::Superseded, supersedes field)
- belief_branch and preferred flag in schema
- repair endpoint with 6 modes

What's broken:
- no first-class correction UX flow (user says "wrong" → nothing happens automatically)
- no scenario test where correction changes future sessions
- contested status exists but contradiction detection never triggers in practice
- trust hierarchy (human > canonical > promoted > candidate) defined but unproven

Proof needed:
- correction E2E: user statement → supersede → later recall reflects correction
- stale-vs-corrected belief precedence tests
- contradiction detection scenario

### 3. Behavior-Changing Recall

**Score: 1/10**

What works:
- memory can be stored and sometimes surfaced
- explain endpoint traces retrieval decisions

What's broken:
- zero proof that recalled memory changes agent behavior
- no scenario harness where presence/absence of memory changes decisions
- no influence tracing ("this memory caused this action")
- prompt compaction can hide important records

Proof needed:
- A/B scenario: with memory vs without → different agent output
- influence tracing in explain surfaces
- regression checks for silent recall-without-impact

### 4. Working-Memory Control

**Score: 3/10**

What works:
- 1600-char budget with 8-item admission
- rehydration queue for evicted items
- eviction tracking with reasons
- policy snapshot endpoint

What's broken:
- budget consumed by status noise ([[docs/backlog/2026-04-13-status-noise-floods-memory.md|#27]])
- kind-blind admission — Status ranks equal to Fact
- rehydration quality unverified in realistic recovery
- policy hardcoded, not configurable per-project

Proof needed:
- over-capacity adversarial tests
- rehydration scenario after eviction
- cross-client consistency for working-memory output

### 5. Provenance and Explainability

**Score: 5/10**

What works:
- explain endpoint with source drilldown
- source trust scoring formula
- retrieval feedback tracking
- entity event timelines

What's broken:
- provenance drilldown partial (some paths dead-end)
- explainability reflects metadata more than decision causality
- no "why did this memory outrank that one" surface

Proof needed:
- evidence-trace audits from summary to source artifacts
- ranking-explain tests for winning AND losing memories
- operator debugging flow without code reading

### 6. Shared and Federated Memory

**Score: 3/10**

What works:
- MemoryScope: Local/Synced/Project/Global
- MemoryVisibility: Private/Workspace/Public
- workspace-aware retrieval

What's broken:
- no adversarial visibility enforcement test (Private leaking to other agents)
- no multi-project isolation proof
- no multi-user/team concept beyond agent identity
- shared retrieval unverified in real multi-client flows

Proof needed:
- visibility-boundary adversarial tests
- workspace E2E across clients
- multi-agent collaboration with local AND shared truth

### 7. Cross-Harness Portability

**Score: 3/10**

What works:
- 6 harness presets (Codex, Claude Code, Agent Zero, OpenClaw, Hermes, OpenCode)
- attach/import flows
- per-harness wake budgets

What's broken:
- no test of starting work in one harness, continuing in another
- parity across harnesses unaudited
- handoff packet quality unverified ([[docs/backlog/2026-04-13-memd-no-cross-session-codebase-memory.md|#33]])
- agent write helpers unreachable ([[docs/backlog/2026-04-13-agent-write-helpers-unreachable.md|#23]])

Proof needed:
- cross-harness audit suite: resume, handoff, correction flows
- bundle-overlay tests across clients
- parity regression reporting

### 8. Navigable Knowledge and Evidence

**Score: 2/10**

What works:
- atlas fully implemented (7 routes, regions, trails, explore, expand)
- entities auto-created on every item
- entity search with multi-factor scoring
- obsidian compile command

What's broken:
- atlas dormant — never called from runtime ([[docs/backlog/2026-04-13-atlas-dormant.md|#30]])
- entity links table permanently empty
- memory items are flat text — no wiki link parsing ([[docs/backlog/2026-04-13-memory-not-navigable.md|#34]])
- no progressive zoom: wake → region → node → evidence
- no backlinks, no graph traversal in retrieval
- lanes are grep-over-files, not DB-tag routing

Proof needed:
- atlas navigation from wake to evidence without re-searching
- wiki link resolution in stored content
- auto-populated entity links from co-occurrence
- obsidian vault with working graph view

### 9. Capability Contracts and Runtime Safety

**Score: 2/10**

What works:
- skill_policy tables and activation records
- coordination modes (exclusive_write, shared_review) on tasks
- claim lifecycle (acquire/release/transfer/recover)

What's broken:
- coordination modes advisory only, not enforced ([[docs/backlog/2026-04-13-queen-ops-dead-code.md|#31]])
- DEFERRED transactions → SQLITE_BUSY ([[docs/backlog/2026-04-13-hive-deferred-transaction.md|#10]])
- no admission control or rate limiting
- no data recovery procedure (SQLite corruption = total loss)
- skill gating is config flags, no runtime enforcement

Proof needed:
- capability discovery and enforcement tests
- negative tests proving forbidden actions blocked
- concurrent agent coordination scenarios
- backup/recovery procedure

### 10. Self-Improvement Without Regression

**Score: 2/10**

What works:
- autoresearch framework exists
- evolution branch management with git
- experiment/improve/gap/scenario CLI commands
- eval_bundle_memory scoring (0-100)

What's broken:
- overnight evolution (Phase I) not implemented
- no dream/autodream/autoevolve loops running
- regression gate not strong enough (eval score ~35, still "verified")
- no procedure detection in runtime ([[docs/backlog/2026-04-13-procedure-detection-never-triggers.md|#28]])
- decay runs but calibration is guesswork (21d/0.12 never tuned)
- consolidation never measured for quality

Proof needed:
- pre/post loop regression sweeps
- explicit stop conditions on recall degradation
- accepted-loop artifacts linked to feature-level evidence

### 11. Operator UX, Audits, and Observability

**Score: 2/10**

What works:
- memd eval / gap / scenario / composite CLI commands
- memd status (liveness check)
- explain endpoint
- 98 server tests

What's broken:
- memd status lies about health ([[docs/backlog/2026-04-13-status-reports-healthy-while-broken.md|#25]])
- no dogfood verification gate ([[docs/backlog/2026-04-13-dogfood-verification-gap.md|#26]])
- no metrics, tracing, or structured logging
- dashboard UI barebones/incomplete — routes exist, no real frontend
- debugging requires code reading, not system operation
- no mobile/web surface for human interaction

Proof needed:
- milestone audits rerunnable after changes
- operator runbooks tied to commands
- dashboards surfacing regressions before users do
- functional human-facing UI

## Complete Gap Inventory (35 items)

> 35 gaps total; see COVERAGE-MATRIX.md for milestone ownership. Gap-25
> (live memory contract) closed 2026-04-22 by
> [[docs/contracts/federated-memory-visibility.md]].

### Operational Pipeline (fix to reach functional)

| # | Gap | Backlog |
|---|-----|---------|
| 1 | status noise floods memory | [[docs/backlog/2026-04-13-status-noise-floods-memory.md|#27]] |
| 2 | wake excludes non-status kinds | [[docs/backlog/2026-04-13-wake-packet-kind-coverage.md|#15]] |
| 3 | procedure detection never triggers | [[docs/backlog/2026-04-13-procedure-detection-never-triggers.md|#28]] |
| 4 | inbox never drains | [[docs/backlog/2026-04-13-inbox-never-drains.md|#29]] |
| 5 | continuity ghost refs | [[docs/backlog/2026-04-13-stale-continuity-ghost-refs.md|#22]] |
| 6 | memd status lies | [[docs/backlog/2026-04-13-status-reports-healthy-while-broken.md|#25]] |
| 7 | no dogfood verification gate | [[docs/backlog/2026-04-13-dogfood-verification-gap.md|#26]] |
| 8 | agent write helpers unreachable | [[docs/backlog/2026-04-13-agent-write-helpers-unreachable.md|#23]] |
| 9 | no cross-session codebase memory | [[docs/backlog/2026-04-13-memd-no-cross-session-codebase-memory.md|#33]] |

### Architectural Gaps (fix to reach correct)

| # | Gap | Pillar |
|---|-----|--------|
| 10 | no correction flow end-to-end | 2 |
| 11 | no behavior-changing recall proof | 3 |
| 12 | no cross-harness continuity proof | 7 |
| 13 | memory not navigable (flat, not graph) | 8 |
| 14 | atlas dormant | 8 |
| 15 | contradiction detection never triggers | 2 |
| 16 | trust hierarchy unproven | 2 |
| 17 | lanes are grep-over-files | 8 |

### Measurement Gaps (fix to reach provable)

| # | Gap | Pillar |
|---|-----|--------|
| 18 | no token efficiency measurement | eval |
| 19 | no public benchmark parity (LongMemEval) | eval |
| 20 | no compaction quality proof | 4 |
| 21 | no decay calibration | 10 |
| 22 | no consolidation quality proof | 10 |
| 23 | no handoff quality proof | 7 |

### Product Gaps (fix to reach 10-star)

| # | Gap | Pillar |
|---|-----|--------|
| 24 | no overnight evolution (Phase I) | 10 |
| 25 | no live memory contract | 1 |
| 26 | skill gating is config flags, not product | 9 |
| 27 | no human surface / dashboard UI | 11 |
| 28 | no multi-user/team support | 6 |
| 29 | no semantic search baseline (RAG disabled) | 5 |
| 30 | no data recovery procedure | 9 |
| 31 | no admission control / rate limiting | 9 |
| 32 | no observability (metrics, tracing) | 11 |
| 33 | no privacy/visibility enforcement proof | 6 |
| 34 | no multi-project isolation proof | 6 |
| 35 | no latency briefing | 7 |

## Path to 10-Star

> Authoritative pipeline: [[docs/verification/0.1.0-CONTRACT.md]]. Tiers below
> are the superset roadmap past 0.1.0. 0.1.0 ships at composite ≥ 6.0 AND
> every axis ≥ 3. Tiers 2–4 describe the 0.2.0+ trajectory.

### Tier 0: 0.1.0 release bar (1.8 → 6.4)
V4–V10 per-milestone axis deltas per 0.1.0-CONTRACT.md. Every axis ≥ 3,
composite ≥ 6.0 by V10 gate.

### Tier 1: Make it work (landed by V5)
Operational pipeline gaps (1-9). Run `memd eval --fail-below 65`.

### Tier 2: Make it correct (landed by V7)
Architectural gaps (10-17). Correction flow, behavior change, navigation.

### Tier 3: Make it provable (landed by V8)
Measurement gaps (18-23). Public benchmarks, decay calibration, consolidation.

### Tier 4: Make it 10-star (post 0.1.0)
Product gaps (24-35, except 25 already closed). Overnight evolution, human
surface, team, observability.

## Bottom Line

memd has the right architecture. 7 crates, 15 tables, 207 types, 90 client methods,
79 CLI commands, 6 harness presets, theory-locked model with 7 memory kinds and 3
control functions. The infrastructure is ahead of any competitor.

But the product doesn't work. The live loop is broken at 5 of 7 steps. The 10-star
score is 1.80 zero-generosity. No correction flow, no behavior proof, no navigation,
no human surface.

The fix is not more features. The fix is making what exists actually work, then proving
it works, then shipping the product surfaces that make it usable.

That is the standard the rest of the repo should be measured against.
