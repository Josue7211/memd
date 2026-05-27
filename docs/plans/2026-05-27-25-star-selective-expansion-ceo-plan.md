# 25-Star Selective Expansion CEO Mode Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Turn selective expansion CEO mode from trigger text into a first-class recall behavior that plans, retrieves, synthesizes, evaluates, and proves 25/5-star strategic answers without over-expanding simple lookup.

**Architecture:** Keep `memd lookup` cheap by default. Add a policy object that classifies the query, builds an expansion plan, selects recall stages (`needle -> thread -> ceo -> forensics`), and emits a CEO synthesis packet only for explicit/inferred strategy asks. Use structured telemetry and fixture-based evals to prove behavior.

**Tech Stack:** Rust, `memd-client`, existing recall runtime under `crates/memd-client/src/runtime/recall/`, recall depth tests under `crates/memd-client/src/main_tests/recall_depth_tests/`, docs contracts under `docs/contracts/`.

---

## Success bar

CEO mode is 25/5 for this slice when:

1. Explicit and inferred strategy asks produce a structured CEO packet.
2. Neutral/factual asks stay normal and do not pay expansion cost.
3. Expansion plan is visible in output and telemetry.
4. Tests prove stage selection, synthesis shape, and negative controls.
5. Docs define the contract honestly: this is CEO-mode recall behavior, not V21-V35 product/company proof.
6. Full targeted suite passes; unrelated latency issue is documented or fixed separately.

## Workstream A - Branch integration baseline

### Task A1: Rebase improved CEO work onto atomic local/T7 base

Create worktree `~/Documents/projects/memd-worktrees/25-star-ceo-integration` on branch `work/25-star-ceo-integration` from `work/internal-alpha-hivemind-validation-local`. Cherry-pick or manually apply improved commit `527658a750bf3660e1025d6b49d9735b8fd3439b`. Run `cargo fmt --check`, `cargo test -p memd-client selective_expansion -- --nocapture`, `cargo check -p memd-client`. Commit `feat(recall): integrate selective expansion CEO policy`.

Verification: `git merge-base --is-ancestor work/internal-alpha-hivemind-validation-local HEAD` passes.

## Workstream B - True expansion ladder

### Task B1: Add selective expansion policy model

Files: `crates/memd-client/src/runtime/recall/escalation.rs`, `crates/memd-client/src/main_tests/recall_depth_tests/mod.rs`.

Add data model:
- `SelectiveExpansionMode::{Normal, CeoExplicit, CeoInferred}`
- `ExpansionStage::{Needle, Thread, Ceo, Forensics}`
- `ExpansionPlan { mode, stages, reason, max_records_hint, include_forensics }`

Rules:
- Normal: `Needle` only.
- CEO explicit/inferred: `Needle, Thread, Ceo`.
- Forensics only when raw history/conflict/debug proof is requested.

Tests: explicit CEO, inferred CEO, neutral negative, forensics positive.
Commit: `feat(recall): model selective expansion plans`.

### Task B2: Wire expansion plan into lookup outcome

Files: `crates/memd-client/src/runtime/recall/mod.rs`, `crates/memd-client/src/runtime/recall/telemetry.rs`, tests.

Behavior:
- `LookupArmOutcome` carries `selective_expansion_plan`.
- Markdown contains compact plan line only when non-normal.
- Telemetry contains structured `selective_expansion` fields, not only text hint.

Commit: `feat(recall): emit selective expansion plan telemetry`.

## Workstream C - CEO synthesis packet

### Task C1: Create CEO synthesis packet renderer

Files: create `crates/memd-client/src/runtime/recall/ceo.rs`; modify recall module and tests.

Packet sections:
1. `Read` - what memory says is happening.
2. `Prize` - highest leverage target.
3. `Bottleneck` - current constraint.
4. `Moves` - 3-5 concrete next moves.
5. `Recommendation` - one primary call.
6. `Proof` - commands/artifacts needed to verify.

Rules: do not invent facts; label gaps; render only for CEO mode.
Commit: `feat(recall): render CEO synthesis packet`.

### Task C2: Add anti-hallucination gap labeling

If no durable matches, `Read` says `No durable match found`. `Proof` suggests exact next lookup/test command. Never claims 25-star product proof from code-only evidence.
Commit: `test(recall): pin CEO synthesis gap handling`.

## Workstream D - Quality evals and negative controls

### Task D1: Add fixture queries for CEO mode

Files:
- `crates/memd-client/fixtures/recall/selective-expansion-ceo-positive.jsonl`
- `crates/memd-client/fixtures/recall/selective-expansion-ceo-negative.jsonl`
- recall depth tests.

Positive examples: `CEO mode: make this 25/5 star`, `how do we make this 10 star`, `what are we missing`, `what is the bottleneck`.
Negative examples: `configuration files`, `how do I parse JSON`, `server logs`, `what is the exact command`.
Commit: `test(recall): add CEO selective expansion fixtures`.

### Task D2: Add quality contract tests

Assert CEO output has all six sections, telemetry includes mode/stages/reason, neutral query lacks CEO packet and expansion telemetry, and forensics requires explicit raw/conflict/debug wording.
Commit: `test(recall): prove CEO expansion quality contract`.

## Workstream E - Config and docs

### Task E1: Add recall contract docs

Files: `docs/contracts/recall-depth.md`, create `docs/contracts/selective-expansion-ceo.md`.

Docs must say default lookup stays cheap; CEO mode is selective expansion, not silent unlimited resume; 25-star product/company proof remains gated by `docs/verification/25-star-CONTRACT.md`; forensics expansion requires ambiguity/conflict/raw-history need.
Commit: `docs(recall): define selective expansion CEO contract`.

### Task E2: Optional config surface

Inspect existing config conventions. If cheap, load trigger policy from config. If not, document deferral with reason.
Commit either `feat(recall): load selective expansion trigger policy` or `docs(recall): defer configurable CEO trigger policy`.

## Workstream F - Known issue cleanup

### Task F1: Isolate wake latency failure

Run targeted recall depth suite on fast local worktree and compare with T7 if needed. If T7/exfat-only, document environment limitation with evidence. If regression, fix root cause.
Commit: `test(recall): document wake latency environment limit` or root-cause fix commit.

## Integration order

1. A baseline integration.
2. B policy/ladder.
3. C synthesis packet.
4. D evals/negative controls.
5. E docs/config.
6. F latency isolation.
7. Final integration review and full verification.

## Final verification

Run:

```bash
cargo fmt --check
cargo test -p memd-client selective_expansion -- --nocapture
cargo test -p memd-client recall_depth_tests -- --nocapture
cargo check -p memd-client
git diff --check
scripts/verify/25-star-roadmap-audit.sh
```

If `recall_depth_tests` latency fails only under T7/exfat, report that separately and do not claim fixed unless proven.
