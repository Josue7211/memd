---
phase: F7
name: User-Visible "I learned X from Y" Surface
version: v7
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [E7]
phase_doc: docs/phases/v7/phase-f7-user-visible-learned-surface.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: correction_retention, trust_provenance
---

# Phase F7 — Implementation Plan

## 0. Executive summary

Wake-time + CLI + digest surfaces recently promoted corrections. Respects visibility. Opt-out via env. No new capture path.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `crates/memd-core/src/correction/learned_surface.rs` | Recent-corrections renderer. |
| `crates/memd-client/src/commands/learned.rs` | CLI. |
| `crates/memd-core/src/correction/digest.rs` | Daily-digest writer. |
| `crates/memd-core/src/main_tests/learned_tests/mod.rs` | Tests. |

### Files to modify

| Path | Change |
| --- | --- |
| Wake-compilation path (V4 D4) | Insert `## Recently Learned` section when surface has content. |
| `memd-client/src/lib.rs` | Wire `learned` subcommand. |
| Phase doc. |

## 2. Schema changes

None.

## 3. API shape

```
memd learned [--since 7d] [--since-session <id>] [--json] [--top 10]
```

Wake insert:

```markdown
## Recently Learned
- YYYY-MM-DD: <brief text>. [source: turn <id>, chain: <N> links]
- …
```

## 4. Test matrix

1. `renderer_emits_top_n_dedup`
2. `renderer_respects_visibility_scope`
3. `renderer_excludes_rolled_back_corrections`
4. `wake_insert_only_when_content_nonempty`
5. `wake_insert_respects_opt_out_env`
6. `cli_learned_happy`
7. `cli_learned_json`
8. `cli_learned_since_session`
9. `digest_writes_daily_markdown`
10. `visibility_leak_guard_never_surfaces_private`

## 5. Fixtures

- `tests/fixtures/correction/f7/corrections-mixed-scope.jsonl` — project + local + global.

## 6. Telemetry

`.memd/logs/learned-digest-<date>.md` daily digest.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_LEARNED_SURFACE` | `1` | Disable wake insert + digest. |
| `MEMD_LEARNED_TOP_N` | `10` | Override top-N. |

## 8. Task list

### Task F7.1 — renderer

- [ ] Tests 1 + 2 + 3 failing.
- [ ] Commit: `feat(correction/f7): learned renderer (F7)`.

### Task F7.2 — wake insert

- [ ] Tests 4 + 5 failing.
- [ ] Commit: `feat(correction/f7): wake insert (F7)`.

### Task F7.3 — CLI

- [ ] Tests 6 + 7 + 8 failing.
- [ ] Commit: `feat(cli/f7): memd learned (F7)`.

### Task F7.4 — digest

- [ ] Test 9 failing.
- [ ] Commit: `feat(correction/f7): daily digest (F7)`.

### Task F7.5 — visibility guard

- [ ] Test 10 failing.
- [ ] Commit: `feat(correction/f7): visibility guard (F7)`.

### Task F7.6 — CI + dogfood

- [ ] CI renders samples; dogfood for 7 days.
- [ ] Commit: `ci(f7): learned surface dogfood (F7)`.

## 9. Bench impact

None directly; improves user trust → fewer correction misses over time (indirect lift to B5).

## 10. Dependency graph

- Requires: E7.
- Blocks: G7 dogfood scenario.

## Exit criteria

1. Tests 1–10 green.
2. Wake insert visible; CLI working.
3. Visibility leak guard green.
4. Digest sample committed.
5. Atomic commits.
