# Iterative Reasoning Contract — V6 / F6

**Version:** `iterative-reasoning/v1`
**Status:** scaffold-symmetric (runtime activation calendar-gated post-2026-05-02 alongside A6.9 / B6 / C6 / D6 / E6)
**Pipeline position:** retrieval → typed-ingest (A6/B6/C6) → bench-compiler (D6) → depth-router (E6) → **reasoning harness (F6)** → judge.

## 1. Purpose

Multi-step reasoning over typed memory. Where E6 lets the model
re-query memd once mid-answer, F6 lets it chain N depth-routed
lookups into a single scratchpad before committing an answer.
Designed for question types that can't be answered with a single
retrieval pass: temporal sequencing (LME temporal subset), multi-hop
chains (LoCoMo sequential-reasoning subset).

## 2. Step schema

The scratchpad is a flat list of step records:

```json
{
  "steps": [
    {"n": 1, "action": "lookup", "query": "…", "depth": "targeted", "result_ids": ["…"]},
    {"n": 2, "action": "lookup", "query": "…", "depth": "resume", "result_ids": ["…"]},
    {"n": 3, "action": "answer", "text": "…"}
  ],
  "terminated_by": "answer | step_cap | token_cap"
}
```

- `n` is 1-indexed and dense.
- `lookup` carries `query`, `depth` (V4 E4 tiers), and the resolved
  `result_ids` from the depth router.
- `answer` carries `text` only and terminates the loop.
- `terminated_by` records whichever stop condition fired.

## 3. Termination rules

The harness stops as soon as one of:

1. The driver emits an `Answer` step.
2. `max_steps` reached (default 5).
3. Cumulative `retrieval_tokens` ≥ `max_retrieval_tokens` (default
   20 000 — slightly above E6's per-answer budget so the multi-step
   chain has slack).

## 4. Hard caps

| Cap | Default | Override |
| --- | --- | --- |
| Steps per question | 5 | `--max-reasoning-steps`, `MEMD_V6_MAX_REASONING_STEPS` |
| Retrieved tokens per question | 20 000 | `--max-reasoning-tokens` |

## 5. CLI surface

```
memd bench public --bench locomo \
  --typed-ingest=episodic+semantic+canonical \
  --compiler=on \
  --depth-routing=on \
  --reasoning=on \
  [--max-reasoning-steps 5] [--max-reasoning-tokens 20000]
```

`--reasoning=off` preserves the E6 single-call answer path.
`MEMD_V6_REASONING=0` forces off regardless of CLI flag.

## 6. Telemetry

Per-question NDJSON appended to
`docs/verification/v6-runs/<date>.ndjson`:

```json
{
  "ts": "<iso8601>",
  "bench_id": "<bench-id>",
  "question_id": "<id>",
  "scratchpad": { /* step schema */ },
  "retrieval_tokens": <usize>,
  "terminated_by": "answer | step_cap | token_cap"
}
```

The pure harness returns the metrics struct; IO is deferred to the
runtime dispatch layer (graduates with A6.9/B6/C6/D6/E6 post-2026-05-02).

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_REASONING` | unset → on | `0` forces off; otherwise honours the CLI flag. |
| `MEMD_V6_MAX_REASONING_STEPS` | unset | Numeric override for the step cap. |
| `MEMD_V6_ALLOW_BELOW_TARGET` | `0` | 10-STAR regen refuses composite < 7.0 unless `1`. |

Independent of `MEMD_V6_DEPTH_ROUTING`. Toggling reasoning off does
not deactivate the E6 router.

## 8. Bench targets (F6 — fixture proxies until graduation)

| Bench / subset | Target | Proxy fixture |
| --- | --- | --- |
| LME temporal | accuracy lift ≥ +0.03 (vs E6-only baseline) | `tests/fixtures/typed_ingest/f6/lme-temporal-10.jsonl` |
| LoCoMo sequential | accuracy lift ≥ +0.04 (vs E6-only baseline) | `tests/fixtures/typed_ingest/f6/locomo-sequential-10.jsonl` |
| Canonical gate (LME) | `qa_accuracy ≥ 0.85` | `tests/fixtures/typed_ingest/f6/canonical-gates.jsonl` |
| Canonical gate (LoCoMo) | `token_f1_avg ≥ 0.75` | same |
| Canonical gate (MemBench) | `mc_accuracy ≥ 0.75` | same |
| Canonical gate (ConvoMem) | `judge_accuracy ≥ 0.90` | same |
| Retrieval gate (LME) | `session_recall_any@5 ≥ 0.95` | same |
| Composite | MEMD-10-STAR ≥ 7.0 (publishable claim) | computed by `star_regen` |

Real-corpus locks graduate with A6.9 / B6 / C6 / D6 / E6 runtime
activation post-2026-05-02. V6 milestone target (4.45 composite) will
publish via `--allow-below-target`; the 7.0 publishable-claim gate
remains pinned for future milestones.

## 9. Versioning

Schema version `iterative-reasoning/v1` surfaced in runtime notice +
NDJSON. Bumping major invalidates older traces.

## 10. Out of scope

- Mutating E6. Reasoning harness is a *driver* on top of
  `run_router`; it does not reach into the parser or caps.
- Recursive in-step reasoning loops (the harness is single-loop;
  nested reasoning would require a v2 schema).
- Mutating the bench-compiler. D6 stays upstream and untouched.
