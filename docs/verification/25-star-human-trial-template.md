# 25-Star Human Trial Template

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

Give this to external verifiers. Do not explain extra context unless they get blocked; any extra help counts as `help_needed=yes`.

## Instructions for verifier

1. Start at README.
2. Install memd.
3. Run guided setup.
4. Prove first memory works.
5. If anything fails, use doctor and troubleshooting docs.
6. Fill the result below.

## Result form

```text
verifier_id:
name/role:
os:
shell:
harness used:
commit:
started_at:
finished_at:
time_to_first_memory:
commands_run:
result: pass|fail|blocked
help_needed: yes|no
what was confusing:
what felt polished:
what felt broken:
privacy/trust concerns:
artifacts/logs:
```

## Minimum pass

- no maintainer help
- can explain where `.memd` data lives
- can run `memd setup-demo --summary`
- can recover from one docs/doctor-guided issue or report that no issue occurred
