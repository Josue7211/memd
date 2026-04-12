# Public Benchmark Parity Design

## Goal

Add a public benchmark lane to `memd` that reproduces the external benchmark style used by MemPalace, but with stronger auditability, stricter claim governance, and tighter integration with the existing `memd` quality system.

The purpose of this lane is not operator health. The purpose is to make defensible public claims such as:

- `memd` raw beats another system's raw baseline on the same public dataset
- `memd` hybrid beats another system's hybrid result on the same public dataset
- every reported result is rerunnable from published commands and recorded manifests

## Core Decision

`memd` should keep three separate quality lanes:

- `memd benchmark`
  cheap structural/operator health
- `memd verify`
  product-truth verification for continuity, handoff, hive, and adversarial flows
- `memd benchmark public`
  external benchmark parity and leaderboard claims

This separation is mandatory. Public benchmark claims must not be mixed into the operator benchmark score, and operator health must not be confused with state-of-the-art public benchmark performance.

## Public Benchmark Targets

The public benchmark lane should reproduce the same benchmark families that MemPalace highlights:

- `LongMemEval`
- `LoCoMo`
- `ConvoMem`
- `MemBench`

Each benchmark should be exposed as its own command family:

- `memd benchmark public longmemeval`
- `memd benchmark public locomo`
- `memd benchmark public convomem`
- `memd benchmark public membench`

The first shipped slice should implement the generic framework plus `LongMemEval`. The architecture must still be designed for all four from day one.

## Claim Classes

Every public result must declare a claim class:

- `raw`
- `hybrid`

Rules:

- `raw` means no model-assisted reranking or answer refinement inside the scored path
- `hybrid` means a model-assisted rerank or equivalent hybrid retrieval stage is active
- `hybrid` results must never be presented as offline or zero-API results
- `raw` and `hybrid` tables must remain distinct even when both are shown together

This is a hard honesty rule. If a run uses a reranker, that fact must be present in the CLI output, manifest, markdown summary, and leaderboard.

## CLI Surface

Recommended shape:

- `memd benchmark public longmemeval --mode raw`
- `memd benchmark public longmemeval --mode hybrid --reranker <id>`
- `memd benchmark public locomo --mode raw`
- `memd benchmark public convomem --mode raw`
- `memd benchmark public membench --mode raw`

Shared flags:

- `--mode <raw|hybrid>`
- `--top-k <n>`
- `--limit <n>`
- `--dataset-root <path>`
- `--reranker <id>`
- `--write`
- `--json`
- `--out <path>`

Optional future flags:

- `--granularity <session|turn|dialog>`
- `--category <name>`
- `--split <name>`
- `--compare-against mempalace`

The command should always print a concise summary in normal mode and a complete machine-readable payload in `--json` mode.

## Dataset Policy

Real benchmark datasets should not be vendored into the repo by default.

Policy:

- datasets download on demand
- every download is checksum verified
- datasets are cached under `.memd/benchmarks/datasets`
- tiny synthetic fixtures are vendored only for tests

This is better for:

- repo size
- licensing safety
- reproducibility
- CI portability

Each dataset adapter should define:

- canonical source URL(s)
- expected checksum(s)
- normalization rules
- split metadata
- local cache path rules

## Run Manifest

Every run must emit a manifest. Without it, the run is not a valid public claim.

Manifest fields:

- benchmark id
- benchmark version
- dataset name
- dataset source URL
- dataset local path
- dataset checksum
- dataset split
- git SHA
- dirty worktree flag
- run timestamp
- mode
- top-k
- reranker id
- reranker provider
- limit
- runtime settings
- hardware summary
- duration
- token usage
- cost estimate

This manifest must live beside the result artifacts and be referenced by the generated markdown report.

## Audit Artifacts

Every public benchmark run should write:

- aggregate JSON
- aggregate markdown
- per-item JSONL
- retrieval trace artifacts
- failure-case artifact
- leaderboard comparison snapshot

The important rule is that every headline score must be explorable down to the item level.

Per-item JSONL records should contain at least:

- item id
- question/prompt id
- claim class
- retrieved items
- retrieval scores
- hit/miss flags
- answer output when applicable
- correctness metrics when applicable
- latency
- token/cost fields

This is how `memd` beats MemPalace on rigor, not just on score reporting.

## Metric Separation

Public benchmark reporting should separate:

- retrieval metrics
- answer metrics
- latency metrics
- token/cost metrics

Do not collapse these into one score.

Examples:

- retrieval:
  - `Recall@5`
  - `Recall@10`
  - `NDCG@10`
- answer:
  - exact match
  - F1
  - abstention correctness
- runtime:
  - time per query
  - total benchmark duration
- cost:
  - prompt tokens
  - completion tokens
  - estimated dollar cost

This prevents cheap headline inflation.

## Leaderboard Policy

`memd` should generate a comparison table against MemPalace-reported public numbers, but only under strict claim governance.

Leaderboard columns:

- benchmark
- metric
- MemPalace reported
- memd raw
- memd hybrid
- delta
- rerunnable status
- notes

Claim rules:

- no `beat` claim unless the same benchmark, split, metric, and claim class are matched
- no `SOTA` claim unless the result is rerunnable with published commands and valid manifests
- no hybrid number may be compared against a raw/offline number as if they are equivalent

This rule should be enforced by the reporting layer itself.

## Anti-Cheating Rule

Benchmark adapters may normalize data and scoring. They may not hide benchmark-specific production shortcuts.

Allowed:

- adapter-level normalization
- benchmark-specific parsing
- benchmark-specific scoring

Disallowed unless explicit in the manifest and mode:

- hidden benchmark-only retrieval shortcuts in the default product path
- benchmark-only memory injection not available in the scored mode contract
- undeclared reranking or answer assistance

This is necessary if the benchmark lane is going to support public credibility.

## Architecture

The public benchmark lane should use a generic framework with benchmark-specific adapters.

Core framework responsibilities:

- dataset fetching and checksum verification
- benchmark run manifest generation
- output directory layout
- shared result schemas
- artifact writing
- leaderboard rendering

Adapter responsibilities:

- parse dataset format
- normalize benchmark items
- execute retrieval or retrieval-plus-answer passes
- compute dataset-native metrics

This allows one shared system with four benchmark adapters.

## Files And Artifacts

Recommended runtime artifact layout:

- `.memd/benchmarks/datasets/`
- `.memd/benchmarks/public/<benchmark-id>/latest/manifest.json`
- `.memd/benchmarks/public/<benchmark-id>/latest/results.json`
- `.memd/benchmarks/public/<benchmark-id>/latest/results.jsonl`
- `.memd/benchmarks/public/<benchmark-id>/latest/report.md`
- `.memd/benchmarks/public/<benchmark-id>/latest/failures.json`
- `.memd/benchmarks/public/leaderboard.json`
- `.memd/benchmarks/public/leaderboard.md`

Recommended docs outputs:

- `docs/verification/PUBLIC_BENCHMARKS.md`
- `docs/verification/PUBLIC_LEADERBOARD.md`

These docs should be generated from artifacts, not edited by hand.

## Initial Rollout

Best rollout order:

1. generic public benchmark framework
2. `LongMemEval` adapter with `raw` and `hybrid`
3. generated manifest + JSONL + markdown outputs
4. leaderboard rendering with empty/unverified rows for the remaining datasets
5. `LoCoMo`
6. `ConvoMem`
7. `MemBench`

This gives the fastest path to a credible public lane without pretending all four are implemented on day one.

## Testing

The implementation should use:

- vendored tiny benchmark fixtures for unit tests
- adapter-level tests for parsing and scoring
- artifact writer tests for manifests and JSONL
- CLI tests for command parsing

The full real datasets should not be required for local unit tests.

## Completion Criteria

This design is complete only if `memd` can:

- run an exact-parity public benchmark lane beside the operator benchmark lane
- reproduce public benchmark runs with explicit `raw` and `hybrid` claim classes
- emit auditable per-item results and manifests
- generate a leaderboard with strict claim governance
- prevent misleading comparisons between raw and hybrid results

That is the 10-star public benchmark direction for `memd`.
