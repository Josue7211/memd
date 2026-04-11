# 10-Star Verifier System Design

## Goal

`memd` needs a verification system that can prove the app's real product claim, not just report structural readiness. The verifier system should show that memory is seamless, continuity survives real workflows, and `with memd` is better than `no memd` on the workflows that matter.

This design adds a live verification control plane on top of the existing benchmark registry. The current benchmark remains the cheap health lane. The verifier system becomes the product-truth lane.

## North Star

`memd` should be able to answer, with evidence:

- every declared feature has a live proof path
- every continuity-critical feature survives real workflows
- adversarial conditions do not silently break continuity
- `with memd` is better than `no memd` on critical journeys
- missing proof coverage is itself treated as a failure signal

The verifier system must remain practical for a single maintainer while still scaling to a large feature list.

## Product Positioning

The existing `memd benchmark` command is valuable and should stay. It is fast, cheap, and useful for drift and readiness signals. It is not the final truth source for product quality.

The verifier system is a second layer:

- `benchmark`
  fast structural and artifact-backed quality signal
- `verify`
  live proof of feature contracts, journeys, adversarial behavior, and comparative product value

The app should not claim a feature is truly working just because the benchmark score is high. Product truth comes from live verifier evidence.

## Canonical Registry

The verifier system should live in the same canonical machine-readable registry as the benchmark system. Do not create a second source of truth for verification.

The canonical verification registry should expand the existing benchmark registry with these top-level sections:

- `verifiers`
- `fixtures`
- `evidence_policies`
- `schedules`

The full top-level shape becomes:

- `quality_dimensions`
- `tiers`
- `pillars`
- `families`
- `features`
- `journeys`
- `loops`
- `verifiers`
- `fixtures`
- `evidence_policies`
- `schedules`
- `scorecards`
- `gates`

This is the 10-star direction because features, journeys, benchmarks, verifiers, and coverage drift all remain attached to the same graph.

## Product Pillars

The verifier system should score and schedule work at the pillar level, not just the command level.

Primary pillars:

- `memory-continuity`
- `visible-memory`
- `truth-and-repair`
- `coordination`
- `knowledge-bridge`
- `autonomous-improvement`
- `cross-harness-runtime`
- `drift-prevention`
- `efficiency`

Every feature belongs to a family and pillar. Verifiers, journeys, and evidence all link back to those same product pillars.

## Tiering

The verifier system should keep the same tiered criticality model as the benchmark system.

- `tier-0-continuity-critical`
  wake, resume, handoff, attach, checkpoint, hook capture, working memory, compact context, continuity drift prevention
- `tier-1-truth-critical`
  search, explain, source, entity, timeline, repair, verify, policy truth
- `tier-2-workflow-critical`
  hive coordination, Obsidian, RAG, multimodal, UI workbench, autonomous improvement paths
- `tier-3-supporting`
  non-critical convenience and support features

Nightly failure semantics should fail hard on tier-0 and critical comparative regressions only.

## Verification Model

The verifier system should support five verifier types:

- `feature_contract`
- `journey`
- `adversarial`
- `comparative`
- `drift`

This is the minimum shape needed to prove product truth instead of only isolated function correctness.

## Hybrid Verifier Model

The verifier system should be declarative first, with bounded helper hooks for hard cases.

Most verifier logic should live in data records inside the canonical registry. Some verifiers, especially hive collision setup, vault seeding, or complex assertion calculations, need helper hooks.

This design should be explicitly hybrid:

- `declarative first`
  steps, assertions, metrics, gate targets, and evidence requirements are declared in data
- `helper-backed when needed`
  named helper routines may be used for bounded setup, snapshot extraction, or complex assertions

Helpers are allowed for:

- fixture seeding
- environment shaping
- snapshot extraction
- complex assertion calculations

Helpers are not allowed to become the hidden source of verifier truth. They should not own gate policy or silently replace the registry-driven verifier definition.

## Verifier DSL

Every verifier should be primarily data-driven.

Verifier record fields:

- `id`
- `name`
- `type`
- `pillar`
- `family`
- `subject_ids`
- `fixture_id`
- `baseline_modes`
- `steps`
- `assertions`
- `metrics`
- `evidence_requirements`
- `gate_target`
- `status`
- `lanes`
- `helper_hooks`

Supported step kinds:

- `cli`
- `api`
- `file_assert`
- `json_assert`
- `sleep`
- `compare`
- `capture_snapshot`
- `helper`

Verifier statuses:

- `unimplemented`
- `declared`
- `runnable`
- `passing`
- `failing`
- `flaky`
- `quarantined`

This lets the system distinguish between missing proof, broken proof, and unstable proof.

## Fixture Packs

Fixtures should be first-class records. Without deterministic fixtures, the verifier system will be too flaky to trust.

Fixture record fields:

- `id`
- `kind`
- `description`
- `seed_files`
- `seed_config`
- `seed_memories`
- `seed_events`
- `seed_sessions`
- `seed_claims`
- `seed_vault`
- `backend_mode`
- `isolation`
- `cleanup_policy`

Core fixture packs:

- `clean_bundle`
- `continuity_bundle`
- `noise_bundle`
- `stale_truth_bundle`
- `hive_two_session_bundle`
- `hive_collision_bundle`
- `obsidian_test_vault`
- `degraded_backend_bundle`
- `semantic_bundle`
- `self_evolution_bundle`

Recommended isolation modes:

- `fresh_temp_dir`
- `fresh_temp_git_repo`
- `fresh_temp_worktree`
- `shared_readonly_fixture`

The default should be `fresh_temp_dir`. Use fresh git repos or worktrees only when branch/worktree behavior matters.

## Evidence Model

Every verifier run must write evidence. Pass/fail alone is not enough.

Evidence artifact fields:

- `id`
- `verifier_id`
- `subject_id`
- `kind`
- `captured_at`
- `confidence_tier`
- `freshness`
- `path`
- `summary`
- `supports`
- `contradicts`

Typical evidence kinds:

- command output
- JSON assertion report
- latency sample
- token sample
- handoff packet snapshot
- resume snapshot
- compiled memory snapshot
- diff against baseline
- failure summary

Suggested artifact layout:

- `.memd/verification/latest.json`
- `.memd/verification/latest.md`
- `.memd/verification/runs/<timestamp>.json`
- `.memd/verification/runs/<timestamp>.md`
- `.memd/verification/evidence/...`

## Evidence Confidence

Not all evidence is equally trustworthy.

Confidence tiers:

- `live_primary`
  real live end-to-end proof from the current verifier run
- `live_secondary`
  real live runtime state, but not full workflow proof
- `artifact_fresh`
  recent artifact from a trusted earlier run
- `artifact_stale`
  older artifact still useful as context
- `derived`
  heuristic or estimated signal

This is critical. Without an explicit evidence confidence model, the system can cheat by inflating scores with structural checks and stale artifacts.

## Gates

The verifier system should use hard gate ladders, not just soft percentages.

Gate ladder:

- `broken`
- `fragile`
- `acceptable`
- `strong`
- `ten_star`

Gate rules:

- only `derived` evidence -> max gate `fragile`
- only artifact evidence -> max gate `acceptable`
- continuity-critical feature without `live_primary` evidence -> cannot exceed `acceptable`
- comparative claims without `live_primary` evidence -> invalid for top-level product truth
- continuity failure on a critical journey -> max gate `fragile`
- `with memd` less correct than `no memd` -> `broken`
- `with memd` not better than `no memd` on a critical journey -> cannot exceed `acceptable`

To reach `ten_star` for a continuity-critical feature:

- feature verifier passes
- journey verifier passes
- adversarial verifier passes
- comparative verifier proves `with memd` advantage
- evidence is fresh and live

## Scheduling

10-star does not mean running everything constantly. It means running the right proofs at the right cadence.

Verifier schedule fields:

- `id`
- `lane`
- `max_tokens`
- `max_duration_ms`
- `tiers`
- `default_types`
- `retry_policy`
- `quarantine_policy`

Execution lanes:

- `fast`
  cheap local continuity and drift checks
- `nightly`
  broader live verification and comparative checks
- `exhaustive`
  milestone or release audit
- `comparative`
  focused `no memd` vs `with memd` workflows

Per-verifier scheduling controls:

- `lanes`
- `priority`
- `max_tokens`
- `max_duration_ms`
- `retry_policy`
- `cooldown_minutes`
- `flaky_after_n`
- `quarantine_after_failures`

## CLI Surface

Nightly should be a normal CLI command so OpenClaw, Hermes, cron, Codex, and shell automation can all drive the same system.

Recommended command surface:

- `memd verify feature <feature-id>`
- `memd verify journey <journey-id>`
- `memd verify adversarial <verifier-id>`
- `memd verify compare <verifier-id>`
- `memd verify sweep --lane fast`
- `memd verify sweep --lane nightly`
- `memd verify sweep --lane exhaustive`
- `memd verify sweep --lane comparative`
- `memd verify doctor`
- `memd verify list`
- `memd verify show <id>`

`memd verify doctor` should surface:

- missing verifiers
- missing fixtures
- missing evidence
- stale evidence
- registry drift
- verifier drift
- quarantined or flaky verifier counts

## Nightly Failure Policy

Nightly should fail hard only when product trust is threatened.

Nightly exit non-zero on:

- `tier-0` verifier failures
- comparative regressions on continuity-critical journeys
- registry or schema drift that blocks trustworthy evaluation

Nightly should stay zero but report loudly on:

- tier-1 or lower failures
- flaky verifiers
- missing non-critical coverage
- non-blocking degraded quality

This keeps the overnight lane useful for a solo maintainer instead of becoming noisy and ignored.

## Operator Workflow

Fast lane:

- `memd verify sweep --lane fast`

Nightly lane:

- `memd verify sweep --lane nightly`

Exhaustive lane:

- `memd verify sweep --lane exhaustive`

Comparative lane:

- `memd verify sweep --lane comparative`

Each sweep should write:

- `.memd/verification/latest.json`
- `.memd/verification/latest.md`
- `.memd/verification/runs/<timestamp>.json`
- `.memd/verification/runs/<timestamp>.md`
- evidence artifacts under `.memd/verification/evidence/`

And update:

- `docs/verification/COVERAGE.md`
- `docs/verification/SCORES.md`
- `docs/verification/MORNING.md`

Morning summary should surface:

- top tier-0 failures
- top continuity regressions
- top comparative losses
- top flaky verifiers
- top missing verifier coverage
- top recommended next actions

## Relationship To Benchmark

The current benchmark remains a fast lane.

The verifier system should not replace it. It should consume the same canonical registry and produce stronger proof.

The benchmark should remain responsible for:

- fast structural readiness
- command and surface coverage
- artifact-backed quality hints
- cheap repeatable health checks

The verifier system should own:

- live feature proof
- live journey proof
- adversarial proof
- comparative product value proof

Top-level product truth should be derived primarily from verifier evidence, not from benchmark heuristics.

## Comparative Verification

`memd` needs real product-proof comparisons against a no-mempath baseline.

Comparative baseline modes:

- `no_mempath`
- `with_memd`
- `with_memd_semantic`
- `with_memd_workspace`
- `degraded_backend`

Critical comparative metrics:

- prompt tokens
- rereads
- reconstruction steps
- correctness
- time to usable state

The system should never claim `memd` is better than no mempath from estimates alone on critical journeys.

## Example Verifiers

Initial verifier set should focus on tier-0 continuity-critical proof.

Feature verifiers:

- `verifier.feature.bundle.wake`
- `verifier.feature.bundle.resume`
- `verifier.feature.bundle.handoff`
- `verifier.feature.bundle.attach`
- `verifier.feature.capture.checkpoint`
- `verifier.feature.capture.hook`
- `verifier.feature.working.context`
- `verifier.feature.working.memory`
- `verifier.feature.messages-send-ack`

Journey verifiers:

- `verifier.journey.resume-handoff-attach`
- `verifier.journey.checkpoint-resume`
- `verifier.journey.capture-to-hot-path`
- `verifier.journey.cross-session-continuity`

Adversarial verifiers:

- stale sibling session
- lane collision
- stale belief after correction
- noisy memory crowdout
- backend unavailable

Comparative verifiers:

- resume `no memd` vs `with memd`
- handoff `no memd` vs `with memd`
- task recovery `no memd` vs `with memd`

## Autodream, Autoresearch, and Self-Evolution

The verifier system should be a first-class target for autonomous improvement.

Autoresearch should be able to discover:

- missing verifiers
- missing fixtures
- flaky verifiers
- poor comparative results
- overreliance on artifact or derived evidence

Autodream should consolidate accepted verification findings into durable benchmark memory.

Self-evolution may propose:

- new verifiers
- new fixtures
- schedule changes
- weight changes
- tighter evidence policies

Self-evolution must not silently lower gate requirements or weaken continuity-critical standards.

## Minimal Executable Slice

The first implementation slice should be intentionally narrow.

Phase 1:

- extend the canonical registry with verifier sections
- add fixture definitions for a minimal tier-0 set
- add verifier DSL parsing and validation
- add `memd verify list`
- add `memd verify show <id>`
- add `memd verify feature <id>` for a minimal feature set
- add `memd verify journey <id>` for one continuity journey
- add evidence artifact writing
- add a verification summary under `.memd/verification/`

Phase 2:

- add comparative verifiers
- add adversarial verifiers
- add `memd verify sweep --lane fast`
- add `memd verify sweep --lane nightly`
- add verifier doctor and coverage drift reporting

Phase 3:

- add full lane scheduling
- add quarantine and flake handling
- integrate overnight gap discovery into autoresearch
- integrate accepted verifier learnings into autodream

## Why This Is 10-Star

This design is the 10-star direction because it:

- keeps one canonical truth graph
- scales to a large feature list
- proves real product behavior, not just command presence
- distinguishes live proof from stale artifacts and heuristics
- keeps the system practical for a single maintainer
- supports OpenClaw and Hermes as orchestration layers without duplicating verifier logic
- lets autodream, autoresearch, and self-evolution improve the system without lowering the trust bar

## Out of Scope For This Spec

This spec does not define:

- the final full verifier catalog for every feature
- the exact helper hook implementation details
- UI visualization for verifier evidence
- CI wiring or deployment-specific automation

Those belong in the implementation plan after the registry and CLI shape are approved.
