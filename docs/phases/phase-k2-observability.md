---
phase: K2
name: Observability
version: v2
status: pending
depends_on: [D2, C2, J2]
backlog_items: [59, 72, 74]
---

# Phase K2: Observability

## Goal

Operators can debug, measure, and monitor without reading source code.

## Deliver

- `memd explain <id>` CLI with full provenance chain
- Tag search and filtering endpoints
- Event spine integrity checks (checksums, validation)
- Structured logging with tracing
- Latency measurement and SLA
- Data recovery procedure (backup/restore)
- Backward compatibility contract for schema migrations

## Pass Gate

- `memd explain <id>` shows: source, confidence, corrections, evidence chain
- Tags searchable via API
- Spine corruption detected and reported on startup
- Structured logs parseable by standard tools
- P95 retrieval latency < 100ms for working memory
- Backup → corrupt DB → restore → data intact

## Evidence

- Explain output for 5 different item types
- Tag search test
- Corruption injection + detection test
- Latency benchmark
- Backup/restore test

## Fail Conditions

- Explain dead-ends
- Spine corruption undetected
- Restore fails or loses data

## Donor Extraction (from inspiration repos)

- **K2-D1** (Omegon `status.rs`): Compiled operator state surface. One `HarnessStatus` struct captures everything: git branch, memory health, inference backends, context class, capability tier. Maps to `memd state` as canonical operator brief.
- **K2-D2** (Omegon `upstream_errors.rs` — **DIRECT RUST LIFT**): Structured error classification. `UpstreamErrorClass` → `RecoveryAction` mapping. Every failure logged with provider, model, attempt count, delay. Replace memd's flat `(StatusCode, String)` errors.
- **K2-D3** (Omegon `bridge.rs`): Token tracking per request. Every call returns `input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens`. Measure wake packet token efficiency.

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert integrity checks if they slow startup > 2s
