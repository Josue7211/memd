---
date: 2026-04-24
kind: handoff
from: v4-a4-executor
to: v4-b4-executor
status: ready-to-execute
entry_phase: B4
branch: research/mining (ahead of main by 11 commits)
---

# Handoff — A4 closed, execute B4

## TL;DR

A4 (Read-State Across Compaction) shipped clean. Ledger survives PreCompact → PostCompact byte-for-byte; breach telemetry wired; `memd hook doctor --check ordering` enforces the normative contract. 10-STAR session_continuity axis 1 → 2; composite 1.80 → 2.00. Next agent opens `docs/phases/v4/phase-b4-plan.md` and executes B4 atomic.

## Repo state at handoff

- Branch: `research/mining` at `21592b0`, 11 commits ahead of `main` (`3306a74`). Not pushed. Not merged.
- A4 commit range: `60c369d..b7edcc5` (9 atomic + 1 auto-checkpoint).
- Working tree: clean.
- Roadmap: `current_phase=B4`, `phase_status=ready_to_execute`.

## A4 commits (chronological)

```
60c369d feat(memd-core/file_ledger): restore module locates and applies sealed ledger (A4)
c31cf34 feat(memd-client/hook): add memd hook restore verb (A4)
488f487 feat(memd-core): breach log line for missing sealed ledger (A4)
99e176e feat(hooks): PostCompact restore hook + MANIFEST entries (A4)
b565062 feat(memd-client/hook): doctor --check ordering (A4)
19a7348 docs(contracts): hook-handoff contract for A4 (ledger survival across compaction)
6a1780b test(memd-client): A4 compaction-survival and breach-detection scenarios
700d66b docs(10-star): axis 1 rescored after A4 pass gate
b7edcc5 docs(handoff): A4.9 deferred flag flip — schedule MEMD_A4_LEDGER_SURVIVAL=1
21592b0 memd auto-commit: A4 complete (current_task checkpoint)
```

## What is proven

- `memd hook seal-ledger` → `memd hook restore` round-trips 5/5 paths (scenario 18).
- `memd hook doctor --check ordering` flags tool-before-restore + missing-restore (scenario 19).
- `scripts/verify/a4-loop.sh 10` → pass=10/10 deterministic, no network, no LLM.
- `cargo test -p memd-core file_ledger::` — 20/20 green.
- `cargo test -p memd-client --bin memd continuity_compaction_tests::` — 10/10 green.
- Release build: `cargo build --release -p memd-client --bin memd` — clean, 8 pre-existing warnings.

## What is deferred

- **A4.9 default flip** — schedule doc at `docs/handoff/2026-04-24-a4-default-on.md`. Flip date 2026-05-01 gated on zero breach lines during dogfood. Flag lives on hook scripts at `.memd/hooks/memd-postcompact-restore.{sh,ps1}`; default-0, opt-in via `MEMD_A4_LEDGER_SURVIVAL=1`.
- **V3 tail** — canonical rerun of LongMemEval / LoCoMo / ConvoMem via codex-lb (`http://127.0.0.1:2455/v1`). Separable follow-up, does not block V4.

## Next step — B4 entry

Open `docs/phases/v4/phase-b4-plan.md`. B4 consumes A4 deliverables:

- Hook runner MUST emit `logs/hook-trace.ndjson` using the tokens in `docs/contracts/hook-handoff.md` §1.
- `memd hook doctor --check ordering` already audits the trace — B4 just needs to keep its emitter honest against the contract.
- B4 does NOT touch the ledger restore path. If B4 needs a new hook token, update `hook-handoff.md` first, then the state machine in `cli_hook_runtime.rs::detect_ordering_breaches`.

B4 dependency: `docs/contracts/hook-handoff.md` (A4.6, landed `19a7348`). No other A4 prerequisites.

## Invariants (do not regress)

1. PostCompact hook MUST exit 0 even on `no-sealed-ledger` (non-blocking).
2. Restore is idempotent — running twice yields identical on-disk state.
3. Newest-sealed selection parses u64 from filename stem — do NOT switch to mtime (NFS clock skew).
4. Breach log is append-only, `<rfc3339-utc> <session_id> breach=<kind>[ key=value]*` format. V7 owns rotation; A4/B4 only append.
5. `MEMD_A4_LEDGER_SURVIVAL` default stays `0` until 2026-05-01 gate clears.

## How to verify before B4.1

```
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd continuity_compaction_tests:: --no-fail-fast
scripts/verify/a4-loop.sh 10
```

Both must be green. If either is red, do NOT start B4 — diagnose first.

## 10-STAR snapshot after A4

| Axis | Before | After | Weight | Contribution |
|------|--------|-------|--------|--------------|
| Session continuity | 1/10 | 2/10 | 20% | +0.20 |
| Correction retention | 1/10 | 1/10 | 15% | — |
| Procedural reuse | 1/10 | 1/10 | 15% | — |
| Cross-harness continuity | 2/10 | 2/10 | 15% | — |
| Raw retrieval strength | 4/10 | 4/10 | 15% | — |
| Token efficiency | 2/10 | 2/10 | 10% | — |
| Trust + provenance | 2/10 | 2/10 | 10% | — |

**Composite: 1.80 → 2.00.**

V4 milestone composite target: 3.45. Remaining lift from B4+C4+D4+E4+F4+G4.
