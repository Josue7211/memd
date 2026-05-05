---
phase: G11
status: closed
closed: 2026-05-05
axes: [session_continuity, correction_retention, token_efficiency]
evidence: [docs/verification/v11-proof-runs/2026-05-05-compiler-sota-suite.ndjson]
---

# G11 Proof Harness

Closed by `scripts/verify/v11-compiler-sota-suite.sh`.

Exit criteria:

- 3-project scenario passes.
- Silent-correction detection latency <= 1000 ms.
- Dynamic compiler decisions and token counts are observable.
- Cost target is respected.
- Negative controls fire for isolation, dropped correction, muted detector, and
  ignored cost target.
- 10-STAR scorecard is regenerated to composite `6.95/10`.
