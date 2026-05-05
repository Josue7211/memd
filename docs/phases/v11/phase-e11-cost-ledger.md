---
phase: E11
status: closed
closed: 2026-05-05
axis: token_efficiency
evidence: [scripts/verify/v11-compiler-sota-suite.sh]
---

# E11 Cost Ledger

Closed by `memd_core::cost::ledger` plus existing `memd configure` cost
surface.

Exit criteria:

- `cost_target_per_turn_cents=0.5` maps to a 1500-token budget.
- Ledger rows record per-turn project, token count, and derived cost.
- G11 proves the compiler respects the cost target.
- Server schema lock creates `cost_ledger`.
