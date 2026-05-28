# Local 25-Star Product Scorecard

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

This is the local-only whole-app 25-star gate. It is intentionally stricter than the setup scorecard but still honest: local proof can reach **local 25-star implementation complete, external validation pending**. True 25-star requires the external verifier checklist in `docs/verification/EXTERNAL-25-STAR-VERIFIERS.md`.

## Definition

Local 25-star means memd behaves like a product, not a research repo:

1. setup is obvious and beginner-safe
2. first memory works in an isolated clean root
3. capture/recall/resume surfaces work without tribal knowledge
4. supported harnesses are wired or explicitly blocked with fixes
5. doctor/status explain issues with exact next actions
6. update/uninstall are safe by default
7. docs answer beginner and operator questions
8. proof scripts reproduce the claim locally
9. every external-only claim is listed as pending, not smuggled into the score

## Local axes

| Axis | 25-star local requirement | Local proof |
| --- | --- | --- |
| Setup comprehension | README/START-HERE route beginner path, guided setup, troubleshooting, privacy, update, uninstall | `memd setup --guided --summary`; doc-lint |
| Install/update/uninstall | install, update, uninstall flows are safe and memory-preserving by default | `scripts/update-memd.sh --dry-run`; `scripts/uninstall-memd.sh --dry-run` |
| First memory proof | isolated temp-root setup creates readable bundle and startup memory surface | `memd setup-demo --summary` |
| Doctor/failure recovery | beginner issue codes map symptom/cause/fix/verify | `docs/setup/failure-registry.md`; `memd doctor --summary` |
| Harness readiness | priority harnesses named, bridge status visible, missing harnesses are surfaced | `memd status --output <bundle> --summary` |
| Data trust | data location and what leaves machine are documented | `docs/setup/data-and-privacy.md` |
| Reliability | smoke proof leaves git clean and can run repeatedly | `scripts/verify/setup-experience-smoke.sh` |
| Whole-app proof packet | local scorecard, verifier list, and proof report are generated | `scripts/verify/local-25-star-product-proof.sh` |

## Local status language

Allowed after local proof passes:

> memd local 25-star implementation proof passed; true 25-star remains external-validation pending.

Not allowed until Josue completes the external checklist:

> memd is true 25-star.
> memd is externally validated.
> memd is Apple-level for anybody.

## External blockers

See `docs/verification/EXTERNAL-25-STAR-VERIFIERS.md`. The user owns those external gates.
