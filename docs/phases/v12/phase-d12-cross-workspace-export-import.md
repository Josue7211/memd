---
phase: D12
status: closed
closed: 2026-05-05
axis: procedural_reuse
evidence: [crates/memd-core/src/routine/library.rs, scripts/verify/v12-interop-sota-suite.sh]
---

# D12 Cross-Workspace Export Import

Closed by `RoutineLibrary::export_workspace` and `RoutineLibrary::import_workspace`.

Exit criteria:

- Routine export is versioned as `memd.routine-library.v1`.
- Export carries workspace id, routines, and checksum.
- Import rejects tampered checksum.
- G12 imports `lint-format` from WS-1 into WS-2.
