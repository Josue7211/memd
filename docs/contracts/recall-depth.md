---
contract: recall-depth
phase: E4
status: normative
opened: 2026-04-24
applies_to:
  - memd lookup
  - memd wake
  - memd resume
---

# Progressive-Depth Recall — Contract

`memd` exposes three recall depths. Each has an explicit cost ceiling and
quality contract. Agents pick depth; nothing escalates silently.

## Depths

| Depth   | Cost ceiling                        | Records returned        | Latency target | Use case                               |
| ------- | ----------------------------------- | ----------------------- | -------------- | -------------------------------------- |
| `wake`  | ≤2 000 tokens                       | compiled wake brief     | <100 ms p50    | session start / overview               |
| `lookup` | ≤500 tokens                        | 1–3 records             | <50 ms p50     | targeted query                         |
| `resume` | bounded by session-history length  | compiled task state     | <500 ms p95    | full reconstruction                    |

`tokens` are measured as compiled-output character count divided by 4
(matching the D4 ledger estimator at
`crates/memd-client/src/runtime/resume/compiler/ledger.rs`).

## CLI surface

```
memd lookup \
  --query "<text>" \
  [--depth wake|lookup|resume]    # default: lookup
  [--explain-depth]                # print which depth, why
  [--output .memd]
  [--json]
```

Default depth is `lookup`. Existing `memd wake` and `memd resume` commands
stay as-is; the depth flag on `memd lookup` is the unified entry point.

Exit codes follow current `memd lookup` semantics. Adding `--depth` does
not change non-zero exit conditions.

## Auto-escalation (hint only)

When `depth=lookup` returns zero records **and** the query matches the
specifier regex set, `memd` prints a one-line hint to stderr:

```
hint: zero results at lookup depth. Escalate with `memd lookup --query "…" --depth resume` (cost ~6k tokens).
```

The dispatcher does **not** silently re-run at `resume` depth. The agent
opts in.

### Specifier regex set

```
\b(the|my|our)\b .+ \b(task|plan|issue|decision|bug|feature)\b
what (was|were) (I|we) \b(doing|working on|trying)\b
where did (I|we) leave off
```

Case-insensitive. Matched against the query post-sanitization (server-side
`memd-server::query_sanitize`).

## Telemetry

Every recall call (any depth, including `memd wake`) appends one NDJSON
line to `<bundle_root>/logs/recall-depth.ndjson`:

```json
{
  "ts_ms": 1714000000000,
  "session_id": "session-…",
  "query": "the migration plan",
  "depth": "lookup",
  "records_returned": 2,
  "tokens_returned": 347,
  "latency_ms": 18,
  "escalation_hint": null
}
```

`escalation_hint` is `null` unless the line corresponds to a zero-hit
lookup that triggered the specifier hint, in which case it carries the
literal hint string.

The schema is frozen for V4. New fields must be additive.

## Feature flags

| Var                           | Default | Effect                                                   |
| ----------------------------- | ------- | -------------------------------------------------------- |
| `MEMD_E4_DEPTH_FLAG`          | `1`     | Enable `--depth`. Off → flag rejected with usage error.  |
| `MEMD_E4_ESCALATION_HINT`     | `1`     | Emit hint on zero-hit + specifier match.                 |

Both default-on at ship; flags exist for emergency rollback only.

## Pass gate

The contract is considered held when, in steady state on a real bundle:

1. `recall-depth.ndjson` records ≥30 % of recall calls at `lookup` depth.
2. p50 latencies meet the table above; p95 of `resume` ≤500 ms.
3. Per-turn token cost (sum of `tokens_returned` in a session) drops vs
   the pre-E4 baseline.

Evidence lives in
`docs/phases/v4/phase-e4-progressive-depth-recall.md` (pass gate) and
the 7-day distribution report attached at E4.7.

## Stability

The depth set (`wake|lookup|resume`) and the JSON schema are frozen for
V4. Sub-depth modes (e.g. `lookup --fusion fts:0.5,sem:0.5`) are
considered tuning knobs and may evolve without breaking the contract.
