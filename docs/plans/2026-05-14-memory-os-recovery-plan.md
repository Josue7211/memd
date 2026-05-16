# memd 25/5 Recovery Plan

Status: active recovery plan.
Opened: 2026-05-14.

## Verdict

memd is not ready for any 25/5, market-best, or monopoly claim.

The current feature registry reports implementation features as working, but
that is not enough. The real state has serious blockers:

- Repo hygiene is broken: untracked external public benchmark caches added
  about 23 million lines under `docs/verification/25-5-memory-os-runs/`.
- Token efficiency is broken at the repo/process level: giant raw cache files
  can enter review/context surfaces and burn tokens before memd saves any.
- Market proof is blocked: Supermemory same-fixture replay is not passing.
- Full external public proof is blocked by policy until explicitly allowed.
- Docs are stale: older plans describe target architecture better than current
  operational truth.
- Proof artifacts are mixed with raw data caches, making review noisy and
  dangerous.
- Feature status is too coarse: `working` does not show dogfood quality,
  cleanup state, adapter freshness, or market-proof state.

## P0 Rule

No more broad benchmark runs, no more cache writes inside the repo, and no
more star claims until cleanup and proof discipline are fixed.

memd's core promise is token efficiency: read once, remember forever, and avoid
rereading unchanged raw sources. A repo containing 23M untracked cache lines
violates that promise because every agent, reviewer, diff UI, and context
builder can waste tokens just seeing the mess.

## Immediate Cleanup Plan

Observed on 2026-05-14 before cleanup:

- `external-public-cache`: 41 files, 1.6 GB, 22,970,488 lines.
- all untracked files before ignore fix: 237 files, 23,011,288 lines.
- after adding `.gitignore` for `external-public-cache`: 197 untracked files,
  41,056 visible untracked lines.
- tracked diff after the incident: 93 files, +15,066 / -1,160.

1. Move raw benchmark caches out of the repo.
   - Bad: `docs/verification/25-5-memory-os-runs/external-public-cache/`
   - Good: `~/.cache/memd/external-public/` or `/private/tmp/memd-public-cache/`

2. Keep only small proof summaries in git.
   - Keep dated `.json`, `.md`, and `.ndjson` reports when they are compact.
   - Do not commit public datasets, cloned benchmark repos, or debug corpus
     mirrors.

3. Add hard ignore rules.
   - `.gitignore` must ignore
     `docs/verification/25-5-memory-os-runs/external-public-cache/`.
   - Verification scripts must default cache dirs outside the repo.

4. Add a repo hygiene guard.
   - Fail if untracked files exceed a sane count.
   - Fail if untracked line count exceeds a small threshold.
   - Fail if any file under `docs/verification/**/cache/` is visible to git.

5. Update docs before any more implementation.
   - Current plan must list blockers, not just target architecture.
   - Feature registry must separate implementation, dogfood, proof, market,
     and hygiene state.

## Current Honest State

### Implementation Features

`memd features --json --output .memd` currently reports these as `working`:

- hybrid local bundle
- server authority
- mandatory retrieval core
- semantic lane
- optional RAG booster
- model-tier context compiler
- capability sync
- access/secret routes
- shared context mesh
- prompt firewall
- knowledge-gap guard
- harness context guardrails
- token savings engine
- proof gates

This means focused implementation tests exist. It does not mean memd is
market-best.

### Blocked Or Unproven Gates

- Supermemory replay: blocked by missing credential/replay proof.
- Full external public proof: blocked unless explicitly enabled.
- Repo hygiene: failed due raw benchmark cache bloat.
- Token-efficiency dogfood: failed until cache bloat is removed and guarded.
- Cross-PC real sync: not proven as a long-running user/device evidence clock.
- Capability sync across all actual PCs/harnesses: partially proven, not yet
  audited against every local install and plugin cache.
- Agent-secrets integration: route-aware, not merged; still needs broker proof.
- Docs: stale until this recovery plan replaces the old market plan.

## Architecture Direction

The target stays the same, but order changes.

1. Local survival layer.
   `.memd/` boots context, last-good wake, local queues, capability inventory,
   access-route hints, and token/source ledgers.

2. Server authority.
   `memd-server` owns shared typed memory, corrections, capability sync,
   access-route refs, hive board, and reconciliation.

3. Retrieval kernel.
   Exact, FTS/BM25, fuzzy, entity/atlas, temporal truth, correction graph,
   provenance, local dense semantic lane, optional RAG booster.

4. Context compiler.
   Every harness receives a model-tier packet before reasoning. Tiny/local
   models get compact next-action packets, not raw dumps.

5. Firewall.
   Retrieved memory is data. It cannot become system policy, tool permission,
   sync setting, or canonical truth without trusted promotion.

6. Token-savings engine.
   Read once, hash source, store source ID, reuse durable memory. Avoid
   rereading unchanged raw files. Measure saved tokens and wasted tokens.

7. Proof.
   Small focused tests during implementation. Public/competitor benchmarks only
   when readiness and repo hygiene are green.

## Feature Registry Upgrade

`memd features` should stop using one broad status for everything. Each feature
needs separate fields:

- `implementation_status`: working, partial, broken, unproven
- `dogfood_status`: working, partial, broken, unproven
- `proof_status`: focused, sampled, full, blocked
- `market_status`: blocked, local-best-candidate, externally-proven
- `hygiene_status`: clean, noisy, broken
- `token_risk`: low, medium, high

Example:

```json
{
  "id": "token_savings_engine",
  "implementation_status": "working",
  "dogfood_status": "partial",
  "proof_status": "focused",
  "market_status": "blocked",
  "hygiene_status": "broken",
  "token_risk": "high"
}
```

## Verification Discipline

Allowed during implementation:

- narrow unit tests for changed files
- focused CLI smoke
- `cargo build -q -p memd-client --bin memd` when needed
- syntax checks for edited scripts
- `memd features --json --output .memd`
- repo hygiene checks

Not allowed during incremental implementation:

- full public benchmark suites
- full external competitor runs
- scripts that clone or cache large datasets into the repo
- proof scripts that generate large raw artifacts under `docs/`

## Next Engineering Order

### P0: Clean And Guard

- Move/delete `external-public-cache` from repo working tree.
- Keep `.gitignore` rule for the cache path.
- Decide which remaining untracked proof artifacts are real repo evidence and
  which are generated noise.
- Add a hygiene verifier that reports untracked file count, untracked line
  count, largest untracked files, and cache-directory violations.
- Update verification scripts to default cache dirs outside the repo.

Gate: Codex UI no longer shows multi-million-line unstaged additions.

### P1: Honest Feature Registry

- Add multi-axis feature status.
- Add repo hygiene and token-risk status to `memd health` and `memd features`.
- Mark market claim blocked when hygiene is broken, even if implementation
  tests pass.

Gate: `memd features` cannot say clean/market-ready while raw caches are
visible to git.

### P2: Capability And Access Dogfood

- Audit actual Codex skills, plugin cache, CLIs, MCPs, harness packs, Ollama
  models, Bitwarden route status, and agent-secrets broker availability.
- Sync them through server authority.
- Render `Active Capabilities` and `Access Routes` in context packets.

Gate: a fresh harness can ask memd what tools exist, what is missing, and how
to ask the user for access without storing secrets.

### P3: Context Token Budget Enforcement

- Add packet budget tests per model tier.
- Add wasted-token telemetry for raw source rereads, giant diffs, and cache
  exposure.
- Add source-ID reuse checks.

Gate: tiny packet stays under 1,000 tokens and still includes task, correction,
procedure, capability, access route, and source IDs.

### P4: Retrieval Quality Work

- Improve no-RAG recall first.
- Keep RAG/semantic as booster, not truth owner.
- Expand focused qrels only after hygiene is clean.

Gate: typo/path/name/command/correction recall improves without visibility
regression.

### P5: Public Proof

- Run sampled external proof only after P0-P4 are green.
- Run full public proof only when explicitly allowed.
- Run Supermemory replay only when credential/access route is available.

Gate: market claim stays blocked until competitor and full public proof pass.

## Documentation Rules Going Forward

- Plans must start with current blockers.
- Proof docs must distinguish focused, sampled, full, and blocked.
- Raw benchmark data belongs outside the repo.
- Docs should link compact artifacts, not embed huge raw data.
- Any "working" claim must say what evidence actually covers.

## Definition Of Clean

memd can resume 25/5 implementation only when:

- untracked cache bloat is gone
- `.gitignore` blocks known cache paths
- scripts write heavy caches outside repo by default
- feature registry exposes hygiene and token-risk status
- docs no longer imply market-best proof exists
- narrow tests for changed code pass
