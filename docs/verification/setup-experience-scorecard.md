# Setup Experience Scorecard

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

This score measures hands-on setup/product experience. It is not a benchmark score and not a roadmap percentage.

## 10-star setup gate

10-star means a fresh developer can install, configure providers/harnesses, verify, and recover from common setup drift using docs and CLI output only. No live maintainer handholding on the happy path.

## Axes

| Axis | 0 | 5 | 10 | 15+ |
| --- | --- | --- | --- | --- |
| Comprehension | no clear start | maintainer can infer | README routes fresh dev | non-expert understands |
| Install success | manual guessing | source installer works | fresh checkout works | public package works |
| Interactive config | flags only | basic prompts | arrow/Enter provider + harness picker | polished full-screen guided setup |
| First-run proof | none | doctor only | doctor + status + resume | demo proves capture/recall |
| Failure recovery | raw errors | some docs | symptom/cause/fix/verify | doctor self-repairs safe drift |
| Trust/privacy | unknown | consent docs | data location clear | external trust review |

## 10-star checklist

- [x] README has install, interactive config, health, first-use proof, and failure path.
- [x] `START-HERE.md` routes first-time users before maintainer recovery.
- [x] `docs/setup/` has install, interactive, first-run, troubleshooting, update, uninstall, and privacy pages.
- [x] `memd setup --interactive` has centered arrow/Enter provider and harness picker.
- [x] `scripts/install-memd.sh` points to interactive setup and proof commands.
- [x] `scripts/verify/setup-experience-smoke.sh` runs setup proof commands.
- [ ] Fresh developer trial passes without live handholding.

## Honest status

This branch can make memd **10-star-ready** internally. The final 10-star claim still needs a fresh developer trial.
