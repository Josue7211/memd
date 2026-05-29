# All Features 25/25 Master Plan

> **For Hermes:** Use `subagent-driven-development` for execution. One independent subagent per pillar/task. Plan mode first; no implementation until the plan is accepted.

**Generated:** 2026-05-29T05:11:01Z

**Goal:** Move memd from setup-slice/local proof toward whole-app/all-features 25/25 with honest gates.

**Current honest state:** setup/local proof exists, but whole-app 25/25 is not done. True 25/25 remains blocked by whole-product registry truth, feature scorecard derivation, hive proof, token/cache proof, competitor replay, UX polish, dogfood windows, external replay, and network identity proof.

**Hard rule:** Do not call memd true 25/25, Apple-level for anybody, externally validated, or category complete until external/dogfood gates pass.

---

## Execution Model

Use 12 workstreams. Each workstream gets:

1. one `/goal`-style planner subagent,
2. then one implementation plan file,
3. then implementation subagents only after plan approval,
4. then spec review subagent,
5. then quality review subagent,
6. then proof command,
7. then commit only after verification.

No pillar closes from docs alone. Every pillar needs runnable proof and artifact paths.

---

## Global Dependency Order

### Stage 1 — Truth foundation

1. Pillar 01 Registry Truth
2. Pillar 10 Feature Scorecard / Coverage Gate

These must land first because every other pillar must register feature, status, proof, blockers, and score caps.

### Stage 2 — Human product path

3. Pillar 02 Apple-Level Setup
4. Pillar 03 Apple-Level Docs
5. Pillar 11 Product UX Surfaces

These make memd understandable and usable by real people.

### Stage 3 — Technical proof pillars

6. Pillar 04 Token Savings / Context Optimization
7. Pillar 05 Shared Research Cache
8. Pillar 07 Hive/Hivemind
9. Pillar 06 Competitor/Public Benchmark Proof

These prove the actual product/system claims.

### Stage 4 — Real-world proof gates

10. Pillar 08 Real Dogfood Windows
11. Pillar 09 External Replay/Auditor Proof
12. Pillar 12 Network Identity/Federation/Market Layer

These are the true 25/25 blockers. Local work can prepare them, not fake-close them.

---

# Pillar 01 — Registry Truth

## Objective

Turn `docs/verification/FEATURES.md` from stale 22-entry registry into whole-app CEO truth.

## Problem

Current registry misses setup, docs, token savings, shared cache, hive, competitor replay, dogfood, external proof, UX, and network identity.

## Deliverables

- `docs/verification/FEATURES.md` rewritten/expanded as whole-app registry.
- `docs/verification/features.registry.json` or strict markdown YAML blocks.
- `docs/verification/features.schema.json`.
- `scripts/verify/feature-registry-audit.sh`.
- `docs/verification/feature-coverage-report.md`.

## Required Schema Fields

Each feature needs:

- `id`
- `name`
- `taxonomy_primary`
- `user_contract`
- `implementation_surfaces`
- `docs`
- `proof_commands`
- `proof_artifacts`
- `current_status`
- `proof_status`
- `dogfood_status`
- `external_status`
- `blocks_25_25`
- `scorecard_axes`
- `claim_allowed`
- `claim_forbidden`
- `freshness_policy`

## Feature Taxonomy

- setup/onboarding
- docs/product education
- core memory substrate
- recall/behavior
- corrections/provenance/trust
- token savings/context compiler
- shared research cache/RAG
- hive/hivemind
- cross-harness integrations
- public benchmarks/competitors
- product UX surfaces
- dogfood/reliability
- external replay/auditor
- network identity/federation/market
- operations/update/uninstall/recovery

## Acceptance Gate

```bash
scripts/verify/feature-registry-audit.sh
scripts/doc-lint.sh
git diff --check
```

Pass means every public claim and surface maps to a feature and proof state.

---

# Pillar 02 — Apple-Level Setup / Install / Onboarding

## Objective

Make setup understandable and reliable for a non-maintainer.

## Deliverables

- improved `README.md` quickstart
- improved `START-HERE.md`
- polished `docs/setup/*`
- stronger `scripts/install-memd.sh`
- stronger `scripts/update-memd.sh`
- stronger `scripts/uninstall-memd.sh`
- new `scripts/reset-memd.sh` or reset command
- `scripts/verify/setup-lifecycle-proof.sh`
- clean-machine proof artifacts
- brother-style human trial template/results

## Required UX

A new user can answer:

- What command do I run first?
- What did it install?
- Where is my memory?
- How do I know it worked?
- How do I fix failure?
- How do I update?
- How do I uninstall without deleting memory?

## Acceptance Gate

```bash
scripts/install-memd.sh --dry-run
scripts/verify/setup-experience-smoke.sh
scripts/verify/setup-lifecycle-proof.sh
scripts/update-memd.sh --dry-run
scripts/uninstall-memd.sh --dry-run
scripts/doc-lint.sh
```

True 25/25 requires external no-handholding users.

---

# Pillar 03 — Apple-Level Docs / Product Education

## Objective

Make docs thorough, understandable, self-verifying, and non-jargon for new users.

## Deliverables

Create `docs/product/`:

- `README.md`
- `what-is-memd.md`
- `why-memd.md`
- `mental-model.md`
- `new-user-path.md`
- `features.md`
- `token-savings.md`
- `shared-research-cache.md`
- `hive.md`
- `proof.md`
- `competitors.md`
- `privacy-and-trust.md`
- `glossary.md`
- `faq.md`
- `claim-to-proof-map.md`

Add scripts:

- `scripts/verify/product-docs-proof.sh`
- `scripts/verify/docs-claim-proof-audit.sh`
- `scripts/verify/docs-no-jargon-audit.sh`
- `scripts/verify/docs-new-user-path-audit.sh`

## Acceptance Gate

```bash
scripts/verify/product-docs-proof.sh
scripts/doc-lint.sh
scripts/lint-links.sh
```

Pass means every public claim has proof or pending label.

---

# Pillar 04 — Token Savings / Context Optimization

## Objective

Prove memd saves tokens without losing important truth.

## Deliverables

- `docs/contracts/token-savings-context-optimization.md`
- `docs/verification/pillar-04-token-savings-scorecard.md`
- `scripts/verify/pillar-04-token-savings-context-optimization.sh`
- fixtures for raw context, compiled context, cache hit/miss, wasted-token cases, ablations
- user-visible `memd tokens status` / equivalent

## Required Proof Areas

- saved-token ledger
- wasted-token detector
- cache hit/miss attribution
- prompt budget enforcement
- compiler quality gates
- quality-preserving ablations
- visible dollars/tokens saved

## Acceptance Gate

```bash
scripts/verify/v11-compiler-sota-suite.sh
scripts/verify/pillar-04-token-savings-context-optimization.sh
```

Local 25/25 requires measurable savings plus 100% critical quality retention on fixture corpus.

---

# Pillar 05 — Shared Research Cache

## Objective

Prove read-once/reuse-everywhere research memory.

## Deliverables

- `docs/contracts/shared-research-cache.md`
- `memd research ingest|lookup|freshness|verify`
- source hash metadata
- freshness/invalidation states
- donor repo extraction fixture
- new-agent reuse proof
- `scripts/verify/shared-research-cache-proof.sh`

## Required States

- fresh
- unchanged
- changed
- source_missing
- extractor_outdated
- superseded

## Acceptance Gate

```bash
cargo test -p memd-client research_cache -- --nocapture
scripts/verify/shared-research-cache-proof.sh
```

Pass means Agent A reads source once; Agent B reuses cached research with raw source read count zero.

---

# Pillar 06 — Competitor / Public Benchmark Proof

## Objective

Make competitor/public benchmark claims honest, replayable, and source-backed.

## Deliverables

- filled `docs/verification/SUBSTRATE_COMPETITOR.md`
- cleaned `docs/verification/PUBLIC_LEADERBOARD.md`
- competitor source cards under `docs/verification/competitors/`
- regenerated leaderboard script that actually checks artifacts
- no-placeholder/no-unbacked-claim gate

## Required Competitor Handling

For Mem0, Supermemory, Letta/MemGPT, MemMachine, MemPalace:

- primary source citation
- comparable metric or `not comparable`
- local replay artifact or `blocked`
- no proxy metric claimed as canonical

## Acceptance Gate

```bash
scripts/regen-leaderboard.sh --check
scripts/verify/25-5-public-benchmark-fixtures.sh
scripts/verify/25-5-supermemory-head-to-head.sh
scripts/verify/25-5-competitor-head-to-head.sh
```

Blocked replay is honest evidence, not a win.

---

# Pillar 07 — Hive / Hivemind

## Objective

Make hive first-class product proof for multi-agent coordination.

## Deliverables

- first-class hive entries in registry
- contracts for claim/lease/ack, divergence, stale bee, handoff, privacy
- expanded `scripts/verify/hive-production-proof.sh`
- multi-agent dogfood packet
- external replay packet

## Required Scenarios

- queen + two workers
- message/inbox/ack
- claim acquire/transfer/release
- lease race rejection
- stale bee pruning
- divergence detection
- cross-agent handoff
- privacy boundary negative control

## Acceptance Gate

```bash
scripts/verify/hive-production-proof.sh
scripts/verify/hive-live-map-guard-contract.sh
```

True 25/25 requires real multi-agent dogfood + external replay.

---

# Pillar 08 — Real Dogfood Windows

## Objective

Close real elapsed proof windows for V14-V18. No synthetic closure.

## Deliverables

`docs/verification/release-1-0-0/dogfood/` with:

- cohort.json
- devices.json
- harness-user-pairs.json
- weekly notes
- V14/V15/V16/V17/V18 rollups
- redacted telemetry artifacts

## Required Windows

- V14: 30-day telemetry dogfood
- V15: 60-day self-tuning dogfood, ideally through 90 days
- V16: 90-day 3-device sync dogfood
- V17: 30-day marketplace/routine dogfood, 5 installs across 3 users
- V18: 3-month correction-chain dogfood, 50 multi-hop chains

## Acceptance Gate

```bash
scripts/verify/v14-telemetry-suite.sh
scripts/verify/v15-self-tuning-suite.sh
scripts/verify/v16-sync-suite.sh
scripts/verify/v17-routine-marketplace-suite.sh
scripts/verify/v18-correction-graph-suite.sh
```

No elapsed time, no true close.

---

# Pillar 09 — External Replay / Auditor Proof

## Objective

Make third-party verifier able to clone, install, run proofs, check claims, and report pass/fail without secrets.

## Deliverables

- `docs/verification/external-replay/README.md`
- verifier packet:
  - `VERIFIER-INSTRUCTIONS.md`
  - `CLAIMS.md`
  - `COMMANDS.sh`
  - `EVIDENCE-TEMPLATE.md`
  - `REDUCTION-RULES.md`
  - `CHECKSUMS.md`
- `scripts/verify/external-replay-export.sh`
- `scripts/verify/no-secrets-export-scan.sh`

## Acceptance Gate

```bash
scripts/verify/external-replay-export.sh
scripts/verify/no-secrets-export-scan.sh /tmp/memd-external-replay-*.tar.gz
```

True 25/25 requires actual external verifier evidence.

---

# Pillar 10 — Feature Scorecard / Coverage Gate

## Objective

Replace stale manual scorecards with registry-derived scoring.

## Deliverables

- `docs/verification/feature-coverage-registry.json`
- `docs/verification/feature-coverage-registry.schema.json`
- `scripts/verify/feature-coverage-gate.sh`
- generated local scorecard
- generated external scorecard
- coverage audit JSON/MD
- claim-language gate

## Score Caps

- no registry row: 0
- implementation, no proof: cap 10
- stale proof: cap 12
- local proof only: local can pass, external capped
- missing external evidence: true 25 blocked
- forbidden language: gate fails regardless of numeric score

## Acceptance Gate

```bash
scripts/verify/feature-coverage-gate.sh --check
scripts/verify/local-25-star-product-proof.sh
```

---

# Pillar 11 — Product UX Surfaces

## Objective

Make CLI/dashboard surfaces obvious, safe, and non-internal.

## Deliverables

- `docs/ux/product-language-contract.md`
- product copy layer
- plain `memd doctor`
- plain `memd status`
- plain `memd features`
- correction UX
- source-trail/provenance UX
- token/context budget UX
- hive/team UX
- dashboard product home
- `scripts/verify/pillar-11-product-ux-proof.sh`
- screenshots

## UX Rule

Default output is human. Internal terms only in `--json`, `--verbose`, or maintainer docs.

Forbidden default jargon examples:

- rag
- atlas_ratio
- bridgeable
- harness_native
- market_claim
- bundle=
- status=partial

## Acceptance Gate

```bash
scripts/verify/pillar-11-product-ux-proof.sh
```

Pass means doctor/status/features/correction/provenance/token/hive/dashboard are covered by snapshots/screenshots.

---

# Pillar 12 — Network Identity / Federation / Market Layer

## Objective

Build the V26+ network layer: one user/org identity across independent apps, federation, work market, external product backend, network trust.

## Critical Correction

“3 apps” means same user/org identity across independent app/client surfaces, not three users.

Recommended proof apps:

- Hermes
- Codex
- OpenClaw / Claw

## V26 Network Identity Deliverables

- `NetworkPrincipal`
- `AppGrant`
- `ScopedIdentityResolver`
- `RevocationReceipt`
- app link/revoke/inspect commands
- 3-app proof matrix
- zero-leakage adversarial tests
- revocation proof

## V27 Federation Deliverables

- federated memory packet
- org-to-org grant
- import/export/revoke/audit commands
- cross-org non-leakage proof

## V28 Agent Work Market Deliverables

- portable `WorkUnit`
- routine package format
- replay proof packets
- marketplace trust metadata

## V29 Default Backend Deliverables

- SDK/plugin path
- 5 external products using memd as primary memory backend
- visibility/revocation/replay tests

## V30 Network Trust Deliverables

- public compatibility registry
- signed/replayable badges
- independent replay
- revoked badge state

## Acceptance Gate

V26 local gate example:

```bash
scripts/verify/v26-network-identity-proof.sh
```

True network 25/25 requires external apps/products/builders, not synthetic fixtures only.

---

## Immediate Next Plan Task

The first implementation-plan file should be:

`docs/plans/2026-05-29-pillar-01-registry-and-scorecard-execution-plan.md`

It should combine Pillar 01 + Pillar 10 because registry and scorecard are one dependency pair.

Then execute with subagent-driven development:

1. Task: add schema
2. Task: seed registry
3. Task: build audit script
4. Task: generate scorecards
5. Task: add claim-language gate
6. Task: wire local proof
7. Task: run verification
8. Task: commit only after green

---

## Honest Close Language

Allowed now:

> Whole-app 25/25 planning packet created. Local setup proof exists. True 25/25 remains blocked on registry truth, scorecard coverage, UX/docs/setup polish, token/cache/hive/competitor proof, dogfood windows, external replay, and network identity evidence.

Forbidden now:

> memd is true 25/25.
> memd is externally validated.
> memd is Apple-level for anybody.
> all features are done.
