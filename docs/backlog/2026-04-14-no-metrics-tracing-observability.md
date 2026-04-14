# No Metrics / Tracing / Observability

- status: `open`
- severity: `high`
- phase: `V2-K2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

No structured logging, metrics, or tracing. Debugging requires reading source code. No visibility into memory API latency, throughput, or correctness. No alerts on anomalies.

## Fix

- Add structured logging (slog or tracing crate)
- Implement OpenTelemetry metrics and spans
- Add Prometheus scrape endpoint
- Document observability practices
