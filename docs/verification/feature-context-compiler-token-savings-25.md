# Feature context_compiler_token_savings 25-star local proof

[[ROADMAP]]: Verification artifact for the local context compiler/token-savings feature slice. This is not an external benchmark, production guarantee, or product-wide savings claim.

## Scope

Feature registry id: `feature.context_compiler_token_savings`

This proof covers the local claim that memd can compile a compact context under a token budget while retaining required facts and reporting honest token savings. It combines:

- a deterministic lightweight fixture ledger in `scripts/verify/feature-context-compiler-token-savings-proof.sh`
- existing compiler/token proof surfaces when present
- explicit external-verification pending status

## Local proof command

Run:

```bash
bash scripts/verify/feature-context-compiler-token-savings-proof.sh
```

The command validates this document, the registry entry, the coverage report row, existing compiler/token proof citations, saved-token arithmetic, quality-retention checks, budget enforcement, and honest pending external status.

## Existing compiler/token proof surfaces

The local proof script requires and cites these existing in-repository proof scripts/artifacts when present:

| Surface | Path | Role in this slice |
| --- | --- | --- |
| V15 token self-tuning proof script | `scripts/verify/v15-self-tuning-suite.sh` | Existing executable proof for self-tuning token savings, quality guardrails, and budget profiles. |
| V15 dated proof artifact | `docs/verification/v15-proof-runs/2026-05-12-self-tuning-suite.md` and `.ndjson` | Historical local result with minimum savings vs V11 dynamic and quality delta. |
| V11 compiler SOTA proof script | `scripts/verify/v11-compiler-sota-suite.sh` | Existing executable proof for dynamic compiler/cost target and wake median budget behavior. |
| V11 dated proof artifact | `docs/verification/v11-proof-runs/2026-05-12-compiler-sota-suite.md` and `.ndjson` | Historical local result for dynamic compiler/token-efficiency axis. |

This slice does not re-label those historical artifacts as external verification. They are local repository evidence only.

## Lightweight fixture ledger

The fixture is embedded in `scripts/verify/feature-context-compiler-token-savings-proof.sh` and uses a stable, tokenizer-agnostic whitespace token count so the arithmetic is reproducible without network or model access.

| Case | Baseline tokens | Compiled tokens | Saved tokens | Savings | Required facts retained | Quality retention | Budget |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| project-switch-resume | 96 | 48 | 48 | 50.00% | 6/6 | 100.00% | 60 |
| correction-aware-resume | 77 | 42 | 35 | 45.45% | 5/5 | 100.00% | 50 |
| provenance-budget-trim | 89 | 49 | 40 | 44.94% | 5/5 | 100.00% | 55 |
| **total** | **262** | **139** | **123** | **46.95%** | **16/16** | **100.00%** | all passed |

## Quality retention

Each fixture case declares required facts as string assertions. The compiled context must retain every required fact; optional stale/noisy lines may be dropped. The local quality gate is strict for this fixture: every required fact must be retained, giving `16/16` retained facts and `100.00%` fixture quality retention.

## Budget enforcement

Each fixture case has a maximum compiled-context budget. The proof fails if any compiled context exceeds its budget. Current fixture budgets pass:

- `project-switch-resume`: `48 <= 60`
- `correction-aware-resume`: `42 <= 50`
- `provenance-budget-trim`: `49 <= 55`

## Current claim level

- `current_status`: partial
- `proof_status`: strong local proof when `bash scripts/verify/feature-context-compiler-token-savings-proof.sh` passes on this commit
- `dogfood_status`: ad_hoc
- `external_status`: planned/pending

Allowed claim: local context-compiler/token-savings proof has a reproducible saved-token ledger, retains all fixture-critical facts, enforces fixture budgets, and is supported by existing local V11/V15 compiler/token artifacts.

Forbidden claim: do not claim externally verified savings, universal quality retention, production-wide savings percentages, or benchmark dominance from this artifact alone.
