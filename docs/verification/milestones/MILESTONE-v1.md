# Milestone v1 Audit

- status: `pending`
- audit_date: `2026-04-06`
- claimed_features:
  - `FEATURE-V1-CORE-STORE`
  - `FEATURE-V1-CORE-SEARCH`
  - `FEATURE-V1-LIFECYCLE-REPAIR`
  - `FEATURE-V1-WORKING-CONTEXT`
  - `FEATURE-V1-WORKING-MEMORY`
  - `FEATURE-V1-EXPLAIN`
  - `FEATURE-V1-PROVENANCE`
  - `FEATURE-V1-BUNDLE-ATTACH`
- result: `pending`

## Findings

- hot-path recall crowd-out was real: synced `resume_state` noise could exclude durable project memory before score-based ranking mattered.
- the server retrieval bug now has regression coverage, and the client bundle path now has a regression proving recalled project facts can remain visible when retrieval returns them.
- low-level supersede/correction mechanics appear to work, but the product still lacks a zero-friction correction flow that normal users will actually trigger.
- bundle attach parity now has runtime launcher proofs across attach, Codex, Claude Code, and OpenClaw bundle startup surfaces.
- provenance drilldown now has cross-harness source-path proof across Codex and OpenClaw memory surfaces.
