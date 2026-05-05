---
phase: D11
status: closed
closed: 2026-05-05
axis: token_efficiency
evidence: [scripts/verify/v11-compiler-sota-suite.sh]
---

# D11 Dynamic Compiler

Closed by `memd_core::runtime::resume::compiler_v2`.

Exit criteria:

- Turn intent maps to per-turn memory depth.
- Recall intent uses immediate + procedural context.
- Lookup intent uses immediate context only.
- Compiler rows expose `compiler_context_id`, `intent_class`,
  `target_token_budget`, `actual_tokens`, and `depth_decision`.
- G11 logs stable per-turn decisions within budget.
