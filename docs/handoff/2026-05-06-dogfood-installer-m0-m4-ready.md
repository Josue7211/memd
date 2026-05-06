---
opened: 2026-05-06
phase: v20-evidence-ops
status: dogfood-installer-m0-m4-ready
prev_handoff: 2026-05-06-security-sweep-complete-v20-evidence-ops-next.md
branch: main
directive: freeze substrate scope; only evidence-blocker fixes before 1.0.0
mode: 10-star-ceo
---

# Dogfood Installer M0-M4 Ready

One sentence: V20 evidence ops now has a first friend/new-machine onboarding
path: install script, device registration, dogfood enrollment, status gate, and
friend packet docs.

## Current Truth

- Evidence clock was already open as of 2026-05-06.
- This change is an evidence-ops unblocker, not new substrate scope.
- Friend setup now starts from `scripts/install-memd.sh`.
- Real dogfood enrollment now starts from `memd dogfood enroll --user-id <id> --consent --summary`.
- Device evidence now starts from `memd device add --summary`.
- `memd dogfood status --summary` reports enrollment/device gate progress and the next action.
- `memd doctor --summary` now includes missing harnesses and an exact setup repair command when `setup_ready=false`.
- Friend-facing packet lives at `docs/DOGFOOD.md`.

## What Landed

- M0: checkout installer script.
- M1: device add command for new-machine evidence.
- M2: doctor/status next-step output for failed setup readiness.
- M3: dogfood enroll/status commands with dated local state and release evidence artifacts.
- M4: friend packet and README quickstart.

## Verification

- `cargo check -p memd-client` passed.
- `cargo build -p memd-client --bin memd` passed.
- Temp-bundle smoke passed:
  - `memd device add --output .memd --user smoke --name smoke-mac --summary`
  - `memd dogfood enroll --output .memd --user-id smoke --consent --summary`
  - `memd dogfood status --output .memd --summary`
- `bash -n scripts/install-memd.sh` passed.
- `git diff --check` passed.

## Next Actions

- Run the installer on one clean secondary machine before sending to friends.
- Fix any installer/doctor blocker found on that clean machine.
- Then enroll three real users and three harness-user pairs.
- Put three devices on current `main`.
- Keep weekly review note due 2026-05-13.
- Do not tag `1.0.0` until real dated artifacts land.

## Known Caveat

Native harness bridge readiness still depends on local harness presence. If
`doctor` reports `setup_ready=false`, follow its `setup_next` command first,
then fix any harness-specific red item it reports.
