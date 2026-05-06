---
milestone: v20
name: Info-Theoretic TE + Bench Ceiling + 1.0.0 Release
status: code_complete_external_replay_and_dogfood_pending
opened: 2026-04-22
depends_on: [v19, ../../verification/1.0.0-CONTRACT.md, ../../verification/1.0.0-AXIS-OWNERSHIP.md, ../../theory/MEMD-SOTA-THEORY.md]
composite_pre: 9.75
composite_target: 10.00
axes_lifted: [raw_retrieval, token_efficiency]
axes_integrated_with: [session_continuity, correction_retention, procedural_reuse, cross_harness, trust_provenance]
---

# Milestone v20 Audit — Info-Theoretic TE + Bench Ceiling + 1.0.0 Release

## Goal

**1.0.0 release gate. Ceiling close — zero margin on every axis.**

Info-theoretic optimal compiler: no token in the compiled context can
be removed without measurable downstream task quality degradation.
Dominates every public bench by ≥10pp margin. Publishes new harder
benches that existing SOTA competitors fail on. Generalizes to zero-
shot domains. Ships RR 9→10 and TE 9→10 (V20 closes the final two
axes at ceiling).

## 10-STAR axis targets (pre / post)

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 10 | 10 | INT (release harness touches; no credit) |
| correction_retention | 15% | 10 | 10 | INT |
| procedural_reuse     | 15% | 10 | 10 | INT |
| cross_harness        | 15% | 10 | 10 | INT |
| raw_retrieval        | 15% | 9  | 10 | **OWNS +1** — ≥10pp bench margin + publish harder benches + zero-shot |
| token_efficiency     | 10% | 9  | 10 | **OWNS +1** — info-theoretic optimal (no token removable without quality loss) |
| trust_provenance     | 10% | 10 | 10 | INT |

**Composite: 9.75 → 10.00** (ceiling; zero margin).

## Phases (code complete; external replay and real gates open)

- **A20** Info-theoretic TE prover (removal harness: remove token T → measure quality delta; optimal iff all deltas ≥ threshold)
- **B20** Bench-domination sweep (LoCoMo, LongMemEval, MemBench, ConvoMem — target ≥10pp margin on each)
- **C20** memd-published harder benches (at least one novel benchmark; SOTA competitors score below memd by ≥15pp)
- **D20** Zero-shot domain generalization (retrieval on unseen domain with ≤5pp quality delta vs tuned)
- **E20** 1.0.0 release harness (aggregates all axes; every axis=10 assertion; zero-generosity regenerator)
- **F20** Third-party replay for every axis (external reviewer reproduces every proof from export)
- **G20** 1.0.0 release tag + full proof bundle in `docs/verification/release-1-0-0/`

## Completion gate (1.0.0 release)

1. Info-theoretic TE proof: every token removal test fails quality threshold (optimal) — **passed synthetic harness**.
2. Bench-domination: ≥10pp margin on all four public benches simultaneously — **passed synthetic sweep; public rerun/external replay pending**.
3. memd-authored harder bench published with SOTA competitor scores ≥15pp below memd — **passed synthetic proof; publication/replay pending**.
4. Zero-shot domain test: retrieval quality delta ≤5pp vs tuned baseline — **passed synthetic proof**.
5. Every axis proven at 10/10 via its owning milestone's harness, aggregated in G20 — **provisional passed**.
6. Third-party replay: external reviewer reproduces every axis proof from export — **pending**.
7. 10-STAR composite regenerated =10.00 exactly; zero axis below 10 — **provisional passed; final close waits on real gates**.
8. 1.0.0 release tag lands on main — **blocked until real dogfood + third-party replay gates land**.

## Evidence

- Core substrate: `crates/memd-core/src/v20.rs`
- Release bundle: `docs/verification/release-1-0-0/`
- Proof script: `scripts/verify/v20-release-suite.sh`
- Summary: `docs/verification/release-1-0-0/2026-05-06-v20-release-suite.md`
- Artifact: `docs/verification/release-1-0-0/2026-05-06-v20-release-suite.ndjson`

## Non-goals

- AGI-level reasoning memory (substrate only)
- Post-quantum cryptography (current ed25519 + ZK primitives are enough for 1.0.0)
- Memory compression via novel tokenizer (scoped as 2.0.0+ work if needed)

## V20.5 recovery reserve

Every axis has zero margin at V20. Any regression blocks 1.0.0. V20.5
is reserved in advance: if V20 close misses any axis, V20.5 files a
recovery phase scoped to that axis before the 1.0.0 tag. Recovery phase
may not claim new axis credit — it restores the axis to its V20 target.

## Changelog

- 2026-04-22 opened. 1.0.0 release gate milestone. Ceiling close for
  every 10-STAR axis. V20.5 recovery phase reserved for zero-margin
  contingencies.
- 2026-05-06 code complete. TE removal prover, bench ceiling sweep, harder-bench/zero-shot fixtures, aggregate every-axis=10 release harness, release proof bundle, and V20 artifacts landed. `1.0.0` tag intentionally not cut until real dogfood and third-party replay gates land.
