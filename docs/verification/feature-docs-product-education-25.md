> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Docs/Product Education Local 25-Star Readiness Proof

Feature: `feature.docs_product_education`

This page records local proof for product education quality. It is not external validation, dogfood evidence, or a claim that whole-product 25/25 has been achieved.

## Scope

In scope:

- A plain-language new-user path for understanding memd.
- A claim-to-proof map for docs/product education claims.
- Guardrails that prevent local-only docs proof from being described as external validation.
- A script that fails if the required docs, setup examples, CLI argument surfaces, internal links, registry/report rows, or claim-honesty language drift.

Out of scope:

- Proving every memd product behavior.
- Claiming sustained dogfood usage.
- Claiming external/auditor validation.

## Claim-to-Proof Map

| Claim | Proof source | Local gate | Honest limit |
| --- | --- | --- | --- |
| New users have a product education start path. | `docs/product/INDEX.md` section `New-User Path`. | `bash scripts/verify/feature-docs-product-education-proof.sh` | Navigation quality only; does not prove product behavior. |
| Product education distinguishes local proof from external validation. | `docs/product/INDEX.md` sections `External Validation` and `What Users Can Trust From These Docs`. | `bash scripts/verify/feature-docs-product-education-proof.sh` | External validation remains pending. |
| Product education claims are tied to registry/proof artifacts. | `docs/verification/features.registry.json` row `feature.docs_product_education`. | `bash scripts/verify/feature-registry-audit.sh` and this slice proof script. | Registry truth is only as current as its cited commands/artifacts. |
| Docs use a plain-language path instead of assuming project jargon. | `docs/product/INDEX.md` section `Jargon Guardrail`. | `bash scripts/verify/feature-docs-product-education-proof.sh` | The script checks required wording and banned unexplained terms; human review still matters. |
| Start-here/README/setup/CLI alignment is locally checked. | `START-HERE.md`, `README.md`, `docs/setup/README.md`, and clap argument definitions under `crates/memd-client/src/cli`. | `bash scripts/verify/feature-docs-product-education-proof.sh` | Static CLI alignment checks prove documented flags exist; they are not a full runtime CLI acceptance test. |
| Broken internal references in the education path are blocked. | Product education path docs and setup docs. | `bash scripts/verify/feature-docs-product-education-proof.sh` | Scope is the beginner/product education path, not every historical/backlog document. |
| Setup command examples remain concrete and discoverable. | README Quickstart plus `docs/setup/*.md`. | `bash scripts/verify/feature-docs-product-education-proof.sh` | Command examples prove documentation alignment, not every OS/install environment. |
| Registry claim honesty is enforced. | `docs/verification/features.registry.json`, `docs/verification/FEATURES.md`, and `docs/verification/feature-coverage-report.md`. | `bash scripts/verify/feature-registry-audit.sh` and this slice proof script. | Strong local status is allowed only while external/dogfood/25/25 blockers stay explicit. |
| Unsupported 25/25 claims are blocked. | Product docs, proof doc, registry report, and FEATURES table. | `bash scripts/verify/feature-docs-product-education-proof.sh` | This prevents docs overclaims; it does not itself satisfy whole-product 25/25. |

## Strong Local Proof Coverage

`bash scripts/verify/feature-docs-product-education-proof.sh` validates these local 25/5 gates:

1. Start-here/README/setup/CLI alignment for the beginner setup path.
2. Broken internal references in the docs/product education path.
3. Setup command examples for install, guided setup, interactive setup, doctor, status, resume, setup-demo, and repair.
4. Registry claim honesty across `features.registry.json`, `FEATURES.md`, and `feature-coverage-report.md`.
5. No unsupported 25/25 claims, no external-validation overclaim, and repeated pending language for dogfood/external limits.

This is strong local proof, not external validation.

## Required Local Gates

Run from the repository root:

```bash
bash scripts/verify/feature-registry-audit.sh
bash scripts/verify/feature-docs-product-education-proof.sh
bash scripts/doc-lint.sh
git diff --check
```

`bash scripts/memd-cargo-guard.sh -- check -p memd-client` is not required for this slice unless code changes.

## Current Local Result

- Local proof status: strong when the commands above pass at the final commit.
- Dogfood status: ad hoc only as recorded in the registry; no sustained dated window is claimed here.
- External validation status: pending. No independent external/auditor artifact is cited by this slice.

## Honest Release Language

Allowed after all local gates pass:

- "Docs/product education has local proof for navigation, claim-to-proof mapping, and honest pending language."
- "This slice improves local 25/5 docs/product education readiness."

Not allowed from this proof alone:

- "memd is 25/25."
- "Product docs are externally verified."
- "The full product behavior is proven by docs."
