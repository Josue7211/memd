---
phase: C12
status: closed
closed: 2026-05-05
axis: procedural_reuse
evidence: [crates/memd-core/src/routine/library.rs, scripts/verify/v12-interop-sota-suite.sh]
---

# C12 Project Routine Inheritance

Closed by `memd_core::routine::library::RoutineLibrary::inherit`.

Exit criteria:

- Global routines load first.
- Project routines override by normalized name.
- Output library keeps project workspace identity.
- G12 fixture proves project override semantics are deterministic.
