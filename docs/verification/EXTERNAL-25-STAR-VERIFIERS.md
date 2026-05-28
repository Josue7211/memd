# External 25-Star Verifier Checklist

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

Josue owns these. Hermes can prepare packets, scripts, and instructions, but must not mark true 25-star until these are checked off with real external evidence.

## Required external verifiers

| ID | Verifier | Ask | Pass evidence |
| --- | --- | --- | --- |
| EV-01 | Non-technical first-time user | Start from README, install, run guided setup, prove first memory without help | screen recording or notes, pass/fail, time-to-first-memory |
| EV-02 | Developer new to repo | Clone fresh, install, run `memd setup-demo`, run smoke proof | terminal log + feedback |
| EV-03 | Agent/harness user | Use memd from one real harness: Codex, Claude Code, Hermes, OpenClaw, or OpenCode | transcript showing setup + resume/recall |
| EV-04 | Second harness user | Repeat from a different harness than EV-03 | transcript/log |
| EV-05 | Failure recovery tester | Intentionally break PATH or bundle config, use doctor/troubleshooting to recover | before/after log, no maintainer help |
| EV-06 | Privacy/trust reviewer | Read data/privacy doc and explain where data lives + what leaves machine | written signoff or issue list |
| EV-07 | Update tester | Run update path on existing bundle and confirm memory preserved | command log + pre/post status |
| EV-08 | Uninstall tester | Run dry-run uninstall and confirm it does not delete `.memd` by default | command log |
| EV-09 | Clean-room Linux tester | Fresh Linux machine/VM, run install/setup/proof | command log |
| EV-10 | macOS tester | Fresh macOS machine, run install/setup/proof; Mac Bridge optional but documented | command log |
| EV-11 | External product-minded reviewer | Judge if docs/product feel understandable and polished enough | written critique + blockers |
| EV-12 | Reliability tester | Run proof script 5 times or across 2 days with no flakes | logs |

## Optional stretch verifiers

- Windows/WSL setup tester.
- Security reviewer for install scripts.
- Competitor user comparing memd setup to Supermemory/MemPalace/Letta/mem0.
- Team/org reviewer trying shared memory/hive flow.

## Evidence packet template

For each verifier, capture:

```text
verifier_id:
person/role:
os:
harness:
commit:
commands_run:
result: pass|fail|blocked
help_needed: yes|no
blockers:
quotes/notes:
artifacts:
```

## Close rule

True 25-star requires EV-01 through EV-12 pass or have explicit accepted waivers. Any fail creates a local fix task before the claim can close.
