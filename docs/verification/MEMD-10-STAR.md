# memd 10-Star Target

> Authoritative 10-star contract. Updated 2026-04-13 from full codebase audit.
> For execution truth: [[ROADMAP.md]]. For audit findings: [[docs/audits/2026-04-13-full-codebase-audit.md]].

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

Weighted scoring from [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md|evaluation theory lock]]:

| Axis | Weight | Score | Status |
|------|--------|-------|--------|
| Session continuity | 20% | 2/10 | broken — ghost refs, expired inbox, status noise |
| Correction retention | 15% | 2/10 | mechanics exist, no UX flow, never proven in practice |
| Procedural reuse | 15% | 1/10 | code complete, detect never triggers, table empty |
| Cross-harness continuity | 15% | 3/10 | 6 presets, wake works, content is status noise, never cross-tested |
| Raw retrieval strength | 15% | 4/10 | search works, but wake/working excludes most kinds, no LongMemEval |
| Token efficiency | 10% | 6/10 | 78% boot reduction, budget enforced, no cost measurement |
| Trust + provenance | 10% | 5/10 | explain surfaces exist, source trust scoring, drilldown partial |

**Composite: ~2.9/10 (weighted)**

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

## Complete Gap Inventory (27 items)

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

### Tier 1: Make it work (3.0 → 6.0)
Fix operational pipeline (gaps 1-9). Run `memd eval --fail-below 65`.

### Tier 2: Make it correct (6.0 → 7.5)
Fix architectural gaps (gaps 10-17). Prove correction flow, behavior change, navigation.

### Tier 3: Make it provable (7.5 → 8.5)
Fix measurement gaps (gaps 18-23). Run public benchmarks, calibrate decay, prove quality.

### Tier 4: Make it 10-star (8.5 → 10.0)
Fix product gaps (gaps 24-35). Overnight evolution, human surface, team support, observability.

## Bottom Line

memd has the right architecture. 7 crates, 15 tables, 207 types, 90 client methods,
79 CLI commands, 6 harness presets, theory-locked model with 7 memory kinds and 3
control functions. The infrastructure is ahead of any competitor.

But the product doesn't work. The live loop is broken at 5 of 7 steps. The 10-star
score is ~2.9. No correction flow, no behavior proof, no navigation, no human surface.

The fix is not more features. The fix is making what exists actually work, then proving
it works, then shipping the product surfaces that make it usable.

That is the standard the rest of the repo should be measured against.
