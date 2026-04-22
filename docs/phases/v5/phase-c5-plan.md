---
phase: C5
name: CrossHarnessContinuity Bench
version: v5
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [V4, A5]
phase_doc: docs/phases/v5/phase-c5-cross-harness-continuity.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: cross_harness
---

# Phase C5 — Implementation Plan

## 0. Executive summary

Drives two real harnesses (claude-code, codex) through scripted sessions that write and read the same memd state. Measures truth conservation + visibility-leak (hard 0) + latency. Harness adapters are thin shims over each harness's CLI.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-client/src/benchmark/substrate/cross_harness.rs` | C5 runner. |
| `crates/memd-client/src/benchmark/substrate/harness_adapter/mod.rs` | Adapter trait. |
| `crates/memd-client/src/benchmark/substrate/harness_adapter/claude_code.rs` | claude-code driver. |
| `crates/memd-client/src/benchmark/substrate/harness_adapter/codex.rs` | codex driver. |
| `.memd/benchmarks/substrate/cross-harness.yaml` | Spec. |
| `.memd/benchmarks/substrate/fixtures/c5/` | Fixture sessions. |
| `crates/memd-client/src/main_tests/substrate_c5_tests/mod.rs` | Integration tests. |

### Files to modify

| Path | Change |
| --- | --- |
| `crates/memd-client/src/benchmark/substrate/mod.rs` | Register C5. |
| `docs/phases/v5/phase-c5-cross-harness-continuity.md` | `plan_spec:` line. |

## 2. Schema changes

None. Bench-only.

```yaml
suite: cross-harness
version: 1
harness_pairs:
  - [claude_code, codex]
  - [codex, claude_code]
scenarios: [fact_roundtrip, preference_roundtrip, correction_roundtrip]
visibility_scopes: [project, local, global]
pass_gate:
  truth_conservation_rate: 0.95
  visibility_leak_count: 0   # hard floor
  latency_p95_ms: 2000
```

## 3. API shape

```
memd bench substrate --suite cross-harness \
  [--pair claude_code:codex] \
  [--scope project] \
  [--skip-harness codex]          # CI graceful skip if harness unavailable
```

Adapter contract:

```rust
pub trait HarnessAdapter {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn run_script(&self, script: &Script) -> Result<HarnessRunOutcome>;
}
```

## 4. Test matrix

1. `adapter_claude_code_detects_availability_via_settings_json`
2. `adapter_codex_detects_availability_via_hooks_json`
3. `adapter_run_script_writes_and_reads_via_memd_cli`
4. `runner_roundtrips_fact_claude_to_codex`
5. `runner_visibility_leak_zero_on_project_scope`
6. `runner_visibility_leak_detected_on_planted_breach` — fault-injection test.
7. `cli_c5_gracefully_skips_unavailable_harness`
8. `cli_c5_happy_both_pairs`
9. `cli_c5_reproducibility_same_seed`
10. `c5_baseline_lock`

## 5. Fixtures

`.memd/benchmarks/substrate/fixtures/c5/`:

| File | Contents |
| --- | --- |
| `fact-roundtrip.jsonl` | 10 facts in write-script, 10 queries in read-script. |
| `preference-roundtrip.jsonl` | 5 prefs. |
| `correction-roundtrip.jsonl` | 5 corrections. |
| `visibility-planted-breach.jsonl` | Deliberately leaks local scope for fault-injection test. |

## 6. Telemetry

Per-harness NDJSON + combined report section.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_SUBSTRATE_C5_HARNESS_ALLOW_SKIP` | `1` in CI, `0` locally | Graceful skip when harness binary missing. |

## 8. Task list

### Task C5.1 — adapter trait + claude-code impl

- [ ] Tests 1 + 3 failing.
- [ ] Implement adapter trait.
- [ ] Wire claude-code driver: read `~/.claude/settings.json` for hook config, run scripted messages via subprocess.
- [ ] Green.
- [ ] Commit: `feat(bench/c5): adapter + claude-code driver (C5)`.

### Task C5.2 — codex adapter

- [ ] Test 2 failing.
- [ ] Drive codex via its CLI; read `~/.codex/hooks.json` for config.
- [ ] Green.
- [ ] Commit: `feat(bench/c5): codex adapter (C5)`.

### Task C5.3 — runner + visibility auditor

- [ ] Tests 4 + 5 + 6 failing.
- [ ] Implement cross-pair runner; visibility auditor reads memd after read-side session and asserts scope integrity.
- [ ] Green.
- [ ] Commit: `feat(bench/c5): runner + visibility auditor (C5)`.

### Task C5.4 — CLI + skip

- [ ] Tests 7 + 8 + 9 failing.
- [ ] Wire dispatcher; implement graceful skip path.
- [ ] Green.
- [ ] Commit: `feat(bench/c5): CLI + graceful skip (C5)`.

### Task C5.5 — baseline + CI

- [ ] Test 10 + CI wiring (CI env must have both harnesses or skip with recorded reason).
- [ ] Commit: `bench+ci(c5): baseline + CI integration (C5)`.

### Task C5.6 — 10-STAR

- [ ] cross_harness axis bump.
- [ ] Commit: `docs(10-star): C5 cross_harness +N`.

## 9. Bench impact

C5 unlocks cross-harness claims. G5 aggregation requires.

## 10. Dependency graph

- Requires: A5 runtime; access to claude-code + codex binaries on CI (see `docs/HARNESS_BRIDGES.md` — note that doc is currently inverted; read `~/.claude/settings.json` and `~/.codex/hooks.json` directly).
- Blocks: G5.
- Parallelizable with B5/D5/E5/F5.

## Exit criteria

1. Tests 1–10 green; visibility-leak test catches planted breach.
2. Truth-conservation ≥0.95 on both pairs.
3. Visibility-leak = 0.
4. CI green (or recorded skip).
5. 10-STAR cross_harness bumped.
6. Atomic commits.
