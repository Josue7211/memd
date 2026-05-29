# Feature 25 proof: doctor/status/recovery/update/uninstall

Reference doc: local verification artifact for the feature registry; see [[ROADMAP]] for product priority context.

## Scope

Feature id: `feature.doctor_status_recovery_update_uninstall`

This proof slice covers the local operator lifecycle needed for a 25-star recovery story:

- `memd doctor --summary` can inspect a configured bundle.
- `memd status --summary` can inspect the same bundle.
- `memd doctor --repair --summary` can recover generated bundle files without deleting memory-bearing files.
- `scripts/update-memd.sh --dry-run` advertises a non-mutating update path and memory preservation.
- `scripts/uninstall-memd.sh --dry-run` advertises a non-mutating uninstall path and memory preservation.
- Failure mode: `memd doctor` against a missing bundle reports `ready=false`, missing bundle parts, and a setup next step instead of silently claiming readiness.

## Executable proof

Run:

```bash
bash scripts/verify/feature-doctor-status-recovery-update-uninstall-proof.sh
```

The script builds `target/debug/memd` via `scripts/memd-cargo-guard.sh` if needed, creates a temporary project bundle, seeds sentinel memory/state files, and checks that the sentinel files survive doctor/status/repair plus update/uninstall dry-runs.

## Coverage matrix

| Requirement | Local check | Status |
| --- | --- | --- |
| Doctor health inspection | `memd doctor --output <tmp> --summary` | covered locally |
| Status inspection | `memd status --output <tmp> --summary` | covered locally |
| Recovery | delete generated `env`/`env.ps1`, then `memd doctor --repair --summary` recreates them | covered locally |
| Reset | no destructive `memd reset` lifecycle command exists in this slice | pending product command/contract |
| Update | `scripts/update-memd.sh --dry-run` prints intended steps and preserves `.memd` | covered as dry-run only |
| Uninstall | `scripts/uninstall-memd.sh --dry-run` prints binary-only removal and memory preservation | covered as dry-run only |
| Memory preservation | sentinel files under `.memd/memory/` and `.memd/state/` survive all local proof steps | covered locally |
| Failure modes | missing bundle doctor call must report `ready=false`, missing files, and setup next step | covered locally |
| External validation | independent machine/user replay | pending external |

## Honest limitations

This is strong local executable proof for the current repository surfaces, not external verification. It does not prove:

- an independent user can recover a real broken installation without maintainer help;
- update works against a dirty, diverged, or packaged release install;
- uninstall behavior for package managers, launch agents, services, or non-default binary locations;
- destructive reset semantics, because no explicit `memd reset` command/contract is present in this slice;
- preservation of every possible future memory store layout.

The registry should therefore remain blocked for 25/25 until reset semantics and external replay are complete.

## Freshness

Re-run this proof after changes to lifecycle commands, setup bundle layout, `.memd` storage, update/uninstall scripts, doctor/status output, or recovery behavior.
