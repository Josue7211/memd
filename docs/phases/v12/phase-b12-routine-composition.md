---
phase: B12
status: closed
closed: 2026-05-05
axis: procedural_reuse
evidence: [crates/memd-core/src/routine/library.rs, scripts/verify/v12-interop-sota-suite.sh]
---

# B12 Routine Composition

Closed by `memd_core::routine::library::RoutineLibrary::compose`.

Exit criteria:

- Routine A + B creates C with combined ordered steps.
- Source routine ids are preserved in `source_ids`.
- Duplicate steps collapse deterministically.
- G12 invokes composed `lint-format` after cross-workspace import.
