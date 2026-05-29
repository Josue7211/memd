# Feature Setup/Install/Onboarding 25-Star Proof

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

This is the local proof record for `feature.setup_install_onboarding`. It is intentionally narrow: it proves the setup/install/onboarding slice without making docs/product-education claims outside the setup lane.

## Claim status

**Local claim:** 25-star implementation complete, external validation pending.

**Not claimed:** public 25/25, universal one-command install, production onboarding, or external-human success. External human validation remains pending until fresh users complete repeatable setup trials without maintainer help.

## User contract

A new local user can:

1. find the beginner path from README or START-HERE,
2. install from a checkout,
3. run guided or interactive onboarding,
4. prove first use with doctor/status/resume/setup-demo,
5. understand update and uninstall safety,
6. see clearly that external validation is still pending.

## Local proof gates

Run:

```bash
bash scripts/verify/feature-setup-install-onboarding-proof.sh
```

The proof validates:

- install docs mention `scripts/install-memd.sh`, `memd doctor --summary`, and first status proof;
- update docs mention `scripts/update-memd.sh --dry-run`, `scripts/update-memd.sh`, and `.memd` preservation;
- uninstall docs mention `scripts/uninstall-memd.sh --dry-run`, `scripts/uninstall-memd.sh`, and memory preservation by default;
- beginner path exists in README, START-HERE, `docs/setup/README.md`, and `docs/setup/first-run.md`;
- proof commands include `bash scripts/verify/setup-experience-smoke.sh` and this proof script;
- lifecycle dry-runs pass without mutating installed binaries or `.memd`;
- registry still blocks an honest public 25/25 claim until external validation is complete.

The proof script also runs:

```bash
bash scripts/verify/setup-experience-smoke.sh
```

Set `MEMD_SETUP_ONBOARDING_SKIP_SMOKE=1` only for quick doc-only iteration; do not use the skip for final feature proof.

## Setup docs in scope

- [Setup overview](../setup/README.md)
- [Install](../setup/install.md)
- [First run](../setup/first-run.md)
- [Update](../setup/update.md)
- [Uninstall](../setup/uninstall.md)
- [Troubleshooting](../setup/troubleshooting.md)
- [Data and privacy](../setup/data-and-privacy.md)

## External validation remains pending

External validation requires replay by people outside the maintainer loop. Minimum future evidence should include:

- fresh checkout setup on supported platforms,
- no live handholding on the happy path,
- install/update/uninstall understanding checked by user feedback,
- captured failure reports for confusing steps,
- explicit pass/fail artifacts linked from the registry.

Until then, the only allowed 25-star wording for this slice is: **25-star implementation complete, external validation pending**.
