---
phase: B11
status: closed
closed: 2026-05-05
axis: session_continuity
evidence: [scripts/verify/v11-compiler-sota-suite.sh]
---

# B11 Compaction-Aware Recall

Closed by `memd_core::compaction::recovery`.

Exit criteria:

- Project-scoped compaction snapshots recover only matching records.
- Active corrections survive heavy post-switch compaction.
- Truncation is explicit when budget cannot fit all project records.
- G11 proves T4 Redis correction survives project B compaction and returns on
  project A round-trip.
