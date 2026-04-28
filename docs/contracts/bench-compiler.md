# Bench Compiler Contract — V6 / D6

**Version:** `bench-compiler/v1`
**Status:** scaffold-symmetric (runtime activation calendar-gated post-2026-05-02 alongside A6.9 / B6 / C6)
**Pipeline position:** retrieval → typed-ingest (A6/B6/C6) → **bench-compiler (D6)** → prompt render → judge.

## 1. Purpose

Apply V4 D4's wake-context compiler to public-benchmark answer prompts.
The compiler is a pure transform: typed buckets → priority order →
cross-bucket dedupe → budget admission → markdown render. D6 is a
bench-side shim that adapts typed-ingest records into `CompilerInput`
and re-uses `runtime::resume::compiler::compile_wake` unchanged.

## 2. Budget profile

`.memd/benchmarks/public/compiler-budgets.json` is the per-bench budget
table. Loaded by `compiler::load_budget_profile(bench_id)`.

Schema:

```json
{
  "version": "bench-compiler/v1",
  "benches": {
    "<bench-id>": {
      "budget_tokens": <usize>,
      "priority": ["canonical", "preferences", "recent_episodic", "semantic", "raw_episodic"]
    }
  }
}
```

`budget_tokens` follows the V4 convention: char count, surfaced via the
"tokens" name for legacy compatibility (see
`runtime/resume/compiler/mod.rs` doc comment).

`priority` orders typed sections from most-to-least preserved on
overflow. Priority labels are the bench-shim vocabulary; the shim
translates them to V4 `BucketKind` for `compile_wake`:

| Bench label | V4 `BucketKind` |
| --- | --- |
| `canonical` | `Canonical` |
| `preferences` | `Preference` |
| `recent_episodic` | `Focus` |
| `semantic` | `Semantic` |
| `raw_episodic` | `Episodic` |

## 3. Overflow policy

Inherited from V4 D4: priority-ordered admission with kinds-coverage
floors honoured first; lowest-priority sections demoted to `memd
lookup` hints when budget is exhausted. Demotion hints are reported
in the per-question telemetry NDJSON, not in the prompt body.

## 4. CLI surface

```
memd bench public --bench lme --typed-ingest=episodic+semantic+canonical --compiler=on
memd bench public --bench lme --compiler=off
```

`--compiler=off` (default until D6.7 graduation) preserves the legacy
flat-RAG prompt path verbatim — no compiler call, no admission, no
demotion. The off-path is pinned by test 6
(`flat_rag_path_unchanged_when_off`).

## 5. Telemetry

Per-question NDJSON written to
`.memd/benchmarks/public/results/compiler-<date>.ndjson`. Schema:

```json
{
  "ts": "<iso8601>",
  "bench_id": "<bench-id>",
  "question_id": "<id>",
  "budget_tokens": <usize>,
  "compiled_tokens": <usize>,
  "sections_included": ["canonical", "..."],
  "sections_dropped": ["..."],
  "tokens_before_drop": <usize>
}
```

Telemetry is appended by the runtime layer; the pure shim returns the
metrics struct, IO is deferred to dispatch.

## 6. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_COMPILER` | unset → behaves as `0` | When `1`/`true`/`on`, treat `--compiler=on` as the implicit default for bench runs. Off otherwise. |

The bench-compiler is independent of the V4 `MEMD_D4_COMPILER` flag
(which routes the live wake path). They can be toggled independently.

## 7. Versioning

Bumping the rule-card or budget schema bumps the `version` field in
`compiler-budgets.json`. The shim refuses to load on unknown major
versions; minor bumps are forward-compatible. Prior compiled prompts
are not invalidated — the compiler is a read-side transform with no
durable artefacts.

## 8. Bench targets (D6.4 – D6.6 — fixture proxies until graduation)

| Bench | Target | Proxy fixture |
| --- | --- | --- |
| LME | mean prompt-tokens drop ≥ 25% | `tests/fixtures/typed_ingest/d6/overflow-scenario.jsonl` (10 questions, baseline-vs-compiled token columns) |
| MemBench | accuracy lift ≥ +0.03 | same fixture, hits-vs-misses columns |
| LoCoMo | accuracy lift ≥ +0.03 | same fixture |
| Regression guard | ConvoMem + canonical paths preserved | covered by test 10 |

Real-data lock graduates with A6.9 / B6 / C6 runtime activation post-
2026-05-02.

## 9. Out of scope

- Mutating the V4 D4 compiler. D6 is strictly a wrapper.
- Token-count semantics. Inherits V4's char-as-tokens convention.
- Routing live wake. Owned by `MEMD_D4_COMPILER`.
- Promotion / dedupe rules. Owned by C6 + V4 dedupe.
