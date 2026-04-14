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

## Rollback

- Revert integrity checks if they slow startup > 2s
