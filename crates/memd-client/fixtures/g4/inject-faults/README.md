# G4 Fault Injection Fixtures

These fixtures are loaded by harness assertion tests 3–8 (see
`docs/phases/v4/phase-g4-plan.md` §4) to verify the proof harness
fails loudly when a V4 phase regresses. Each fault file is a thin
override on top of `seed-state.json` + `session-*.jsonl`; the harness
applies the override before running and expects an assertion failure
on the corresponding cut.

| File | Test | Simulates |
| --- | --- | --- |
| `a4-skip-postcompact-restore.json`   | Test 3 | A4 PostCompact restore hook silently no-ops at session-2 wake. |
| `b4-silent-hook-swallow.json`        | Test 4 | B4 hook trace omits a `PostToolUse` line; assertion must catch the gap. |
| `c4-correction-missing-provenance.json` | Test 5 | C4 stores correction C2 without `provenance` field. |
| `d4-wake-exceeds-budget.json`        | Test 6 | D4 compiler emits a wake brief above 2000 tokens. |
| `e4-lookup-stale.json`               | Test 7 | E4 lookup returns `uuid` for the `primary ID` query (pre-correction value). |
| `f4-drift-undetected.json`           | Test 8 | F4 drift detector silently skips the verbose-drift turn. |

Authored shells only — concrete schema is finalized when the harness
driver lands in G4.2 and the assertions module lands in G4.3, since
both will read these files.
