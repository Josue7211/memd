# Feature memory_core 25-star local proof

[[ROADMAP]]: Verification artifact for the local memory-core feature slice. This is not an external benchmark or production reliability claim.

## Scope

Feature registry id: `feature.memory_core`

This proof maps the core memory substrate to current in-repository commands, tests, and artifacts for:

- capture
- lookup
- resume
- corrections
- provenance
- trust

The claim is deliberately narrow: memd has local Rust surfaces and tests for these memory-core behaviors, plus historical verification artifacts for broader memory-OS work. This document does **not** claim perfect recall, universal trust guarantees, production readiness, or independent external validation.

## Local proof command

Run:

```bash
bash scripts/verify/feature-memory-core-proof.sh
```

The proof script checks that this document, the feature registry entry, cited source surfaces, cited tests, and cited historical artifacts remain present. It also runs `feature-registry-audit.sh`. Cargo checks/tests are listed below as the behavior proof commands and were run for this feature slice when feasible.

## Behavior map

| Axis | Existing command or surface | Existing tests/artifacts | What this supports | Boundary |
| --- | --- | --- | --- | --- |
| Capture | `memd store`, `memd candidate`, `memd ingest` through `crates/memd-client/src/cli/cli_memory_runtime.rs`; server store paths in `crates/memd-server/src/store_memory_runtime.rs` and `crates/memd-server/src/store_memory_domains.rs` | `crates/memd-server/src/tests/memory_behaviors.rs` (`dogfood_store_fact_survives_context_retrieval` and related store/search tests) | Memory items can be accepted by local client/server surfaces and persisted in the local store test harness. | Does not prove durability across every deployment/storage backend. |
| Lookup | `memd lookup` and compiled-memory query paths in `crates/memd-client/src/cli/cli_memory_runtime.rs`; recall depth runtime in `crates/memd-client/src/runtime/recall/` | `crates/memd-client/src/main_tests/recall_depth_tests/mod.rs`; `crates/memd-server/src/tests/memory_behaviors.rs` | Lookup depth is parsed, bounded, dispatched, and covered by local search/context tests. | Does not claim semantic perfect recall or benchmark dominance. |
| Resume | `lookup --depth resume` synthesis via `synth_resume_args`; resume/wake depth handling in `crates/memd-client/src/runtime/recall/` | `crates/memd-client/src/main_tests/recall_depth_tests/mod.rs` (`lookup_depth_resume_returns_full_task_state`) | Resume depth preserves bundle identity and routes to the resume snapshot path. | Does not prove every real session can be fully reconstructed. |
| Corrections | `memd correction detect/capture/list` in `crates/memd-client/src/cli/cli_correction_runtime.rs`; detector/judge modules in `crates/memd-core/src/correction/` | `crates/memd-client/src/main_tests/correction_e2e_tests/mod.rs`; release artifacts under `docs/verification/release-0-1-0/*axis-correction_retention*`; `scripts/verify/v18-correction-graph-suite.sh` | Corrections can be detected/captured locally, logged with provenance fields, survive a compaction-style copy/restore test, and have historical correction-retention/graph artifacts. | Does not prove automatic correction of all stale beliefs. |
| Provenance | Source fields on memory requests/items; correction log fields `session_id`, `turn`, `captured_by`, `corrects_id`; provenance auditor/integrity benchmark modules | `crates/memd-client/src/benchmark/substrate/provenance_auditor.rs`; `crates/memd-client/src/benchmark/substrate/provenance_integrity.rs`; release artifacts under `docs/verification/release-0-1-0/*axis-trust_provenance*`; `scripts/verify/v19-zk-provenance-suite.sh` | Local structures and historical proof runs track provenance metadata and integrity-oriented checks. | Does not claim cryptographic or third-party provenance verification for this slice. |
| Trust | `MemoryVisibility`, `SourceQuality`, repair/verify paths in client/server surfaces; trust/provenance historical reviews | `crates/memd-server/src/tests/memory_behaviors.rs`; `docs/verification/release-0-1-0/*axis-trust_provenance*`; backlog item `docs/backlog/m3/2026-04-14-trust-hierarchy-unproven.md` | Trust boundaries are represented in schemas/tests and the registry explicitly marks external validation as absent. | Trust hierarchy remains locally tested/partial, not externally validated. |

## Suggested cargo verification

Use the guarded cargo wrapper so worktrees do not collide on cache/target directories:

```bash
bash scripts/memd-cargo-guard.sh check -p memd-client -p memd-core -p memd-schema
bash scripts/memd-cargo-guard.sh test -p memd-client correction_e2e_tests
bash scripts/memd-cargo-guard.sh test -p memd-client recall_depth_tests
bash scripts/memd-cargo-guard.sh test -p memd-client runtime_memory_tests

# Additional server-side memory behavior command to run when the memd-server test harness compiles:
bash scripts/memd-cargo-guard.sh test -p memd-server memory_behaviors
```

These commands are intentionally local and repository-scoped. The client/core/schema commands are the current green local proof set. The server-side memory behavior command is relevant but can be blocked by unrelated memd-server test-harness compile issues; treat it as an additional check, not as a passed artifact unless it is re-run successfully. Passing local commands does not convert the feature to externally verified status.

## Current claim level

- `current_status`: partial
- `proof_status`: strong local proof for the mapped substrate artifacts when the proof script and current green guarded cargo commands pass on this commit
- `dogfood_status`: ad_hoc
- `external_status`: none

Allowed claim: memory-core substrate surfaces have current local proof coverage across capture, lookup, resume, corrections, provenance, and trust boundaries.

Forbidden claim: do not claim reliable memory OS, perfect recall, production-grade trust, or external verification from this artifact alone.
