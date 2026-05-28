# Setup Experience Scorecard

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

This score measures hands-on setup/product experience. It is not a benchmark score and not a roadmap percentage. Architecture depth does not raise this score unless a real user can feel it during setup.

## North star: Apple-level setup

The score answers one question: can anybody install memd, understand what is happening, prove first memory works, recover from common failures, and trust it as a product that just works?

A 25-star claim requires the product experience, not just internal proof:

- one obvious start point
- plain-language install and setup copy
- polished guided/interactive path
- exact proof that first memory works
- clear data/privacy explanation
- beginner failure messages with symptom/cause/fix/verify
- update and uninstall paths that are safe by default
- repeated clean-room and external human setup trials

## 10-star setup gate

10-star means a fresh developer can install, configure providers/harnesses, verify, and recover from common setup drift using docs and CLI output only. No live maintainer handholding on the happy path.

## 25-star setup gate

25-star means Apple-level hands-on setup: a non-expert can start from zero, finish setup, understand the product, and see it reliably work across install, first run, update, repair, and uninstall. Internal automation can only mark **25-star implementation complete, external validation pending** until real external users pass the trial.

## Axes

| Axis | 0 | 5 | 10 | 15 | 20 | 25 |
| --- | --- | --- | --- | --- | --- | --- |
| Comprehension | no clear start | maintainer can infer | README routes fresh dev | non-expert understands | docs teach mental model | anybody can explain success/failure |
| Install success | manual guessing | source installer works | fresh checkout works | clean-room Linux proof | packaged install path | public install works across supported OSes |
| Guided config | flags only | basic prompts | arrow/Enter provider + harness picker | guided summary/json path | polished setup center | product-quality setup with no docs detour |
| First-run proof | none | doctor only | doctor + status + resume | isolated setup demo | capture/recall demo | repeated real-user first-memory proof |
| Failure recovery | raw errors | some docs | symptom/cause/fix/verify | beginner issue codes | safe self-repair | common failures feel routine, not scary |
| Documentation | scattered | maintainer notes | setup docs complete | screenshots/examples | task-based docs for all OSes | thorough understandable docs for anybody |
| Trust/privacy | unknown | consent docs | data location clear | update/uninstall safe | external trust review | product trust strong enough for daily memory |
| Reliability | unknown | local smoke | smoke script passes | clean-room proof passes | regression gate blocks setup drift | product just works repeatedly |

## 10-star checklist

- [x] README has install, interactive config, health, first-use proof, and failure path.
- [x] `START-HERE.md` routes first-time users before maintainer recovery.
- [x] `docs/setup/` has install, interactive, first-run, troubleshooting, update, uninstall, and privacy pages.
- [x] `memd setup --interactive` has centered arrow/Enter provider and harness picker.
- [x] `scripts/install-memd.sh` points to interactive setup and proof commands.
- [x] `scripts/verify/setup-experience-smoke.sh` runs setup proof commands.
- [x] `memd setup --guided` exposes the beginner path without needing this doc.
- [x] `memd setup-demo --summary` proves setup in a temp bundle.
- [x] `scripts/verify/local-25-star-product-proof.sh` aggregates local proof gates and external blockers.
- [ ] Fresh developer trial passes without live handholding.

## Honest status

This branch can make memd **10-star-ready** internally. It can also build toward **25-star implementation complete, external validation pending**. A true 25-star claim still needs repeated clean-room installs and external human trials.
