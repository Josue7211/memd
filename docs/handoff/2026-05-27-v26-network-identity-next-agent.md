# Handoff: V26 Network Identity Next Agent

Date: 2026-05-27
Repo: `/home/josue/Documents/projects/memd`
Branch: `main`
HEAD: `b38405767ed4748ad5ef8603f3362e6bb2b22d6a`
Remote: `origin/main` at same HEAD
Voice: caveman-lite

## Start Here

Read these first:

1. `.memd/wake.md`
2. `.memd/mem.md`
3. `AGENTS.md`
4. `docs/strategy/25-star-master-roadmap.md`
5. `docs/contracts/selective-expansion-ceo.md`
6. `docs/plans/2026-05-27-25-star-selective-expansion-ceo-plan.md`

Mandatory recall before claims:

```bash
memd lookup --output .memd --query "V26 Network Identity next 25/5-star layer handoff selective expansion CEO mode"
```

## Current State

Selective expansion CEO mode is landed, merged, pushed, and verified.

Final shipped commits include:

- `c02a27f` docs: plan 25-star selective expansion CEO mode
- `f49f5fc` feat(recall): add selective expansion CEO synthesis
- `cd08e41` docs: add selective expansion CEO eval fixtures
- `5791c05` fix(recall): keep lookup-depth wake read-only
- `6508219` rag: support LightRAG sidecar payloads
- `c427921` fix(recall): render CEO selective expansion guidance
- `b384057` docs(strategy): set next 25/5-star layer

`main` and `origin/main` both point at:

```text
b38405767ed4748ad5ef8603f3362e6bb2b22d6a
```

T7 state:

- T7 repo: `/run/media/josue/T7/projects/memd`
- T7 final integration worktree: `/run/media/josue/T7/projects/memd-25-star-final-integration`
- T7 branch `t7/main-final` fetched from current local `main`
- Final integration branch still preserved for reference: `work/25-star-final-integration`

## Verification Already Run

Fresh final verification passed before this handoff:

```bash
cargo fmt --check
cargo test -p memd-client selective_expansion -- --nocapture
MEMD_AR=/usr/bin/ar bash scripts/memd-cargo-guard.sh -- test -p memd-client recall_depth_tests::latency_budgets_hold_on_fixture_set -- --nocapture
cargo test -p memd-rag
cargo test -p memd-client cli_embed_runtime
cargo check -p memd-client -p memd-rag
git diff --check
bash scripts/verify/25-star-roadmap-audit.sh
git push origin main
```

Observed proof:

- tests passed
- `25-star-roadmap-audit: ok`
- remote `main` equals local `main`
- root repo clean: `## main...origin/main`

## Cleanup Already Done

Removed temp worktrees:

- `~/Documents/projects/memd-worktrees/25-star-ceo-core`
- `~/Documents/projects/memd-worktrees/25-star-ceo-evals-docs`
- `~/Documents/projects/memd-worktrees/25-star-rag-latency`
- `~/Documents/projects/memd-worktrees/rag-lightrag-sidecar`
- `~/Documents/projects/memd-worktrees/selective-expansion-ceo`

Deleted temp branches:

- `work/25-star-ceo-core`
- `work/25-star-ceo-evals-docs`
- `work/25-star-rag-latency`
- `work/25-star-selective-expansion-plan`
- `work/rag-lightrag-sidecar-local`
- `work/rag-lightrag-sidecar-v2`
- `work/selective-expansion-ceo-local`
- `work/selective-expansion-ceo-v2`

Preserved:

- `work/25-star-final-integration`
- locked Claude agent worktrees under `.claude/worktrees`
- T7 final integration worktree

## Next Work

Next 25/5-star layer is **V26 Network Identity**.

Decision was recorded in `docs/strategy/25-star-master-roadmap.md` under:

```text
## Current Next 25/5-Star Layer: V26 Network Identity
```

Why V26 next:

- CEO mode now improves strategic recall and answer shape.
- Next leverage point is cross-app trust, not more synthesis.
- V26 starts the memory-network gate and unlocks federation, agent work markets, and external product adoption.

Target outcome:

- One portable user/org memory identity works across independent apps without private leakage.

Hard gate from roadmap:

- 3 independent apps authenticate and resolve the same memory identity.
- Zero cross-app private leakage in adversarial tests.
- Revocation takes effect in every app within the proof window.

Kill criterion:

- Identity requires app-specific forks.

Recovery:

- If gate fails, open V26.5 identity contract recovery before claiming network progress.

## Recommended First Task For Next Agent

Create the V26 plan file:

```text
docs/plans/2026-05-27-v26-network-identity-plan.md
```

Use A-G phase structure:

A. Capability: identity model, app grants, scoped resolver, revocation/transfer receipts.
B. UX: commands/docs showing app identity linking, grants, revoke, inspect.
C. Evidence harness: 3-app fixture matrix, adversarial cross-app leak tests, revocation timing checks.
D. Dogfood: run against memd CLI/server/dashboard or equivalent 3-app local setup.
E. External review: define reviewer checklist and replay packet.
F. Kill/recovery decision: decide pass/recovery using hard gate evidence.
G. Final proof packet: receipts, hashes, commands, audit summary.

Do not implement before plan unless user explicitly says skip planning.

## Commands To Re-Verify Starting State

```bash
cd ~/Documents/projects/memd
git status --short --branch
git rev-parse HEAD
git ls-remote origin refs/heads/main
bash scripts/verify/25-star-roadmap-audit.sh
```

Expected:

```text
## main...origin/main
b38405767ed4748ad5ef8603f3362e6bb2b22d6a
origin/main = b38405767ed4748ad5ef8603f3362e6bb2b22d6a
25-star-roadmap-audit: ok
```

## Important Cautions

- Do not claim V26 is complete from synthetic proof. The gate requires 3 independent apps and leakage/revocation proof.
- Do not delete locked `.claude/worktrees` unless the owner confirms.
- Do not overwrite T7 dirty/untracked state. T7 main repo had untracked `.hermes/`; leave it alone unless explicitly asked.
- For future T7/memd work: save dirty T7 work as atomic commits first, sync/clone local, then branch/worktree. Never patch-copy dirty work before saving commits.
- Use `MEMD_AR=/usr/bin/ar bash scripts/memd-cargo-guard.sh -- ...` for guarded cargo tests where needed.

## One-Line Resume Prompt

Continue memd from clean `main` at `b384057`: selective expansion CEO mode is shipped; next task is V26 Network Identity plan and proof harness design for 3 independent apps, zero cross-app leakage, and revocation proof.
