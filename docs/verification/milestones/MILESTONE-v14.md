---
milestone: v14
name: Telemetry + Observability Foundation
status: planned
opened: 2026-04-22
depends_on: [v13, ../../verification/1.0.0-CONTRACT.md, ../../verification/1.0.0-AXIS-OWNERSHIP.md, ../../theory/MEMD-SOTA-THEORY.md]
composite_pre: 8.50
composite_target: 8.60
axes_lifted: [token_efficiency]
axes_integrated_with: [session_continuity, correction_retention, procedural_reuse, cross_harness, raw_retrieval, trust_provenance]
---

# Milestone v14 Audit — Telemetry + Observability Foundation

## Goal

Real-user telemetry substrate. After 0.1.0 ships, memd collects
anonymized per-user per-harness usage data (opt-in, local-first,
exportable) that powers V15 self-tuning compiler and V20 info-theoretic
TE optimality proof. Cost ledger (V8 E8) gets a telemetry backend.
This milestone is foundational — TE +1 lift is the axis credit, but
every post-0.1.0 milestone depends on V14's substrate.

## 10-STAR axis targets (pre / post)

Baseline from V13 post (0.1.0 release): all axes at 0.1.0 ship values.

| axis | weight | pre | post | basis |
| --- | --- | --- | --- | --- |
| session_continuity   | 20% | 9 | 9 | INT (telemetry reads SC data; no credit) |
| correction_retention | 15% | 8 | 8 | INT |
| procedural_reuse     | 15% | 9 | 9 | INT |
| cross_harness        | 15% | 8 | 8 | INT |
| raw_retrieval        | 15% | 9 | 9 | INT |
| token_efficiency     | 10% | 7 | 8 | **OWNS +1** — telemetry-backed cost ledger; per-user per-harness usage visibility |
| trust_provenance     | 10% | 9 | 9 | INT (telemetry pipeline itself has provenance; no credit) |

**Composite: 8.50 → 8.60** (weighted arithmetic: +0.10 from TE +1).

## Phases (planned; spec at phase-land time)

- **A14** Telemetry schema + opt-in CLI (`memd telemetry enable/disable/status`; local-first; exportable)
- **B14** Per-user per-harness cost-ledger backend (V8 E8 extended; time-series storage under `.memd/telemetry/`)
- **C14** Anonymization primitives (PII scrubbers; ULID-based user hashing; differential-privacy noise on bench-shareable exports)
- **D14** Telemetry dashboard CLI (`memd telemetry report --window <duration>`)
- **E14** `memd configure` integration (V8 G8 exposes `telemetry.enabled`, `telemetry.retention_days`, `telemetry.export_scope`)
- **F14** Telemetry export format (versioned NDJSON; compatible with V15 self-tuning ingest, V20 info-theoretic bench)
- **G14** V14 gate harness (≥30-day real-user dogfood window; TE +1 regenerator; telemetry NDJSON reproducible)

## Completion gate

1. ≥30-day telemetry dogfood window with ≥3 real memd users (dogfooders).
2. `memd telemetry report` emits per-user per-harness token cost breakdown.
3. Cost ledger UI shows real telemetry-backed numbers, not synthetic fixtures.
4. Anonymization verified: exports contain no PII, ULIDs are hash-consistent per user.
5. `memd configure telemetry.enabled=false` disables pipeline cleanly (no orphaned state).
6. 10-STAR composite regenerated ≥8.60 with TE=8.
7. V15 self-tuning compiler substrate ready (telemetry NDJSON ingest contract locked).

## Non-goals

- Cloud telemetry aggregation (V14 is local-first; cloud aggregation is V17+ federation work)
- PII-bearing exports (scope explicitly excluded — differential privacy enforced)
- Every-axis telemetry lifts (V14 owns TE only; other axes read telemetry as integrators with no credit)

## Changelog

- 2026-04-22 opened. First post-0.1.0 milestone. Skeleton; full spec
  drafted at phase-land time when real-user deployment data is available.
