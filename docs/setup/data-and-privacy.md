# Data and Privacy

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

## Where data lives

Project setup writes local bundle files under `.memd/`.

Important files:

- `.memd/wake.md`
- `.memd/mem.md`
- `.memd/events.md`
- `.memd/state/`
- `.memd/compiled/`

## What leaves the machine

Local setup is local-first. Network behavior depends on configured `MEMD_BASE_URL`, `MEMD_RAG_URL`, sync, or hive settings.

## Dogfood consent

`memd dogfood enroll --consent` means this machine can count as dogfood evidence.

## Secrets

Do not paste tokens, passwords, private keys, or full session values into bug reports.
