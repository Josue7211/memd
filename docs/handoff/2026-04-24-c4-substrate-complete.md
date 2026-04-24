---
opened: 2026-04-24
phase: C4
status: substrate-complete
prev_handoff: 2026-04-24-c4-entry.md
next_step: dogfood gate (C4.8) — needs 7 days of live captures
---

# C4 substrate complete — dogfood gate is the only thing left

## What landed this session

| Task | Commit | Status |
|------|--------|--------|
| C4.1 schema migration | `6d3aa2c` | landed |
| C4.2 detector module | `5b98d7b` | landed |
| C4.3 judge client + cache | `248cad1` | landed |
| C4.4 memd correction CLI verbs | `fe2fda3` | landed |
| C4.5 hook capture --kind correction | `6136e31` | landed |
| C4.6 E2E + fixtures | `96ff20d` | landed |
| C4.7 contract doc | `7a0aeec` | landed |
| C4.10 cross-harness Lamport resolver | `04af2b5` | landed |
| C4.8 dogfood + precision review | placeholder | **deferred** |
| C4.9 flag graduation + 10-STAR rescore | n/a | **blocked on C4.8** |

## Test totals

- memd-schema: 33/33 (3 C4 tests landed in C4.1)
- memd-core correction module: 14/14
- memd-client correction CLI + hook + E2E: 10/10
- workspace-wide: green

## Why C4.8 / C4.9 are deferred

C4.8 is a measurement gate that needs 7 calendar days of live dogfood
with `MEMD_C4_CORRECTION_DETECT=1`. It cannot be satisfied in a single
agent session — synthetic fixtures defeat the gate's purpose. See
`docs/phases/v4/c4-precision-review-PENDING.md` for the full procedure.

C4.9 is a one-line default flip + 10-STAR rescore that depends on a
passing C4.8.

## Pickup from here

1. Set `MEMD_C4_CORRECTION_DETECT=1` in your shell rc.
2. Use Claude Code normally for ≥7 days.
3. Run the precision-review procedure in
   `docs/phases/v4/c4-precision-review-PENDING.md`.
4. If gates clear → flip the flag default in `args.rs` /
   `cli_hook_runtime.rs` and rescore correction_retention 2 → 4 in
   `docs/verification/MEMD-10-STAR.md`.

## Notable design overrides

- **Schema** (C4.1): handoff overrode plan from 4 flat fields to nested
  `Option<CorrectionMetadata>`. Saved 30+ literal-site edits.
- **Schema version** (C4.1): no `user_version` PRAGMA bump — change is
  JSON-layer additive serde, zero DDL.
- **Detector clamp** (C4.2): score clamps to ≤0.5 when no prior-claim
  reference exists, so the judge still gates promotion.
- **Judge transport seam** (C4.3): trait-based; tests use a stub queue,
  prod uses reqwest blocking against codex-lb.
- **Hook ordering** (C4.5): NDJSON write happens BEFORE the standard
  checkpoint/promote flow so partial loss is preferable to total loss.
- **Cross-harness** (C4.10): `pick_correction_winner` requires explicit
  supersede + Lamport-greater. Equal versions tie-break on
  `source_agent` lexicographic for determinism.

## Dependent phases

- **D4** wake compiler reads corrections as a top-priority bucket — was
  parallelizable with C4.1 onwards.
- **F4** preference drift correction promotes to preference — needs
  Correction variant (✅ landed).
- **G4** proof harness cross-harness flip — needs C4.10 (✅ landed).
- **V5 B5** correction propagation bench — unblocked.
