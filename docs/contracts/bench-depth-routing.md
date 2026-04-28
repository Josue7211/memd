# Bench Depth-Routing Contract — V6 / E6

**Version:** `depth-router/v1`
**Status:** scaffold-symmetric (runtime activation calendar-gated post-2026-05-02 alongside A6.9 / B6 / C6 / D6)
**Pipeline position:** retrieval → typed-ingest (A6/B6/C6) → bench-compiler (D6) → **depth-router (E6)** → judge.

## 1. Purpose

Multi-call depth routing on the bench harness. The model can re-query
memd mid-answer using the V4 E4 depth-flag tiers (`wake → targeted →
resume`). The router parses inline tool-calls out of the model's
generation, resolves each via `memd lookup` (real runtime) or a
fixture stub (tests), splices the result into the conversation, and
loops until the model emits no further calls or a hard cap fires.

E6 is the first multi-call bench path in memd; F6 layers an
iterative-reasoning harness on top of it.

## 2. Tool-call surface

Inline format the model emits:

```
<<memd_lookup query="…" depth="wake|targeted|resume">>
```

- `query` — required, free-form. Backslash-escapes `\"` and `\\`.
- `depth` — optional, defaults to `targeted`. Maps to V4 E4's depth flag.

The router substitutes a fenced result block:

```
[memd_lookup depth=targeted query="…"]
…body…
[/memd_lookup]
```

The fenced block is intentionally distinct from the call surface so
the router does not re-parse its own injections (recursive tool-call
loops are out of scope for E6).

## 3. Escalation policy

Two observable signals decide whether to escalate beyond the model's
explicit calls. Pure helpers in `depth_policy.rs`:

1. **Empty wake result** — prior call was `depth="wake"` and returned
   zero hits. Escalate to `targeted`.
2. **Low-confidence answer** — model self-reported confidence falls
   below `MEMD_V6_DEPTH_CONFIDENCE_FLOOR` (default 0.6). Escalate to
   `resume`.

Empty-wake takes precedence over low-confidence so a wake-tier miss
deterministically retries before re-grounding against the long-form
record set.

## 4. Hard caps

| Cap | Default | Override |
| --- | --- | --- |
| Calls per answer | 3 | `--max-depth-calls`, `MEMD_V6_MAX_DEPTH_CALLS` |
| Retrieved tokens per answer | 10 000 | `--max-retrieval-tokens` |

Caps are enforced in the router itself; the loop returns a
`TerminationReason` of `MaxCalls` or `MaxRetrievalTokens` when fired
so per-question telemetry can record which cap won.

## 5. CLI surface

```
memd bench public --bench locomo \
  --typed-ingest=episodic+semantic+canonical \
  --compiler=on \
  --depth-routing=on \
  [--max-depth-calls 3] [--max-retrieval-tokens 10000]
```

`--depth-routing=off` preserves the legacy single-call answer path
verbatim (no parser, no resolver). `MEMD_V6_DEPTH_ROUTING=0` forces
off regardless of the CLI flag.

## 6. Telemetry

Per-question NDJSON appended to
`.memd/benchmarks/public/results/depth-telemetry-<date>.ndjson`:

```json
{
  "ts": "<iso8601>",
  "bench_id": "<bench-id>",
  "question_id": "<id>",
  "calls_issued": <usize>,
  "retrieval_tokens": <usize>,
  "termination": "no_more_calls | max_calls | max_retrieval_tokens",
  "depths": ["wake", "targeted", ...]
}
```

The pure router returns the metrics struct; IO is deferred to the
runtime dispatch layer (graduates with A6.9/B6/C6/D6 post-2026-05-02).

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_DEPTH_ROUTING` | unset → on | `0` forces off; any other value (or unset) honours the CLI flag. |
| `MEMD_V6_MAX_DEPTH_CALLS` | unset | Numeric override for the call cap. |
| `MEMD_V6_DEPTH_CONFIDENCE_FLOOR` | unset → `0.6` | Low-confidence escalation threshold. |

Independent of `MEMD_D4_COMPILER` / `MEMD_V6_COMPILER` /
`MEMD_V6_TYPED_INGEST`. Toggling depth-routing off does not
deactivate the D6 compiler.

## 8. Bench targets (E6.5 — fixture proxies until graduation)

| Bench | Target | Proxy fixture |
| --- | --- | --- |
| LoCoMo multi-hop | accuracy lift ≥ +0.04 | `tests/fixtures/typed_ingest/e6/multihop-10.jsonl` |
| LME temporal | accuracy lift ≥ +0.03 | `tests/fixtures/typed_ingest/e6/temporal-10.jsonl` |
| Cumulative | LME ≥ +0.07, LoCoMo ≥ +0.07, MemBench ≥ +0.06, ConvoMem ≥ +0.03 | aggregated post-graduation |
| Regression guard | routed prompt-tokens + accuracy ≥ baseline | covered by test 10 |

Real-corpus locks graduate with A6.9 / B6 / C6 / D6 runtime activation
post-2026-05-02 (E6 graduates in the same wave; the F6 reasoning
harness depends on it).

## 9. Versioning

The schema version is pinned at `depth-router/v1` and surfaced in the
runtime notice + telemetry NDJSON. Bumping the major invalidates
older traces; minor bumps are forward-compatible.

## 10. Out of scope

- Mutating V4 E4. The router is a wrapper around the existing
  `memd lookup --depth=…` CLI.
- Recursive tool-call loops (the router does not re-parse its own
  injected blocks). Multi-step reasoning lives in F6.
- Mutating the bench-compiler. D6 is upstream of the router and
  un-altered.
