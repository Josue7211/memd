---
opened: 2026-05-06
phase: v16-to-v20-ceo-mode
status: handoff-ready
prev_handoff: 2026-05-06-v15-code-complete-dogfood-next.md
branch: main
repo_state: pending commit at packet creation
directive: execute everything remaining through V20 and 1.0.0 proof bundle
mode: 10-star-ceo
release_note: Next agent owns the full ceiling push from V16 to V20; keep wall-clock dogfood gates honest.
---

# V16 to V20 CEO Mode - Execute All Remaining

One sentence: pick up from V15 code-complete and drive the remaining 10-STAR
ceiling push through V20 and the 1.0.0 proof bundle, with no fake closure of
real-time dogfood gates.

## Pickup

```bash
cd /Volumes/T7/projects/memd
git status --short --branch
sed -n '1,220p' .memd/wake.md
memd lookup --output .memd --query "v16 v17 v18 v19 v20 10 star CEO mode remaining roadmap handoff"
sed -n '1,220p' docs/handoff/LATEST.md
sed -n '1,140p' ROADMAP.md
```

Expected pickup state after this packet commit: clean `main`, ahead
`origin/main`.

## Non-Negotiables

- Work from durable truth: `.memd/wake.md`, `memd lookup`, `ROADMAP.md`, and milestone docs.
- Execute in order: V16 -> V17 -> V18 -> V19 -> V20.
- Keep each version atomic. Prefer one commit per completed version substrate plus one final release-prep commit.
- Do not mark any real-time dogfood gate closed without real elapsed evidence.
- Synthetic proof can mark `code_complete_dogfood_pending`; it cannot mark final close.
- Every milestone needs code, tests, proof artifacts, roadmap update, milestone doc update, and handoff update.
- Before every commit: `cargo fmt --check`, `git diff --check`, targeted tests, milestone proof script.
- Remove Apple sidecars before commit:

```bash
find . -path './.git' -prune -o -path './target' -prune -o -name '._*' -type f -delete
```

## Current State

- V13 closed at composite `8.50`.
- V14 telemetry substrate code complete, provisional composite `8.60`; real >=30-day, >=3-user telemetry dogfood remains pending.
- V15 self-tuning compiler substrate code complete, provisional composite `8.70`; real >=60-day, >=3 harness-user-pair tuning dogfood remains pending.
- Latest V15 proof:
  - `docs/verification/v15-proof-runs/2026-05-06-self-tuning-suite.md`
  - minimum savings vs dynamic: `27.73%`
  - minimum quality delta: `+0.02`
- Current roadmap state: V15 `code_complete_dogfood_pending`.

## Mission

Ship the rest of the ceiling push:

| Version | Target | Composite | Axis Lift |
| --- | --- | --- | --- |
| V16 | Cross-device sync at scale | 8.70 -> 9.05 | SC 9->10, CH 8->9 |
| V17 | Cross-user routine economy | 9.05 -> 9.35 | PR 9->10, CH 9->10 |
| V18 | Correction graph + silent detection | 9.35 -> 9.50 | CR 8->9 |
| V19 | Zero-knowledge provenance | 9.50 -> 9.75 | CR 9->10, TP 9->10 |
| V20 | Info-theoretic TE + bench ceiling + 1.0.0 | 9.75 -> 10.00 | RR 9->10, TE 9->10 |

## V16 Plan

Source: `docs/verification/milestones/MILESTONE-v16.md`.

Build:
- CRDT memory layer for `MemoryRecord`.
- Sync protocol with opt-in relay or peer-to-peer mode.
- Multi-device wake/read path with <=2s post-sync visibility.
- Offline merge and deterministic conflict resolution.
- Cross-device replay harness.
- Config keys: `sync.enabled`, `sync.relay_url`, `sync.conflict_policy`.
- Proof script: `scripts/verify/v16-sync-suite.sh`.

Gate:
- Synthetic conflict scenario passes with no data loss.
- Same turn sequence on two devices produces identical memory state.
- Dormant-project replay proof exists.
- Real >=90-day, 3-device dogfood remains pending unless real evidence exists.

## V17 Plan

Source: `docs/verification/milestones/MILESTONE-v17.md`.

Build:
- Content-addressed routine marketplace schema.
- Author, version, reputation, allowlist, blocklist.
- Parameterized routine generalization from >=3 traces.
- `memd routines marketplace search|browse|install`.
- Federation scale test for >=1000 synthetic users.
- Zero-data-leakage proof that private citations are stripped.
- Proof script: `scripts/verify/v17-routine-marketplace-suite.sh`.

Gate:
- >=10 parameterized routines validated.
- >=1000-user synthetic federation test passes.
- Adversarial leakage audit passes.
- Real >=30-day marketplace dogfood remains pending unless real evidence exists.

## V18 Plan

Source: `docs/verification/milestones/MILESTONE-v18.md`.

Build:
- Correction graph edges: `cites`, `supersedes`, `affects`.
- Multi-hop propagation engine.
- Silent correction detector v2 with measurable precision/recall harness.
- Downstream-effect surfacing in query output.
- Correction graph export format.
- Third-party replay harness.
- Proof script: `scripts/verify/v18-correction-graph-suite.sh`.

Gate:
- Synthetic multi-hop chains traced end to end.
- Detector metrics meet >=0.90 precision and >=0.85 recall on labeled fixture.
- Third-party replay deterministic on exported fixtures.
- Real >=3-month dogfood and >=50 real multi-hop chains remain pending unless real evidence exists.

## V19 Plan

Source: `docs/verification/milestones/MILESTONE-v19.md`.

Build:
- ZK proof system selection note with rationale.
- Correction-applied proof circuit or pragmatic proof substrate with explicit limits.
- Standalone verifier: `memd audit verify-zk <proof>`.
- Multi-party attestation, two-of-three signing.
- Compliance-grade audit UI/export path.
- External auditor replay fixture.
- Proof script: `scripts/verify/v19-zk-provenance-suite.sh`.

Gate:
- >=10 generated correction-applied proofs verify.
- Tamper-evidence catches post-hoc audit-log mutation.
- Multi-party attestation works end to end.
- External auditor smoke artifacts remain pending unless real evidence exists.

## V20 Plan

Source: `docs/verification/milestones/MILESTONE-v20.md`.

Build:
- Info-theoretic token removal prover.
- Public bench domination sweep for LongMemEval, LoCoMo, MemBench, ConvoMem.
- memd-authored harder bench with competitor comparison fixtures.
- Zero-shot domain generalization test.
- 1.0.0 aggregate release harness with every axis = 10 assertion.
- Third-party replay export for every axis.
- Proof bundle under `docs/verification/release-1-0-0/`.
- Proof script: `scripts/verify/v20-release-suite.sh`.

Gate:
- Do not tag `1.0.0` unless every axis is genuinely proven at 10.
- If any V20 axis misses, file V20.5 recovery instead of lowering the bar.
- Third-party replay evidence is required before release close.

## Execution Pattern Per Version

1. Read the milestone doc and relevant existing code.
2. Add the smallest substrate that satisfies the planned phases.
3. Add focused tests and a version proof script.
4. Run targeted tests and proof script.
5. Update milestone status to `code_complete_dogfood_pending` or `closed` only if gate evidence is real.
6. Update `ROADMAP.md` truth block and status snapshot.
7. Add proof artifacts under `docs/verification/vXX-proof-runs/`.
8. Add or refresh handoff.
9. Stage intended files only.
10. Commit with `feat(vXX): ...`.

## Final CEO Checklist

- V16 proof substrate landed.
- V17 proof substrate landed.
- V18 proof substrate landed.
- V19 proof substrate landed.
- V20 proof substrate landed.
- `docs/verification/release-1-0-0/` exists with aggregate proof bundle.
- `ROADMAP.md` has exact current truth.
- No dogfood gate is lied about.
- Tree clean.
- Handoff says what remains, with dates and artifacts.

## If Time Is Short

Prioritize in this order:

1. V16 CRDT/sync substrate and proof.
2. V18 correction graph, because V19 depends on it.
3. V19 verifier/audit substrate.
4. V20 aggregate proof harness.
5. V17 marketplace, unless cross-user federation code already has a clear local surface.

But the user intent is clear: next agent should attempt the whole V16-V20 ceiling push, not stop after planning.
