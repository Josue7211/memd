# memd-client fixtures

Fixture dirs for V4 phases. Ownership per `docs/phases/v4/V4-INTEGRATION.md` §2.

## Layout

```
fixtures/
├── shared/
│   ├── sessions/       # multi-phase session fixtures (G4 owns, A4/C4/D4 consume)
│   ├── preferences/    # preference drift fixtures (F4 owns, D4/G4 consume)
│   ├── transcripts/    # aligned + drift 10-turn transcripts (F4 owns, G4 consumes)
│   └── hook-traces/    # canonical-trace.ndjson (B4 owns, A4/G4 consume)
├── a4/                 # A4-exclusive: pre-compact-ledger.json, post-compact-expected.json
│                         5-file synthetic session transcript
├── b4/                 # B4-exclusive (create on B4 start)
├── c4/                 # C4-exclusive (create on C4 start)
├── d4/                 # D4-exclusive (create on D4 start)
├── e4/                 # E4-exclusive (create on E4 start)
├── f4/                 # F4-exclusive (create on F4 start)
└── g4/                 # G4-exclusive (create on G4 start)
```

## Promotion rule

A fixture graduates from `<phase>/` to `shared/` the moment a second phase
references it. Leave a compat shim (symlink or `pub use`) in the original
dir for one phase cycle before removing.
