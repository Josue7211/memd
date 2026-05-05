---
phase: A11
status: closed
closed: 2026-05-05
axis: session_continuity
evidence: [docs/contracts/project-isolation.md, scripts/verify/v11-compiler-sota-suite.sh]
---

# A11 Project-Aware Wake

Closed by `memd_core::isolation`.

Exit criteria:

- `ProjectScope(project_id, workspace_id)` filters all wake records.
- Same workspace with different project is hidden.
- Same project with different workspace is hidden.
- Server schema lock exposes generated `memory_items.project_id` and
  `memory_items.workspace_id`.
- G11 3-project scenario proves project A -> project B -> project A wake
  restores A focus without B records.
