# Benchmark Registry 10-Star Design

## Goal

Create a 10-star benchmark system for `memd` that lets a single maintainer know,
with explicit evidence, whether every important feature actually works, whether
continuity is holding, whether the app is improving over time, and whether
`with memd` is meaningfully better than `no memd`.

The benchmark system must:

- cover the full app, not a small hand-picked subset
- make continuity and seamless memory the top product bar
- measure quality, not only feature existence
- measure token efficiency as a first-class outcome
- detect drift automatically
- support overnight gap discovery through `autoresearch`
- support consolidation through `autodream`
- support controlled improvement through `self-evolution`
- stay auditable and hard to game

## Why This Exists

The current product problem is not only missing features.

The bigger problem is quality:

- memory is not yet seamless
- continuity is not yet perfect
- visible truth and runtime truth can drift
- features can exist but still feel fragile
- token cost can stay too high relative to the benefit
- a fast-moving solo maintainer needs a way to know what actually works

The benchmark system must therefore answer:

- what are all the features?
- which ones are continuity-critical?
- which ones are weak?
- where is drift happening?
- where is `with memd` still not beating `no memd`?
- what should autoresearch work on next?
- what should autodream consolidate?
- what may self-evolution propose changing safely?

## 10-Star Shape

The best shape is not one loop per feature and not one flat checklist.

It is a canonical benchmark graph with explicit links between:

- pillars
- families
- features
- journeys
- loops
- scorecards
- evidence
- gates
- runtime policies

This is the right design because `memd` is not trying to prove that commands
exist. It is trying to prove that the app behaves like a seamless memory control
plane.

## Core Design Principles

### 1. Product Truth Beats Surface Count

The benchmark system should optimize for the app goal:

- seamless memory
- durable continuity
- visible truth
- bounded recovery effort
- low token waste
- drift resistance

A feature is not strong because it exists in the CLI or API. A feature is strong
when the user-facing contract holds with evidence.

### 2. Continuity Is The Top Bar

The most important benchmark question is:

- can the next turn, next tab, next session, next harness, or next person
  continue correctly without reconstructing state by hand?

Any break here is a first-class product failure.

### 3. Quality Matters As Much As Coverage

Every feature should be benchmarked for:

- correctness
- continuity
- reliability
- token efficiency
- latency
- provenance
- inspectability
- boundary safety
- cross-harness parity
- drift resistance

Coverage is necessary. It is not sufficient.

### 4. Drift Prevention Is A Product Category

For `memd`, drift is continuity failure.

Important drift classes:

- memory drift
- continuity drift
- surface drift
- docs drift
- harness drift
- policy drift
- benchmark drift

The system must benchmark drift directly, not treat it as incidental cleanup.

### 5. Evidence First

Scores without evidence are gameable.

Loop results, quality scores, and self-evolution decisions must all anchor back
to evidence objects:

- command output
- API output
- latency samples
- token samples
- compiled pages
- handoff packets
- resume snapshots
- artifact diffs
- screenshots when needed
- loop reports

### 6. Gap Discovery Must Be Native

The benchmark registry defines expected truth.

`autoresearch` and `autodream` should still discover:

- missing loops
- missing feature-to-journey links
- weak scores
- new drift patterns
- missing registry coverage
- policy settings that harm quality
- places where `with memd` still does not outperform `no memd`

But discovery must propose truth changes, not silently rewrite them.

### 7. The System Must Stay Operable For One Maintainer

The benchmark system is only good if one person can run it, trust it, and act
on it quickly.

That means the design must optimize for:

- a small always-on slice
- bounded nightly cost
- clear morning triage
- obvious next actions
- no requirement to reread benchmark internals to understand what broke

### 8. Benchmarking Must Not Become A Second Unbounded Product

The benchmark system exists to improve `memd`, not to grow into a second
unbounded complexity sink.

Important rule:

- every benchmark surface must justify its ongoing cost
- low-value or duplicate loops should be removable without weakening trust
- the minimal continuity-critical slice must remain useful even if the full
  benchmark graph is not yet complete

## Canonical Artifact Model

The machine-truth layer should live in:

- `docs/verification/benchmark-registry.json`
- `docs/verification/benchmark-registry.schema.json`

Human and operator views should be generated from that canonical registry plus
loop telemetry:

- `docs/verification/BENCHMARKS.md`
- `docs/verification/LOOPS.md`
- `docs/verification/COVERAGE.md`
- `docs/verification/SCORES.md`

Runtime outputs should live under:

- `.memd/loops/`
- `.memd/telemetry/`

Important rule:

- the canonical JSON registry is the only hand-edited machine-truth file
- generated markdown is never edited directly
- telemetry never mutates registry truth directly

## Relationship To Existing Verification Files

`FEATURES.md` remains valuable.

It should stay the human contract ledger:

- reusable feature descriptions
- user contracts
- implementation surfaces
- verification methods
- notes and caveats

The benchmark registry should not replace it. The benchmark registry should add
the machine-usable structure needed to:

- map features to loops
- map features to journeys
- compute scorecards
- track drift
- schedule runs
- compare `no memd` vs `with memd`

This means the right architecture is:

- `FEATURES.md` for human-readable product contracts
- `benchmark-registry.json` for machine-readable benchmark truth

## Registry Top-Level Shape

Recommended top-level sections:

- `version`
- `app_goal`
- `quality_dimensions`
- `tiers`
- `pillars`
- `families`
- `features`
- `journeys`
- `loops`
- `scorecards`
- `evidence`
- `gates`
- `baseline_modes`
- `runtime_policies`
- `generated_at`

This is intentionally broader than a feature list because the benchmark system
has to reason about real product behavior.

## Product Pillars

The registry should model pillars above families. Families are implementation-
facing. Pillars are product-facing.

Recommended pillars:

### Memory Continuity

The core product promise:

- wake
- resume
- compact context
- working memory
- handoff
- attach
- checkpoint
- hook capture
- session and tab continuity

### Visible Memory

The human-facing truth layer:

- compiled memory pages
- lane pages
- item pages
- event pages
- quality scoring
- workbench and inspection surfaces

### Truth And Repair

The durable memory and correction pillar:

- store
- candidate
- promote
- verify
- expire
- repair
- belief branches
- contradiction handling
- source lanes

### Coordination

The shared work and hive pillar:

- awareness
- heartbeat
- hive
- hive-project
- hive-join
- claims
- messages
- tasks
- shared workspace state

### Knowledge Bridge

The external knowledge pillar:

- Obsidian import and sync
- writeback
- compile
- roundtrip
- semantic sync
- multimodal attachments

### Autonomous Improvement

A first-class product pillar, not an internal helper:

- autodream
- autoresearch
- self-evolution
- telemetry
- durability re-checks
- portability tagging
- quality uplift over time

### Cross-Harness Runtime

The supported harness runtime promise:

- Codex
- Claude Code
- OpenClaw
- Hermes
- Agent Zero
- OpenCode

### Drift Prevention

An explicit pillar for:

- runtime/docs/registry alignment
- parity alignment
- benchmark coverage integrity
- continuity drift detection

### Efficiency

The proof that the app pays for itself:

- token reduction
- reread reduction
- bounded prompt size
- event noise reduction
- `no memd` vs `with memd` advantage

## Families

Each feature belongs to one family, and each family belongs to one pillar.

Recommended families:

- `core-memory`
- `retrieval`
- `inspection`
- `bundle-runtime`
- `capture-compaction`
- `visible-memory`
- `coordination-hive`
- `cross-harness`
- `obsidian`
- `rag-semantic`
- `multimodal`
- `skills-policy`
- `autodream`
- `autoresearch`
- `self-evolution`
- `drift-prevention`
- `efficiency`

## Tiers

Tiers should exist to help a solo maintainer prioritize what must stay healthy.

### Tier 0: Continuity-Critical

These always come first:

- wake
- resume
- working memory
- compact context
- handoff
- attach
- checkpoint
- hook capture
- session and tab identity
- continuity drift loops

### Tier 1: Truth-Critical

These define what the system knows and whether it can correct itself:

- store
- search
- lookup
- verify
- expire
- repair
- explain
- source lanes
- entity and timeline
- policy truth

### Tier 2: Workflow-Critical

These matter heavily for real use but sit above the continuity/truth substrate:

- hive coordination
- tasks
- claims
- messages
- Obsidian workflows
- semantic sync
- visible memory workbench
- autonomous improvement workflows

### Tier 3: Supporting

These still matter, but should not outrank the core continuity substrate:

- inspiration
- convenience tooling
- secondary inspection helpers
- lower-risk support surfaces

## Quality Dimensions

Every important feature should declare its relevant quality dimensions.

Recommended dimensions:

- `correctness`
- `continuity`
- `reliability`
- `token_efficiency`
- `latency`
- `provenance`
- `inspectability`
- `boundary_safety`
- `cross_harness_parity`
- `drift_resistance`

Not every feature will weight them equally, but continuity-critical features
must always score continuity, correctness, reliability, and token efficiency.

## Drift Categories

The registry should standardize drift types:

- `memory-drift`
- `continuity-drift`
- `surface-drift`
- `docs-drift`
- `harness-drift`
- `policy-drift`
- `benchmark-drift`

Every feature, journey, or family should declare the relevant drift risks.

## Baseline Modes

The benchmark system should compare multiple runtime modes, not only the happy
path.

Recommended baseline modes:

- `baseline.no-memd`
- `baseline.with-memd`
- `baseline.with-memd-semantic`
- `baseline.with-memd-workspace`
- `baseline.degraded-backend`

Important rule:

- `with memd` only counts as strong when it meaningfully beats `no memd` on the
  product promise

## Feature Records

Every feature record should contain enough information to benchmark it without
manual rediscovery.

Recommended fields:

- `id`
- `name`
- `pillar`
- `family`
- `tier`
- `continuity_critical`
- `user_contract`
- `source_contract_refs`
- `commands`
- `routes`
- `files`
- `journey_ids`
- `loop_ids`
- `quality_dimensions`
- `drift_risks`
- `failure_modes`
- `coverage_status`
- `last_verified_at`

Recommended `coverage_status` values:

- `unbenchmarked`
- `auditing`
- `partial`
- `verified`
- `broken`

## Journey Records

Journeys must be first-class records because continuity is mostly a journey
property, not a single-feature property.

Recommended fields:

- `id`
- `name`
- `goal`
- `feature_ids`
- `loop_ids`
- `quality_dimensions`
- `baseline_mode_ids`
- `drift_risks`
- `gate_target`

Important journeys should include:

- setup -> status -> wake -> resume
- remember -> search -> explain -> repair -> resume
- checkpoint -> handoff -> attach -> continue
- hook capture -> event lane -> visible memory refresh
- hive join -> heartbeat -> claims -> tasks -> coordination
- Obsidian import -> compile -> writeback -> open
- semantic sync -> resume fallback -> inspect provenance
- autoresearch -> accepted finding -> autodream consolidation -> self-evolution

## Loop Records

Loops should be reusable records, not embedded inside feature entries.

That is the strongest design because:

- one loop can cover many features
- one feature can require many loops
- drift loops can be shared
- journey loops stay normal instead of duplicated

Recommended fields:

- `id`
- `name`
- `pillar`
- `family`
- `type`
- `covers_features`
- `journey_ids`
- `quality_dimensions`
- `baseline_mode`
- `workflow_probe`
- `adversarial_probe`
- `cross_harness_probe`
- `metrics`
- `guardrails`
- `stop_condition`
- `artifacts_written`
- `status`

Recommended loop types:

- `feature-contract`
- `feature-quality`
- `journey`
- `drift-prevention`
- `cross-harness`
- `performance-cost`
- `adversarial`
- `self-evolution`
- `meta-coverage`

## Evidence Records

Evidence should be a first-class top-level registry concept.

Recommended evidence kinds:

- `command-output`
- `api-response`
- `artifact-diff`
- `latency-sample`
- `token-sample`
- `compiled-page`
- `handoff-packet`
- `resume-snapshot`
- `loop-report`
- `screenshot`

Recommended evidence fields:

- `id`
- `subject_type`
- `subject_id`
- `kind`
- `path_or_ref`
- `captured_at`
- `baseline_mode`
- `supports_dimensions`
- `supports_loops`
- `summary`
- `verdict`

Recommended evidence verdicts:

- `supports`
- `contradicts`
- `partial`

## Scorecards

Scorecards should be derived, not hand-edited.

They should exist at least for:

- feature
- journey
- family
- pillar
- app

Recommended score fields:

- `correctness`
- `continuity`
- `reliability`
- `token_efficiency`
- `latency`
- `provenance`
- `inspectability`
- `boundary_safety`
- `cross_harness_parity`
- `drift_resistance`
- `overall`

Important rule:

- separate top-level scorecards come first
- a single app-level score is derived secondarily

The app-level score should never hide weak continuity or weak drift resistance.

## Score Resolution Rules

The benchmark system should define hard score-resolution rules so scores cannot
look healthy while product truth is weak.

Required rules:

- if a continuity-critical feature fails continuity evidence, its gate may not
  exceed `fragile`
- if a feature passes correctness but fails workflow continuity, its gate may
  not exceed `fragile`
- if a feature lacks required evidence, it may not exceed `partial` coverage
  and may not exceed `fragile`
- if `with memd` performs worse than `no memd` on the core journey the feature
  is supposed to improve, that feature may not exceed `acceptable`
- if a pillar contains continuity-critical children below `acceptable`, the
  pillar may not exceed `fragile`
- drift failures cap the affected subject's gate until the drift source is
  resolved
- contradictory evidence must lower confidence and should bias toward the
  weaker gate until the contradiction is explained

This matters because `memd` is highly vulnerable to false positives where:

- storage works but recall fails
- visible pages look good but hot-path retrieval is weak
- loop scores improve while continuity stays broken
- policy weights hide real product weakness

## Gate Ladder

The benchmark system should not rely only on percentages.

It should expose a gate ladder:

- `unsafe`
- `fragile`
- `acceptable`
- `strong`
- `ten-star`

Interpretation:

- `unsafe`
  - trust is compromised
- `fragile`
  - works, but continuity or quality is not dependable
- `acceptable`
  - usable without major trust collapse, but visible gaps remain
- `strong`
  - reliable and efficient under normal and adversarial use
- `ten-star`
  - seamless, drift-resistant, continuity-preserving, clearly better than no-memd

Important rules:

- no feature reaches `acceptable` without evidence from contract and workflow
  probes
- no continuity-critical surface reaches `acceptable` without continuity
  evidence
- no feature reaches `strong` if it still loses badly to `no memd`
- no pillar reaches `strong` if its continuity-critical children remain weak
- `ten-star` requires proven product advantage, not just passing checks

## Runtime Policies As Benchmark Subjects

The benchmark system should model policy/config surfaces that influence quality.

These should be first-class benchmark subjects, not hidden implementation
details.

Examples:

- dream trigger cadence
- loop weights
- route defaults
- working-memory budget
- promotion thresholds
- decay thresholds
- consolidation thresholds
- semantic fallback enablement
- workspace and visibility defaults

Recommended policy fields:

- `id`
- `name`
- `cli_surface`
- `default_value`
- `allowed_range`
- `quality_dimensions_affected`
- `risk_level`
- `loop_ids`

Important rule:

- quality-affecting runtime knobs should be benchmarkable the same way features
  are benchmarkable

## Token Efficiency Model

Token efficiency must be first-class.

It should be measured:

- per feature where the feature touches the hot path
- per journey for real continuity flows
- system-wide for overall app behavior
- comparatively: `no memd` vs `with memd`

Token efficiency should include:

- prompt size
- resume size
- handoff size
- reread count
- event noise
- semantic fallback cost
- reconstruction effort

The app should be expected to cut token cost over time without sacrificing
correctness or continuity.

## `No memd` Vs `With memd` Comparison Framework

This is a core product proof.

The benchmark system should compare:

- continuity success rate
- reconstruction effort
- token cost
- latency to usable state
- truth freshness
- drift rate
- handoff quality
- inspectability

Important rule:

- `with memd` should not be considered strong unless it beats `no memd` on the
  app promise:
  - less rereading
  - less reconstruction
  - stronger continuity
  - acceptable or better correctness
  - acceptable token overhead relative to benefit

Comparative loops should include:

- resume with vs without memd
- handoff with vs without memd
- search/reconstruction with vs without memd
- shared workspace continuity with vs without memd
- semantic fallback value vs token cost
- dream cadence value vs token overhead

Important rule:

- comparative loops should prefer the most continuity-relevant baseline, not
  the easiest baseline to beat
- the system should preserve a history of these comparisons so improvements or
  regressions over time remain visible

## Standard Loop Bundle Per Family

Each major family should eventually have:

- contract loops
- quality loops
- adversarial loops
- drift loops
- journey loops where relevant

Examples:

### Core Memory

- storage correctness
- recall reliability
- stale-belief recovery
- hot-path token cost
- runtime-vs-visible drift

### Retrieval

- route correctness
- crowd-out resistance
- compactness quality
- latency
- with-vs-without-memd benefit

### Bundle Runtime

- setup truth
- status truth
- resume continuity
- handoff actionability
- attach continuity
- continuity drift

### Coordination Hive

- presence freshness
- claim safety
- task safety
- collision handling
- continuity drift across sessions

### Obsidian

- import quality
- roundtrip integrity
- compiled-page usefulness
- workspace boundary safety

### RAG And Multimodal

- sync alignment
- bounded fallback
- semantic usefulness vs token cost
- attachment linkback integrity

### Autonomous Improvement

- gap-discovery quality
- accepted-vs-discarded boundary
- dream consolidation integrity
- portability classification correctness
- overnight uplift
- regression guardrails

## Autonomous Improvement As A Product Pillar

`autodream`, `autoresearch`, and `self-evolution` must be treated as first-class
product families.

They are not only maintenance helpers. For harnesses like OpenClaw and Hermes,
they are part of the product promise:

- the system should find quality gaps
- the system should improve over time
- accepted learning should become durable memory
- risky changes should stay gated
- harness-specific improvements should not be over-generalized

This should be split into separate families under one product pillar:

- `autodream`
- `autoresearch`
- `self-evolution`
- `cross-harness-adaptation`

These are product surfaces because they should improve the app itself:

- better continuity
- less drift
- lower token cost
- better harness adaptation
- better default memory behavior over time

## Gap Discovery During Autoresearch And Dream

The benchmark registry defines expected truth.

Overnight systems should still discover:

- missing benchmark coverage
- missing loops
- missing journeys
- missing feature links
- new drift categories
- bad scoring weights
- bad default policies
- places where `with memd` still fails to beat `no memd`

But discovery must not silently rewrite canonical truth.

It should emit proposal artifacts such as:

- suggested new features
- suggested new loops
- suggested new journeys
- suggested tier changes
- suggested weight changes
- suggested policy changes

Important rule:

- new features may be proposed automatically, but they must not be promoted into
  canonical truth without explicit gated acceptance

Authority boundaries should be explicit.

### What Overnight Systems May Do Automatically

- emit benchmark evidence
- emit loop results
- emit gap candidates
- emit proposed new loop records
- emit proposed new journey links
- emit proposed policy experiments
- emit ranked next-action recommendations

### What Overnight Systems May Not Auto-Promote

- new canonical feature records
- tier changes
- pillar/family remaps
- gate ladder changes
- score weight changes
- runtime policy changes that weaken continuity, correctness, or drift
  guardrails
- truth claims that reinterpret a failed `with memd` comparison as success

### What Requires Explicit Acceptance

- new feature promotion into the canonical registry
- benchmark schema changes
- tier changes
- score-weight changes
- policy-default changes
- guardrail changes
- any self-evolution change that widens authority

## Operating Model

The strongest operating model is a multi-speed system.

### Always-On

Cheap trust-preserving checks:

- tier-0 continuity-critical loops
- hot-path token regressions
- obvious runtime/surface drift
- registry/schema validity

Purpose:

- catch continuity damage immediately

Always-on loops must be cheap enough to remain enabled during normal work.

### Compaction-Triggered Dream

`dream` should also run in a faster lane tied to compaction count.

Current intended behavior:

- dream runs every 5 compactions

Important refinement:

- this is a runtime policy, not a hardcoded design constant
- it should be configurable through the CLI
- benchmark loops should verify the configured cadence is honored
- benchmark loops should compare quality and token outcomes across cadence
  settings

Purpose:

- preserve continuity-relevant patterns before they decay
- compact repeated accepted signal sooner

### Nightly

Nightly `autoresearch` should:

- benchmark broader families and journeys
- compare `no memd` vs `with memd`
- detect missing coverage
- detect drift
- rank gaps
- propose improvements

Then `autodream` should:

- consolidate accepted findings
- compress stable recurring patterns
- seed the next cycle with the highest-value gaps

Nightly runs should answer, in operator-facing form:

- what improved
- what regressed
- what still breaks continuity
- where drift is growing
- where `with memd` still loses to `no memd`
- what the maintainer should fix next

### Milestone Audits

Heavy exhaustive passes:

- full feature coverage sweep
- cross-harness parity
- adversarial failure modes
- pillar and family gate decisions
- docs/runtime/registry drift review

Purpose:

- decide whether the app is materially closer to the OSS goal

## Relationship Between Dream, Autoresearch, And Self-Evolution

The intended model:

- `autoresearch`
  - benchmarks
  - finds gaps
  - proposes improvements
- `autodream`
  - consolidates accepted findings
  - compacts frequent signal
- `self-evolution`
  - governs controlled change to:
    - registry links
    - weights
    - policies
    - low-risk runtime behavior

Important guardrails:

- only accepted findings flow into dream consolidation as durable truth
- discarded experiments remain searchable, but do not become durable memory
- self-evolution may propose changing weights or policies, but must not weaken
  continuity or correctness guardrails silently

## Benchmark Cost Budgets

Benchmarking itself should be cost-bounded.

The system should model:

- always-on latency budget
- always-on token budget
- nightly total token budget
- nightly wall-clock budget
- heavy audit budget

Important rules:

- tier-0 continuity checks should stay cheap enough for normal development
- a loop that costs too much relative to its trust value should be downgraded,
  sampled, or moved to a slower cadence
- benchmark cost regressions should be tracked the same way app token
  regressions are tracked
- the benchmark system should not quietly spend more tokens than the continuity
  benefit it provides

This is especially important because the app promise includes reducing token
waste over time.

## Scoring Weights

Default weights should exist, but they should be treated as runtime policy
rather than frozen doctrine.

Recommended default emphasis:

- continuity highest
- correctness next
- reliability next
- drift resistance next
- token efficiency strongly weighted
- provenance and inspectability still visible

Important rule:

- self-evolution may propose weight changes
- weight changes must be explicit, benchmarked, auditable, and reversible
- score changes must never be allowed to hide real continuity weakness

## Minimal Executable Slice

The full benchmark graph is large. The first shippable slice should be much
smaller and should still deliver trust value quickly.

The minimal executable slice should:

- ingest the existing `FEATURES.md` contracts
- create the canonical benchmark registry and schema
- model only tier-0 continuity-critical features first
- model the most important continuity journeys first
- generate basic coverage and score reports
- run a minimal always-on loop bundle
- emit evidence records for those loops
- compare `no memd` vs `with memd` for at least the top continuity journeys

The first continuity-critical slice should cover at least:

- wake
- resume
- compact context
- working memory
- handoff
- attach
- checkpoint
- hook capture
- continuity drift loops for those surfaces

This slice is the minimum system that lets the maintainer trust whether the app
is holding together at its core.

Only after this slice is healthy should the implementation expand into broader
families.

## Solo-Maintainer Morning View

The benchmark system should generate one operator-facing morning summary that
answers the most important questions without requiring raw JSON inspection.

It should show:

- top continuity failures
- top drift risks
- top token regressions
- top `with memd < no memd` losses
- newly discovered coverage gaps
- proposed next actions
- whether the app-level continuity gate moved up or down

This morning view should be treated as a required product output of the
benchmark system, not a nice-to-have report.

## Generated Views

The benchmark system should generate human-readable views from the registry and
telemetry.

Recommended generated outputs:

- feature and family summaries
- loop catalog
- coverage gaps
- journey status
- pillar scorecards
- gate ladder summaries
- `no memd` vs `with memd` deltas
- morning operator summary

This is important for solo-dev operation. The maintainer should not have to
query raw JSON to know what is broken.

## Drift Rules

To prevent the benchmark system itself from drifting:

- generated docs are not edited directly
- loops do not mutate registry truth directly
- feature IDs are stable
- scorecards are derived from evidence and loop results
- evidence paths are explicit
- runtime policy changes are tracked as benchmark subjects
- suggested new benchmark objects enter through proposal artifacts

## Completion Criteria

This design is only successful when the resulting benchmark system can answer,
without hand-waving:

- what are all the features?
- which features are continuity-critical?
- which features are weak or broken?
- what evidence proves each verdict?
- which journeys hold continuity?
- where is drift happening?
- where is token waste happening?
- where does `with memd` still fail to beat `no memd`?
- what should nightly improvement work on next?
- what can self-evolution change safely?

## Recommended Rollout

### Phase 1

Create the registry backbone:

- schema
- canonical registry
- pillar/family/tier taxonomy
- core feature ingestion from `FEATURES.md`
- basic generated reports
- morning operator summary skeleton

### Phase 2

Add continuity-critical journeys and loop mappings:

- resume
- handoff
- attach
- wake
- working memory
- drift prevention
- hard score-resolution rules

### Phase 3

Add `no memd` vs `with memd` comparative benchmarking:

- token and reconstruction metrics
- continuity score deltas
- generated comparison reports
- cost budgets for benchmark runs

### Phase 4

Add autonomous gap discovery:

- autoresearch proposals for missing loops and missing links
- dream consolidation of accepted benchmark findings
- self-evolution proposals for weight/policy changes
- explicit authority boundaries for what may and may not auto-promote

### Phase 5

Expand toward full-pillar coverage:

- coordination
- Obsidian
- semantic/multimodal
- cross-harness adaptation
- autonomous-improvement families

## Recommendation

Adopt this benchmark registry as the 10-star quality control plane for `memd`.

It is the strongest fit for the current product reality because:

- quality is the real problem, not just missing features
- continuity must stay the top product bar
- drift prevention must be explicit
- token efficiency must prove product value
- a fast-moving solo maintainer needs machine-readable trust, not only docs
- autodream, autoresearch, and self-evolution are core product features and
  should be benchmarked as such
