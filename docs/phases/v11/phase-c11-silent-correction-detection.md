---
phase: C11
status: closed
closed: 2026-05-05
axis: correction_retention
evidence: [scripts/verify/v11-compiler-sota-suite.sh]
---

# C11 Silent-Correction Detection

Closed by `memd_core::correction::silent`.

Exit criteria:

- Two user rephrases about a prior answer raise a `correction_flags`-shaped
  flag.
- Single confirmation does not false-positive.
- Detection is project-scoped.
- G11 proves flag latency <= 1000 ms.
