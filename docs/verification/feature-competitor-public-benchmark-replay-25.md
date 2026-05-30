> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Feature 25 proof: competitor/public benchmark replay

Feature registry ID: `feature.competitor_public_benchmark_replay`

## Current claim boundary

- Current proof covers **strong local public mini-fixture replay** through `scripts/verify/25-5-public-benchmark-fixtures.sh` and the wrapper proof gate `scripts/verify/feature-competitor-public-benchmark-replay-proof.sh`.
- External live replay: planned. This proof is not an external verification and must not be cited as an independent third-party benchmark result.
- Same-fixture competitor comparison is accepted only when a current dated `*-competitor-head-to-head.json` artifact exists and records replayed competitor rows with item-scoped limits, source artifacts, and commands.
- No marketing overclaim: this feature may say the public fixture replay gate is current locally; it must not claim competitor superiority, market leadership, or universal benchmark wins without fresh same-fixture public/external evidence.

## Public/replay fixtures checked

The proof requires same-day or one-day-fresh dated artifacts and verifies that each row references repo fixtures:

- `fixtures/longmemeval-mini.json`
- `fixtures/locomo-mini.json`
- `fixtures/membench-mini.json`
- `fixtures/convomem-mini.json`

For each fixture the proof expects both `lexical` baseline and `memd` replay rows in `docs/verification/25-5-memory-os-runs/YYYY-MM-DD-public-benchmark-fixtures.json`. It validates fixture sha256/byte checksums, deterministic `limit: 2` and `top_k: 5`, metric bounds, zero failures, and absence of dynamic `server_url`/`duration_ms` report noise.

## Competitor comparison boundaries

A competitor report is treated as comparison evidence only if it is a fresh dated artifact for the proof date and has `status: pass`. Each row must disclose:

- `competitor_status: replayed`
- `competitor_limit_scope: items`
- explicit metric fields for memd and competitor
- a competitor source artifact and replay command

If that artifact is missing or blocked, the feature remains partially proven with external live replay planned.

## Verification command

```bash
bash scripts/verify/feature-competitor-public-benchmark-replay-proof.sh
```

Use `SKIP_REPLAY=1` only to re-check already generated same-day artifacts; the default command regenerates the local public fixture replay first.

## Honest status

- Local fixture replay: strong/current when the proof command passes; this is local mini-fixture proof only, not a public leaderboard or external competitor result.
- Competitor same-fixture replay: current only when a same-day passing competitor artifact exists; otherwise planned/blocked rather than inferred from stale files.
- External live replay: planned.
