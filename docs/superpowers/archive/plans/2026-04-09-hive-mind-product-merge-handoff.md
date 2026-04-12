# Hive Mind + Product Parallel Lab Merge Handoff

## Goal

Merge `feature/product-parallel-lab` and `feature/hive-mind` without duplicating the capability product surface or losing the newer continuity, task, memory, and maintenance work.

## Branch Ownership

### `feature/product-parallel-lab`

Owns:

- first-class `memd capabilities` product surface
- capability query/apply behavior
- capability bridge product UX

Do not replace this branch's capability command flow with older or parallel capability work from `feature/hive-mind`.

### `feature/hive-mind`

Owns:

- session continuity overlays across runtime surfaces
- hive awareness / coordination truth
- `memd tasks` summary-first cowork views
- `memd memory` truth-first runtime summary and JSON
- maintenance trend / history / auto recommendation surfaces

## Overlap Risk

Both branches touch:

- `crates/memd-client/src/main.rs`
- `crates/memd-client/src/render.rs`

The overlap is concentrated around capability surfaces. Task, memory, continuity, and maintenance work from `feature/hive-mind` should be preserved. Capability command UX from `feature/product-parallel-lab` should win.

## Safe Merge Order

1. Start from `feature/product-parallel-lab`
2. Bring over non-capability `feature/hive-mind` commits after `857e2e0`
3. Resolve `main.rs` and `render.rs` by keeping:
   - capability command flow from `feature/product-parallel-lab`
   - task views, memory truth JSON, continuity summaries, maintenance history/auto recommendation from `feature/hive-mind`
4. Re-run full `memd-client` verification

## Hive Mind Commits To Harvest

Keep:

- `c983620` selectively:
  - task summary-first surface
  - truth-first memory surface
  - exclude capability command flow if it conflicts with product branch
- `f2baa66`
  - task views
  - memory JSON
  - maintenance history
- `74c0429`
  - continuity overlay on claims/messages
  - richer memory contradiction/supersession reasons
  - status cowork view breakdowns

Already shared base:

- `526d02d`
- `857e2e0`

## Conflict Resolution Rules

When `main.rs` conflicts:

- keep `Commands::Capabilities` behavior from `feature/product-parallel-lab`
- keep `TasksArgs.view`, `TasksArgs.json`, `run_tasks_command` filtering
- keep `read_memory_surface` truth-first JSON fields
- keep `read_bundle_status` maintenance history / auto recommendation
- keep claims/messages continuity overlay fields and summaries

When `render.rs` conflicts:

- keep status rendering for:
  - cowork views
  - maintenance auto recommendation/history
- keep product branch capability rendering if there is disagreement

## Verification Gate

Must pass after integration:

```bash
cargo fmt --all
cargo test -p memd-client --bin memd
cargo test -p memd-server
```

Manual smoke:

```bash
memd capabilities --summary
memd tasks --summary
memd tasks --view owned --summary
memd memory
memd memory --json
memd status --summary
memd claims --summary
memd messages --summary
```

## Merge Outcome

Target merged product should have:

- one capability surface
- one continuity truth model
- one compact task cowork surface
- one truth-first memory JSON surface
- one maintenance auto/history surface
