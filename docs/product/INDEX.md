> Product education doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]]. This page is a local navigation and claim-safety guide, not external validation.

# Product Education: What memd Is, What Is Proven, and Where to Start

## New-User Path

If you are new to memd, read in this order:

1. `START-HERE.md` - the first-run path and setup orientation.
2. `README.md` - what the project is trying to provide and the current quickstart.
3. `docs/WHERE-AM-I.md` - current project truth and recovery context.
4. `docs/product/INDEX.md` - this plain-language product map.
5. `docs/verification/features.registry.json` - machine-readable feature truth.
6. `docs/verification/feature-docs-product-education-25.md` - local proof map for this docs slice.

Do not start with old benchmark, phase, or backlog pages unless a current page points you there. Those pages can be useful history, but they are not automatically current product truth.

## Plain-Language Product Summary

memd is being built as a memory layer for developer/agent workflows. The intended product helps capture useful working context, recover continuity across sessions, and make claims about product behavior traceable to proof.

Current honest status: memd has working local surfaces and verification scripts, but this repository does **not** yet have complete external validation, sustained dogfood windows, or independent auditor evidence for a 25/25 product claim. Local proof can pass; external validation remains pending unless a cited artifact says otherwise.

## What Users Can Trust From These Docs

| Claim type | What the docs may say | Required proof | Current docs posture |
| --- | --- | --- | --- |
| Navigation | A new reader can follow a start path without needing hidden context. | This page, `START-HERE.md`, and doc lint. | Local proof only. |
| Feature truth | Feature claims are tracked in the registry. | `docs/verification/features.registry.json` plus registry audit. | Registry-backed, not product-complete. |
| Product behavior | A command or workflow works. | A named executable proof command and current artifact. | Only claim behavior where proof is linked. |
| 25-star readiness | The product is ready for high-confidence release. | Local proof, dogfood, and external/auditor replay. | Not yet; blockers remain registered. |

## Jargon Guardrail

Use these plain terms in product education pages:

- "memory layer" instead of unexplained architecture labels.
- "proof command" instead of unexplained harness language.
- "local proof" for checks run by this repository.
- "external validation pending" when no independent evidence exists.
- "dogfood evidence pending" when no dated usage window exists.

If a technical term is necessary, define it the first time it appears. Product education must not assume readers already know project-internal labels.

## Claim-to-Proof Rule

Every strong product claim needs one of these:

1. a registry row that allows the claim,
2. an executable command listed in that row, and
3. a verification artifact or report that explains the result.

If any part is missing, phrase the claim as pending or planned. Never convert planned work, stale proof, or local-only proof into an external or production-readiness claim.

## External Validation

External validation status for this docs/product education slice: **pending**.

Acceptable wording:

- "Local docs/product education proof passes."
- "External validation is pending."
- "Dogfood evidence remains pending unless a dated window artifact is added."

Forbidden wording:

- "External verification is complete" without an external artifact.
- "Highest product rating achieved" while registry blockers remain.
- "Ready for production" based only on doc lint or registry audit.

## Maintenance Checklist

When product docs change:

1. Update this page if navigation or claim wording changes.
2. Update only the matching `feature.docs_product_education` registry row if status/proof changes.
3. Run `bash scripts/verify/feature-docs-product-education-proof.sh`.
4. Run `bash scripts/verify/feature-registry-audit.sh`.
5. Run `bash scripts/doc-lint.sh`.
6. Run `git diff --check`.
