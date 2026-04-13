# .planning/ Ghost References in Tests

Status: `closed` — false positive, intentional project fixture setup
Created: 2026-04-13
Phase: cross-phase

26 occurrences across 6 files create `.planning/` directories in temp fixtures.
Some test `.planning/` integration (valid), some should use `.memd/` (the canonical
bundle dir). Original count of 7 was from shallow scan.

## Locations

- `evaluation_runtime_tests_support.rs:927,939,977` — creates `.planning/`, asserts `.planning/STATE.md`
- `evaluation_runtime_tests_support.rs:1012,1075` — creates `.planning/` in fixtures
- `verifier_runtime.rs:703-707` — creates `.planning/` for sender/target in verifier
- `tasks_hive_tests/mod.rs:107,231,355,476,738,933` — 6 hive test fixtures
- `gap_coordination_tests/mod.rs:954,971` — creates `.planning/` + `STATE.md`
- `bootstrap_harness_tests/mod.rs:1301,1364` — 2 bootstrap tests
- `workflow/improvement/mod.rs:721` — fallback string `"planning evidence unavailable in .planning"`
