> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Feature Registry

## Status Vocabulary

- `unverified`
- `auditing`
- `verified`
- `partial`
- `broken`

## Verification Depth Vocabulary

- `minimal`
- `strong`
- `exhaustive`

## Feature Template

### FEATURE-000: Example Feature

- version: `v0`
- milestones: `v0`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Describe what a user should experience if this feature works.

#### Implementation Surfaces

- `path/to/file`
- `binary command`
- `HTTP route`

#### Dependencies

- `FEATURE-...`

#### Verification Methods

- unit/integration: `command here`
- workflow: `command or manual flow here`
- adversarial: `noise or failure scenario here`
- migration: `if required`
- cross-harness: `if required`

#### Failure Modes

- how this feature usually breaks

#### Notes

- audit notes and caveats

## v1 Features

### FEATURE-RAW-TRUTH-SPINE: Source-Linked Raw Truth Spine

- version: `vNext`
- milestones: `phase-a`
- status: `unverified`
- verification_depth: `strong`

#### User Contract

The main ingest paths write a source-linked raw spine record first, keep the
server event timeline source-linked, and surface the raw spine in compiled
bundle events.

#### Implementation Surfaces

- `crates/memd-client/src/runtime/raw_spine.rs`
- `crates/memd-client/src/runtime/checkpoint.rs`
- `crates/memd-client/src/runtime/ingest_runtime.rs`
- `crates/memd-client/src/cli/cli_hook_runtime.rs`
- `crates/memd-client/src/compiled/event.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/tests/mod.rs`

#### Dependencies

- `FEATURE-V1-CORE-STORE`

#### Verification Methods

- unit/integration: `cargo test -q -p memd-server store_item_records_source_linked_event -- --nocapture`
- workflow: `memd remember`, `memd checkpoint`, `memd ingest`, `memd hook capture`, `memd hook spill --apply`, then `cat .memd/state/raw-spine.jsonl`
- adversarial: write through multiple ingress paths and confirm source metadata survives into raw spine and compiled event output
- migration: not required in first audit
- cross-harness: deferred until Phase A paths are exercised from more than one harness pack

#### Failure Modes

- path writes memory but skips raw spine
- raw spine record loses `source_system` or `source_path`
- compiled event output omits the raw spine section
- server timeline strips source metadata on candidate or canonical writes

#### Notes

- commands in scope: `memd remember`, `memd checkpoint`, `memd ingest`, `memd hook capture`, `memd hook spill --apply`
- pass means each path writes a source-linked raw spine record, server timeline keeps source metadata, and compiled bundle events surface the raw spine
- benchmark_feature_id: `feature.raw_truth_spine`

### FEATURE-SESSION-CONTINUITY: Fresh Session Continuity

- version: `vNext`
- milestones: `phase-b`
- status: `unverified`
- verification_depth: `strong`

#### User Contract

A fresh session can answer what we are doing, where we left off, what changed,
and what next from compact memd continuity surfaces instead of rebuilding from
transcript.

#### Implementation Surfaces

- `crates/memd-client/src/runtime/resume/mod.rs`
- `crates/memd-client/src/runtime/resume/wakeup.rs`
- `crates/memd-client/src/render/render_summary.rs`
- `crates/memd-client/src/bundle/memory_surface.rs`

#### Dependencies

- `FEATURE-RAW-TRUTH-SPINE`
- `FEATURE-V1-WORKING-CONTEXT`

#### Verification Methods

- unit/integration: `cargo test -q -p memd-client render_resume_prompt_surfaces_explicit_continuity_answers -- --nocapture`
- unit/integration: `cargo test -q -p memd-client render_handoff_prompt_surfaces_explicit_continuity_answers -- --nocapture`
- workflow: run `memd resume --output .memd --prompt` in a fresh session and confirm the prompt contains the four continuity answers
- adversarial: inject inbox pressure and rehydration backlog and confirm continuity answers stay explicit instead of collapsing into mixed sections
- migration: not required in first audit
- cross-harness: rerun the same continuity check through handoff and wake surfaces

#### Failure Modes

- resume requires reading multiple sections to infer continuity
- handoff and wake pages disagree about current task or next action
- prompt keeps continuity implicit inside working or inbox noise

#### Notes

- pass means the first continuity block answers: what are we doing, where did we leave off, what changed, what next
- benchmark_feature_id: `feature.session_continuity`
- working-memory traces now carry typed-memory families so verification can prove type-aware retrieval instead of only flat working-set recall
- canonical promotion is stage-aware and visible in server tests: candidate items promote to canonical stage and stay canonical in storage
- compiled memory quality reports now probe `session_continuity`, and benchmark artifacts surface that typed-retrieval evidence in `latest.md`
- the benchmark evidence path now carries `typed_retrieval_probe=session_continuity` so typed retrieval shows up in scorecards, not only in raw traces

### FEATURE-V1-CORE-STORE: Durable Typed Memory Storage

- version: `v1`
- milestones: `v1`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

`memd remember` and equivalent store paths create durable typed memory records that survive later retrieval and resume.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`
- `POST /memory/store`

#### Dependencies

- none

#### Verification Methods

- unit/integration: `cargo test -p memd-server --quiet`
- workflow: `memd remember ...` followed by `memd resume` or `memd search`
- adversarial: store a project fact, add synced `resume_state` noise, and confirm later recall still surfaces the fact
- migration: not required in first audit
- cross-harness: deferred until bridge and attach claims are audited end to end

#### Failure Modes

- store succeeds but later hot-path recall fails
- memory persists in SQLite but is omitted from compact context or working memory
- newer derived session state crowds out durable operator-authored memory

#### Notes

- storage exists, but recall is the higher-risk part of the contract
- current regression coverage includes keeping a recalled project fact visible in bundle memory during resume
- first Phase C slice now exposes top-level typed-memory families in lookup and bundle surfaces so retrieval no longer reads like one flat bucket
- benchmark_feature_id: `feature.v1.core.store`

### FEATURE-V1-CORE-SEARCH: Bounded Memory Search

- version: `v1`
- milestones: `v1`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Users can search memory through stable routes and receive bounded, inspectable results instead of unbounded transcript dumps.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`
- `POST /memory/search`

#### Dependencies

- `FEATURE-V1-CORE-STORE`

#### Verification Methods

- unit/integration: `cargo test -p memd-server --quiet`
- workflow: `memd search --query "..." --limit ...`
- adversarial: search with overlapping local, synced, and project memories and confirm limits and route behavior stay stable
- migration: not required in first audit
- cross-harness: deferred until multiple client harnesses are audited against the same server state

#### Failure Modes

- search returns relevant records without respecting limits
- route bias hides durable project facts behind local or synced noise
- search works as inspection tooling but not as a reliable retrieval surface for real sessions

#### Notes

- the roadmap requires bounded responses and stable routing, so search needs explicit audit separate from store
- retrieval bias found in the memory audit is a risk here too, not just in resume
- benchmark_feature_id: `feature.v1.core.search`

### FEATURE-V1-LIFECYCLE-REPAIR: Verify, Expire, Supersede, And Dedupe

- version: `v1`
- milestones: `v1`
- status: `partial`
- verification_depth: `exhaustive`

#### User Contract

Operators can repair memory state by verifying, expiring, superseding, and deduplicating records so stale beliefs stop winning.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-schema/src/lib.rs`
- `POST /memory/repair`

#### Dependencies

- `FEATURE-V1-CORE-STORE`

#### Verification Methods

- unit/integration: `cargo test -p memd-server superseded_memory_drops_out_after_manual_correction_loop -- --nocapture`
- workflow: `memd repair ...`, `memd verify ...`, or `memd expire ...` followed by `memd resume`
- adversarial: supersede a stale belief, store the correction, and confirm the stale memory drops out under current-task retrieval
- migration: not required in first audit
- cross-harness: deferred until one correction loop is proven across multiple clients

#### Failure Modes

- repair metadata updates but stale memory still survives hot-path retrieval
- duplicate or superseded records remain visible in compact context
- lifecycle tools work for inspection views but do not change later behavior

#### Notes

- current low-level correction lifecycle is promising: a superseded stale memory can drop out after a manual correction loop
- duplicate revival is also covered via `tests::explicit_store_revives_superseded_canonical_duplicate`
- the unresolved gap is product UX, not the underlying supersede mechanics alone
- benchmark_feature_id: `feature.v1.lifecycle.repair`

### FEATURE-V1-WORKING-CONTEXT: Compact Context By Route And Intent

- version: `v1`
- milestones: `v1`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

`memd` can fetch compact context for the current task so recalled memory is small enough for the hot path without losing critical facts.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/routing.rs`
- `GET /memory/context`
- `GET /memory/context/compact`

#### Dependencies

- `FEATURE-V1-CORE-STORE`
- `FEATURE-V1-CORE-SEARCH`

#### Verification Methods

- unit/integration: `cargo test -p memd-server --quiet`
- workflow: `memd resume` and inspect compact context in generated bundle output
- adversarial: add multiple synced `resume_state` records and confirm a durable project fact remains visible under `current_task`
- migration: not required in first audit
- cross-harness: rerun compact-context checks from at least two attach clients once client parity is audited

#### Failure Modes

- route order hard-crowds durable project memory out before score matters
- compact context stays fresh but loses the user-important memory
- the right memory exists in storage but never reaches the rendered prompt

#### Notes

- the memory audit identified and reproduced a `current_task` crowd-out bug in `build_context`
- that bug has regression coverage now, but the feature still remains `unverified` until full audit
- benchmark_feature_id: `feature.v1.working.context`

### FEATURE-V1-WORKING-MEMORY: Managed Working Memory With Budget Signals

- version: `v1`
- milestones: `v1`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Users can fetch managed working memory with explicit budget, admission, eviction, and rehydration signals that remain useful under pressure.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/working.rs`
- `crates/memd-server/src/main.rs`
- `GET /memory/working`

#### Dependencies

- `FEATURE-V1-WORKING-CONTEXT`

#### Verification Methods

- unit/integration: `cargo test -p memd-server --quiet && cargo test -p memd-client --quiet`
- workflow: `memd resume` and inspect working-memory sections in bundle artifacts
- adversarial: exceed the working-memory budget with mixed local, synced, and project records and verify critical records stay visible with budget reporting
- migration: not required in first audit
- cross-harness: compare working-memory output across at least two attached harnesses once attach parity is audited

#### Failure Modes

- budget reporting exists but important records are evicted first
- working-memory records are technically returned but not represented in bundle output
- rehydration signals are present in schema only and do not support actual session recovery

#### Notes

- resume currently writes a derived `resume_state` record on every run, which can compete with durable memory in this surface
- current client regression proves a recalled project fact can remain visible in generated bundle memory when retrieval returns it
- benchmark_feature_id: `feature.v1.working.memory`

### FEATURE-V1-EXPLAIN: Explainability For Ranking And Existence

- version: `v1`
- milestones: `v1`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Operators can ask why a memory exists and why it ranked the way it did, instead of treating retrieval as a black box.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/inspection.rs`
- `GET /memory/explain`

#### Dependencies

- `FEATURE-V1-CORE-SEARCH`
- `FEATURE-V1-WORKING-CONTEXT`

#### Verification Methods

- unit/integration: `cargo test -p memd-server --quiet`
- workflow: `memd explain <memory-id>`
- adversarial: explain a memory that lost ranking behind noise and verify the output exposes the relevant route, scope, or policy reasons
- migration: not required in first audit
- cross-harness: deferred until explain output is confirmed consistent across attached clients

#### Failure Modes

- explain output exists but omits the decisive ranking or provenance reasons
- operators can inspect a memory statically but still cannot diagnose hot-path omission
- explanation drifts from the actual retrieval logic after policy changes

#### Notes

- this feature matters because the current debugging process depends on being able to inspect why retrieval behaved incorrectly
- audit should verify that explain helps real diagnosis rather than just dumping metadata
- benchmark_feature_id: `feature.v1.explain`

### FEATURE-V1-PROVENANCE: Provenance Drilldown To Source Artifacts

- version: `v1`
- milestones: `v1`
- status: `verified`
- verification_depth: `exhaustive`

#### User Contract

Operators can drill from a memory summary down to its source artifacts and provenance trail to judge trust and freshness.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-client/src/render.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/inspection.rs`
- `crates/memd-server/src/store.rs`
- `GET /memory/source`

#### Dependencies

- `FEATURE-V1-CORE-STORE`
- `FEATURE-V1-EXPLAIN`

#### Verification Methods

- unit/integration: `cargo test -p memd-server provenance -- --nocapture`
- workflow: inspect source-memory and explain flows for a stored memory with evidence attached
- adversarial: verify provenance remains drillable after compaction, summarization, or repair of the derived memory
- migration: not required in first audit
- cross-harness: deferred until provenance links are validated in more than one client surface

#### Failure Modes

- summary memory exists without a reachable evidence trail
- source drilldown works for some lanes only and silently fails for others
- provenance metadata is stored but too weak or too indirect to support trust decisions

#### Notes

- `ROADMAP.md` now treats canonical-truth provenance as implemented in core server/client surfaces, and cross-harness validation now has direct proof
- route proof now exists via `tests::source_memory_route_returns_provenance_aggregates_for_filtered_source`
- visible-memory provenance surface is covered via `ui::tests::visible_memory_provenance_trust_reason_exposes_epistemic_state`
- client inspection summary is covered via `render::render_summary::render_memory_summary::tests::render_source_summary_surfaces_provenance_trail_and_tags`
- client source command path is covered via `e2e_tests::source_command_uses_live_source_route_and_bundle_defaults`
- live client request defaults are covered via `main_tests::runtime_memory_tests::runtime_memory_tests_core::source_memory_request_uses_repo_bundle_identity_defaults`
- resume truth summary uses live source provenance via `runtime::resume::tests::truth_summary_uses_top_source_provenance_for_non_live_truth_lanes`
- same source-artifact path now has cross-harness proof via `main_tests::bootstrap_harness_tests::provenance_source_path_survives_across_codex_and_openclaw_memory_surfaces`
- benchmark_feature_id: `feature.v1.provenance`

### FEATURE-V1-BUNDLE-ATTACH: Bundle Configuration And Client Attach Flow

- version: `v1`
- milestones: `v1`
- status: `verified`
- verification_depth: `exhaustive`

#### User Contract

Project bundles can configure runtime defaults, and attach flows make `memd` usable from supported clients without manual reconfiguration each session.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-client/src/render.rs`
- `ROADMAP.md`
- `memd attach`

#### Dependencies

- `FEATURE-V1-WORKING-CONTEXT`
- `FEATURE-V1-WORKING-MEMORY`

#### Verification Methods

- unit/integration: `cargo test -p memd-client --quiet`
- workflow: run `memd attach`, inspect generated attach snippet, then run `memd resume` from the configured bundle
- adversarial: change bundle route, intent, workspace, or base URL and verify attach-driven clients pick up the right overlay instead of stale defaults
- migration: not required in first audit
- cross-harness: required, because the requirement claims Claude Code, Codex, Mission Control, and OpenClaw attach through the same control plane

#### Failure Modes

- attach script is generated but does not configure a usable runtime
- bundle defaults drift from actual resume behavior
- one harness attaches cleanly while others silently degrade or bypass bundle config

#### Notes

- this contract spans beyond a single CLI snippet; the real audit must cover actual harness attach behavior
- generated attach startup now has execution proof via `evaluation_runtime_tests::evaluation_runtime_tests_bundle_setup::attach_snippet_executes_wake_with_bundle_route_intent_and_env_defaults`
- generated Codex and Claude Code launcher profiles now have parity proof via `evaluation_runtime_tests::evaluation_runtime_tests_bundle_setup::codex_and_claude_profiles_execute_same_bundle_defaults`
- generated OpenClaw launcher profile now has parity proof via `evaluation_runtime_tests::evaluation_runtime_tests_bundle_setup::openclaw_profile_executes_same_bundle_defaults`
- bundle startup surfaces now prove route, intent, project, and workspace overlays survive actual launcher execution instead of string inspection alone
- benchmark_feature_id: `feature.v1.bundle.attach`

### FEATURE-V1-WAKE-PACKET: Wake Packet Compiler

- version: `v1`
- milestones: `v1`
- status: `verified`
- verification_depth: `exhaustive`

#### User Contract

`memd wake` compiles a compact, action-ready packet that keeps current task, durable truth, and next action visible without forcing transcript rereads.

#### Implementation Surfaces

- `crates/memd-client/src/runtime/resume/wakeup.rs`
- `crates/memd-client/src/render/render_summary.rs`
- `crates/memd-client/src/workflow/autoresearch/mod.rs`
- `crates/memd-client/src/verification/verifier_runtime.rs`

#### Dependencies

- `FEATURE-RAW-TRUTH-SPINE`
- `FEATURE-SESSION-CONTINUITY`
- `FEATURE-V1-PROVENANCE`

#### Verification Methods

- unit/integration: `cargo test -p memd-client wakeup_markdown_surfaces_durable_truth_without_verbose_mode -- --nocapture`
- unit/integration: `cargo test -p memd-client run_prompt_efficiency_loop -- --nocapture`
- unit/integration: `cargo test -p memd-client runtime::resume::wakeup::tests::wakeup_markdown_stays_compact_under_pressure_and_keeps_continuity -- --nocapture`
- unit/integration: `cargo test -p memd-client main_tests::autoresearch_evolution_tests::run_prompt_surface_loop_records_high_pressure_diagnostics -- --nocapture`
- unit/integration: `cargo test -p memd-client main_tests::runtime_verification_tests::run_verify_feature_command_executes_seeded_wake_verifier -- --nocapture`
- workflow: run `memd wake --output .memd --summary` and confirm the packet stays compact while preserving doing, left_off, changed, and next
- adversarial: add repeated context and confirm the wake packet still prefers compact durable truth over reread-style expansion
- migration: not required in first audit
- cross-harness: compare wake startup execution across Codex, Claude Code, and OpenClaw launcher surfaces and confirm the same bundle target plus agent identity survive launch; keep route/intent execution covered by the attach startup proof

#### Failure Modes

- wake becomes a transcript dump instead of a packet
- packet shrinks but loses next action or durable truth
- token savings exist only on paper and not in the compiled output

#### Notes

- `render_bundle_wakeup_markdown` is the current compiler surface
- `run_prompt_efficiency_loop` compares `core_prompt_tokens` against `estimated_prompt_tokens`
- wake packet pressure path now has regression coverage for compact output under high-pressure snapshots
- wake packet efficiency artifacts now record pressure diagnostics and warning reasons alongside token savings
- `verifier.feature.bundle.wake` now consumes the live wake-packet efficiency artifact instead of only checking that `wake.md` exists
- wake correctness already has coverage; this feature focuses on measurable packet compaction
- supported harness startup parity now has explicit wake proof via `main_tests::hive_coordination_tests::wake_packet_cross_harness_profiles_keep_same_bundle_defaults`, while route/intent execution remains covered by `attach_snippet_executes_wake_with_bundle_route_intent_and_env_defaults`

## v2 Features

### FEATURE-V2-TRUST-CONTRADICTION: Trust-Aware Belief State

- version: `v2`
- milestones: `v2`
- status: `partial`
- verification_depth: `exhaustive`

#### User Contract

Each durable belief preserves source trust, freshness, verification, and contradiction state, and low-trust or contested beliefs are demoted instead of masquerading as truth.

#### Implementation Surfaces

- `crates/memd-server/src/store.rs`
- `crates/memd-server/src/inspection.rs`
- `crates/memd-server/src/working.rs`
- `crates/memd-server/src/main.rs`
- `GET /memory/policy`
- `GET /memory/explain`
- `GET /memory/working`

#### Dependencies

- `FEATURE-V1-CORE-STORE`
- `FEATURE-V1-LIFECYCLE-REPAIR`
- `FEATURE-V1-EXPLAIN`
- `FEATURE-V1-PROVENANCE`

#### Verification Methods

- unit/integration: `cargo test -p memd-server contested_synthetic_items_collect_policy_reasons -- --nocapture`
- workflow: store canonical, synthetic, stale, contested, and superseded records with different trust scores; inspect `memd explain`, `memd working`, and `memd policy`
- adversarial: make a low-trust contested record share the same retrieval neighborhood as a verified record and confirm the verified item still wins
- migration: not required in first audit
- cross-harness: rerun the same trust/contradiction case from at least two clients once attach parity exists

#### Failure Modes

- trust metadata is stored but ignored by ranking
- contested records still win hot-path retrieval
- policy snapshot lies about the active trust floor

#### Notes

- this is the current audit pressure point, because v1 proved explainability exists but not that trust or contradiction handling actually changes behavior
- ranking coverage now exists via `working::tests::active_recent_canonical_items_rank_above_stale_contested_items`
- freshness weighting is covered via `working::tests::recently_verified_items_rank_above_unverified_items`
- contradiction/trust-floor policy reasons are covered via `working::tests::contested_synthetic_items_collect_policy_reasons`

### FEATURE-V2-BRANCHABLE-BELIEFS: Branchable Competing Beliefs

- version: `v2`
- milestones: `v2`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Conflicting beliefs can coexist on separate branches, be inspected as siblings, and be preferred or superseded without overwriting the other branches.

#### Implementation Surfaces

- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/inspection.rs`
- `crates/memd-server/src/keys.rs`
- `crates/memd-client/src/main.rs`
- `POST /memory/store`
- `POST /memory/promote`
- `GET /memory/explain`
- `GET /memory/context`

#### Dependencies

- `FEATURE-V2-TRUST-CONTRADICTION`
- `FEATURE-V1-CORE-STORE`
- `FEATURE-V1-LIFECYCLE-REPAIR`

#### Verification Methods

- unit/integration: `cargo test -p memd-server --quiet`
- workflow: store the same canonical fact on `mainline` and `fallback` branches, promote one branch to preferred, and verify both remain discoverable
- adversarial: supersede one branch and confirm sibling inspection still shows the remaining branch rather than collapsing the entire belief family
- migration: not required in first audit
- cross-harness: rerun the branch case from at least two clients once attach parity exists

#### Failure Modes

- dedupe collapses branches
- preferred flag deletes siblings
- branch identity disappears from explain output

#### Notes

- current code already tracks `belief_branch` and sibling inspection, so this contract is about branch behavior staying observable and reversible under load

### FEATURE-V2-REVERSIBLE-COMPRESSION: Summary-First Retrieval With Raw Evidence Recovery

- version: `v2`
- milestones: `v2`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Compressed resume output stays small, but any important summary can be expanded back into source evidence, event trail, and entity context without losing the original trail.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-client/src/render.rs`
- `crates/memd-server/src/inspection.rs`
- `crates/memd-server/src/main.rs`
- `GET /memory/context/compact`
- `GET /memory/source`
- `GET /memory/explain`
- `GET /memory/timeline`

#### Dependencies

- `FEATURE-V1-WORKING-CONTEXT`
- `FEATURE-V1-EXPLAIN`
- `FEATURE-V1-PROVENANCE`

#### Verification Methods

- workflow: run `memd resume` and then expand a returned item through `memd explain`, `memd source`, and `memd timeline`
- adversarial: force compact-context truncation, then confirm the same memory is still recoverable from source and evidence surfaces
- unit/integration: `cargo test -p memd-client --quiet && cargo test -p memd-server --quiet`
- migration: not required in first audit
- cross-harness: rerun the compression/recovery case from at least two clients once attach parity exists

#### Failure Modes

- summary loses source IDs
- compact context omits key evidence
- rehydration trail points at a synthetic summary only

#### Notes

- this is the contract that prevents good compression from becoming lossy amnesia

### FEATURE-V2-WORKING-POLICY-GOVERNOR: Explicit Working-Memory Admission And Rehydration

- version: `v2`
- milestones: `v2`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Working memory declares its admission limit, truncation behavior, rehydration queue, and trust floor so hot-path memory stays explainable under pressure.

#### Implementation Surfaces

- `crates/memd-server/src/working.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-client/src/main.rs`
- `GET /memory/working`
- `GET /memory/policy`
- `memd working`
- `memd resume`

#### Dependencies

- `FEATURE-V1-WORKING-MEMORY`
- `FEATURE-V1-WORKING-CONTEXT`
- `FEATURE-V2-TRUST-CONTRADICTION`

#### Verification Methods

- workflow: request working memory at different limits and verify admission, eviction, rehydration, and budget fields all change coherently
- adversarial: overfill with mixed-source records and confirm the rehydration queue still prioritizes the right evicted items
- unit/integration: `cargo test -p memd-server --quiet`
- migration: not required in first audit
- cross-harness: rerun the same working-memory pressure case from at least two clients once attach parity exists

#### Failure Modes

- admission limit is reported but not enforced
- eviction and rehydration disagree
- trust floor exists in policy only

#### Notes

- this is the concrete guardrail around hot-path behavior, and it should stay smaller than the broader retrieval-policy contract

### FEATURE-V2-RETRIEVAL-POLICY-LEARNING: Feedback-Driven Ranking

- version: `v2`
- milestones: `v2`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Retrieval ranking is not just a static heuristic table; repeated retrieval feedback can alter what surfaces first, while policy inspection stays readable.

#### Implementation Surfaces

- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/working.rs`
- `crates/memd-server/src/inspection.rs`
- `crates/memd-server/src/store.rs`
- `GET /memory/policy`
- `GET /memory/explain`
- `GET /memory/working`
- `POST /memory/search`

#### Dependencies

- `FEATURE-V2-TRUST-CONTRADICTION`
- `FEATURE-V1-CORE-SEARCH`
- `FEATURE-V1-WORKING-CONTEXT`

#### Verification Methods

- workflow: run repeated search, context, working, and explain flows on the same memory set and verify feedback records accumulate and policy hooks stay stable
- adversarial: add fresh but lower-trust noise and confirm the policy does not regress into freshness-only ranking
- migration: not required in first audit
- cross-harness: compare policy snapshot and surfaced ranks from two clients after the same feedback history

#### Failure Modes

- feedback is recorded but never affects rank
- policy hooks drift from actual ranking logic
- freshness overwhelms trust and contradiction signals

#### Notes

- current repo already records retrieval feedback and exposes policy snapshots, so the audit should prove those signals actually shape behavior over time

## v3 Features

### FEATURE-V3-WORKSPACE-SHARED-RETRIEVAL: Workspace-Aware Shared Memory Retrieval

- version: `v3`
- milestones: `v3`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

When a workspace is active, `memd` surfaces matching workspace memory ahead of cross-workspace noise so shared project context stays visible in `resume`, `handoff`, and workspace inspection flows.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`
- `GET /memory/context`
- `GET /memory/workspaces`
- `memd resume`
- `memd handoff`
- `memd workspaces`

#### Dependencies

- `FEATURE-V1-WORKING-CONTEXT`
- `FEATURE-V1-WORKING-MEMORY`
- `FEATURE-V2-TRUST-CONTRADICTION`

#### Verification Methods

- unit/integration: `cargo test -p memd-server --quiet && cargo test -p memd-client --quiet`
- workflow: load a workspace-scoped memory, then run `memd workspaces` and `memd resume` to confirm the matching workspace lane is the one surfaced
- adversarial: mix a matching workspace fact with heavier cross-workspace synced noise and verify the matching workspace fact still survives the hot path
- migration: not required in first audit
- cross-harness: rerun the same workspace-scoped recall case from two attached clients using the same workspace overlay

#### Failure Modes

- workspace-scoped memory exists but is hidden behind unrelated synced state
- the workspace lane is present in inspection views but not in the resume or handoff prompt
- cross-workspace noise wins because routing or truncation ignores the active workspace

#### Notes

- this contract is grounded in the current `workspace_memory` API and the workspace-preferring resume/handoff tests in the repo
- the audit must prove the behavior under real mixed-noise conditions, not just in isolated workspace listings

### FEATURE-V3-VISIBILITY-BOUNDARIES: Permission-Aware Memory Visibility

- version: `v3`
- milestones: `v3`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Private, workspace, and broader shared memory boundaries stay explicit so the wrong agent, session, or workspace cannot see memory it should not see.

#### Implementation Surfaces

- `crates/memd-schema/src/lib.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/repair.rs`
- `crates/memd-server/src/store.rs`
- `POST /memory/store`
- `POST /memory/repair`
- `GET /memory/workspaces`
- `POST /coordination/claims`

#### Dependencies

- `FEATURE-V1-CORE-STORE`
- `FEATURE-V1-LIFECYCLE-REPAIR`
- `FEATURE-V2-TRUST-CONTRADICTION`

#### Verification Methods

- unit/integration: `cargo test -p memd-server --quiet`
- workflow: store private, workspace, and shared records, then confirm `memd search`, `memd workspaces`, and `memd explain` respect the declared visibility
- adversarial: attempt to read a private or foreign-workspace record from another workspace or session and verify it does not leak through retrieval or inspection
- migration: not required in first audit
- cross-harness: repeat the same visibility check from at least two clients attached to the same server

#### Failure Modes

- visibility metadata is stored but ignored by retrieval
- repair can rewrite scope but the hot path still exposes the wrong audience
- workspace-scoped records leak into a broader session or provider lane

#### Notes

- the schema and store already carry `workspace` and `visibility`, so the audit should focus on enforcement, not field existence
- this is the contract that keeps shared memory useful without collapsing privacy boundaries

### FEATURE-V3-HANDOFF-CONTINUITY: Delegation Memory Across Agents And Humans

- version: `v3`
- milestones: `v3`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

`memd handoff` preserves reasoning state, source context, and next-recovery guidance so delegation can continue without re-deriving the same setup on the next agent or machine.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`
- `POST /coordination/messages/send`
- `GET /coordination/messages/inbox`
- `POST /coordination/claims`
- `POST /coordination/sessions/upsert`
- `memd handoff`
- `memd resume`

#### Dependencies

- `FEATURE-V3-WORKSPACE-SHARED-RETRIEVAL`
- `FEATURE-V3-VISIBILITY-BOUNDARIES`
- `FEATURE-V1-PROVENANCE`
- `FEATURE-V2-REVERSIBLE-COMPRESSION`

#### Verification Methods

- unit/integration: `cargo test -p memd-client --quiet && cargo test -p memd-server --quiet`
- workflow: run `memd handoff --output ...` and confirm the generated prompt includes workspace, source, and recovery state that a second agent can resume from
- adversarial: point the handoff at a stale or missing target session and confirm the output degrades safely instead of fabricating continuity
- migration: not required in first audit
- cross-harness: transfer the same handoff bundle across two different clients and verify the target session and workspace survive the round trip

#### Failure Modes

- handoff preserves a prompt but drops the reasoning trail that made the prompt actionable
- source context is present in storage but omitted from the handoff artifact
- stale target-session references create false confidence in delegation continuity

#### Notes

- the current code already has explicit handoff and coordination surfaces, so the audit should verify that they preserve usable state rather than only emitting a summary
- this is the contract that separates delegation memory from a plain resume snapshot

### FEATURE-V3-SYNCED-HOT-LANE: Canonical Short-Term State Sync Across Clients

- version: `v3`
- milestones: `v3`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

Canonical short-term state such as focus, blockers, recovery steps, branch scope, ports, and presence updates sync quickly enough that another attached client can continue the same live work without guessing.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`
- `memd heartbeat`
- `memd resume`
- `memd refresh`
- `POST /coordination/sessions/upsert`
- `GET /memory/working`

#### Dependencies

- `FEATURE-V1-BUNDLE-ATTACH`
- `FEATURE-V2-WORKING-POLICY-GOVERNOR`
- `FEATURE-V3-HANDOFF-CONTINUITY`

#### Verification Methods

- unit/integration: `cargo test -p memd-client --quiet && cargo test -p memd-server --quiet`
- workflow: update live state on one client, then run `memd resume` or `memd heartbeat` from a second client and confirm the canonical lane reflects the new state
- adversarial: inject stale resume-state noise and verify the canonical short-term record still wins over older or less relevant session artifacts
- migration: not required in first audit
- cross-harness: reproduce the same live-state update across two machines or harnesses and confirm the shared lane converges

#### Failure Modes

- short-term state syncs only inside one bundle and never becomes shared canonical truth
- the right state exists but arrives too late to influence the next turn
- stale heartbeat or resume data overrides the current live branch

#### Notes

- the repo already derives and stores `resume_state` and hive-group/session updates, so the audit should prove those records are functioning as a canonical live lane
- this is the contract that makes coworking feel synchronous instead of replay-based

### FEATURE-V3-MERGE-COLLISION-GOVERNOR: Merge, Divergence, And Provider Collision Handling

- version: `v3`
- milestones: `v3`
- status: `unverified`
- verification_depth: `exhaustive`

#### User Contract

When local and shared truth diverge, or when multiple providers and sessions collide, `memd` reports the conflict and preserves the competing state instead of silently overwriting it.

#### Implementation Surfaces

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`
- `Commands::Awareness`
- `memd awareness`
- `POST /coordination/sessions/upsert`
- `POST /coordination/claims`
- `GET /coordination/claims`

#### Dependencies

- `FEATURE-V3-SYNCED-HOT-LANE`
- `FEATURE-V3-VISIBILITY-BOUNDARIES`
- `FEATURE-V2-BRANCHABLE-BELIEFS`

#### Verification Methods

- unit/integration: `cargo test -p memd-client --quiet && cargo test -p memd-server --quiet`
- workflow: run `memd awareness` and confirm session, workspace, and base-url collisions are surfaced instead of hidden
- adversarial: simulate duplicate session identity or provider base_url reuse and verify warnings appear without one provider silently overwriting the other
- migration: not required in first audit
- cross-harness: compare the same collision case from two clients and confirm both report the same divergence rather than normalizing it away

#### Failure Modes

- collision detection exists only in local views and not in shared awareness
- merge logic favors the newest writer even when the session identity is ambiguous
- provider-specific truth gets collapsed into a single lane with no provenance for the conflict

#### Notes

- the client already has awareness and collision-summary code paths, so the audit should verify that they are enforcement signals, not just status decoration
- this contract is the backstop against multi-provider overwrite bugs
