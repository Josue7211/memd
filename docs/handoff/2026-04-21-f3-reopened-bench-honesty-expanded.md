---
date: 2026-04-21
phase: F3
part: reopen-and-expand
status: checkpoint
next_phase: G3
next_part: bench-adapter-parity
branch: research/mining
base_head: 564cea3
tests: cargo test -p memd-server green at b31bcbf (RAG fallback contracts pinned); roadmap-only commit at head, no code changes
---
# F3 reopened — V3 bench honesty expanded to G3/H3/I3/J3

## TL;DR

F3 Bench Honesty reopened 2026-04-21 after the paired E3+F3 bench run exposed two structural gaps:

1. **Adapter parity gap.** `build_context_retrieval_run_report` at `crates/memd-client/src/benchmark/public_benchmark.rs:1373` is pure token-intersection lexical ranking. LoCoMo, MemBench, and ConvoMem all route through it. Only LongMemEval has a `LongMemEvalRetrievalBackend` enum (`:1713`, Lexical/Sidecar/Rrf/Memd). Consequence: every B3/C3/D3 retrieval change is invisible in 3 of 4 published numbers. The 2026-04-21 `make bench-public` run produced LoCoMo 0.4149 + MemBench 0.3463 — matching M0 baselines to 3 sig figs because the path is still lexical.

2. **Non-canonical metric gap.** Research 2026-04-21 on true industry-standard AI memory benchmarks (what Mem0, Supermemory, Letta, MemPalace publish against):

   | Bench | Canonical metric | Competitor anchors |
   | --- | --- | --- |
   | LongMemEval | GPT-4o (`gpt-4o-2024-08-06`) binary QA accuracy | Mem0 93.4%, Supermemory 81.6%/84.6%, MemPalace 96.6% (disputed) |
   | LoCoMo | Token F1 on answer span | Mem0 91.6%, MemMachine 91.69%, Letta 74.0% |
   | MemBench | MQI composite (weights undisclosed); fall back to MC accuracy | none published |
   | ConvoMem | Accuracy over first 150 conversations | none published |

   memd publishes `session_recall_any@5`, `evidence_hit_rate@5`, `target_hit_rate@5` — retrieval diagnostics, not the canonical metrics. Zep's 84%→58% LoCoMo correction and MemPalace's contested 96.6% show scores ≥0.90 in this space are plausibly gaming unless audit trail accompanies.

**Phantom retractions.** Prior "verified release board" LoCoMo `>0.80` (user-confirmed 2026-04-21) and MemBench `0.993` do not reproduce at head on either lexical or the (still-hardwired) memd path. Retracted from `ROADMAP.md` Status Snapshot; replaced by explicit retraction reference pending I3's formal retraction log.

**F3 reopened, split into four follow-ups:**

| Phase | Job |
|---|---|
| G3 Bench Adapter Parity | generic `PublicBenchmarkBackend` enum; all 4 benches route `--backend {lexical,memd,sidecar,rrf}` |
| H3 Canonical Metrics | GPT-4o judge LME, token-F1 LoCoMo, MC accuracy MemBench, accuracy ConvoMem; reproduction audit ±0.03 vs upstream naive baseline |
| I3 Leaderboard Transparency | per-row method card (backend/metric/judge/fixture/commit/repro cmd), retraction log, gaming-audit rule for ≥0.90 |
| J3 V3 Floor Verification | paired intrinsic (sidecar-OFF) / accelerated (sidecar-ON) run on canonical metrics; stranger-reproducible; one run → one verdict |

V3 completion gate rebound to J3's canonical-metric run — proxy metrics cannot satisfy ≥0.70 floor.

## What changed in this handoff window

Roadmap-only commit. No retrieval code touched.

Commits landed on `research/mining`:
- `b313d10` — test(rag_bridge): pin timeout env, degraded-read, and ingest retry contracts (advisor follow-up from prior session)
- `b31bcbf` — docs(roadmap): bench pair partner remains F3, not blocker-resolution (advisor correction)
- `d83a48e` — docs(roadmap): LoCoMo >0.80 per user correction 2026-04-21
- `d69e947` — memd auto-commit: E3 HANDOFF READY — research/mining branch (D1+D2+D3+D4+D5 atomic shipped)
- `8f390ee` — docs(roadmap): E3 code-complete, bench deferred per every-2-phases cadence
- **`564cea3`** — **docs(roadmap): V3 bench honesty expanded — F3 reopened, G3/H3/I3/J3 added** ← this handoff's head

Files touched at `564cea3`:
- `ROADMAP.md` — Status Snapshot rewrite (phantom retraction, F3 reopen), ROADMAP_STATE block (`current_phase=F3`, `phase_status=reopened_2026-04-21`, active_blockers populated), V3 table rows G3/H3/I3/J3 added, V3 completion gate rebound to canonical metrics
- `docs/phases/v3/phase-f3-bench-honesty.md` — status `complete` → `reopened`, reopen note with research summary + competitor anchor table, split_into frontmatter pointing at G3-J3
- `docs/phases/v3/phase-g3-bench-adapter-parity.md` — new
- `docs/phases/v3/phase-h3-canonical-metrics.md` — new
- `docs/phases/v3/phase-i3-leaderboard-transparency.md` — new
- `docs/phases/v3/phase-j3-floor-verification.md` — new

## Bench state at handoff (retrieval-diagnostic, NOT canonical)

From `.memd/benchmarks/history/benchmark-runs.jsonl` (git SHA `b31bcbfd…`):

| benchmark | primary metric | value | item count |
| --- | --- | --- | --- |
| longmemeval | session_recall_any@5 | 0.882 | 500 |
| locomo | evidence_hit_rate@5 | 0.4149 | 1986 |
| convomem | accuracy | 0.9028 | 525 |
| membench | target_hit_rate@5 | 0.3463 | 3000 |

**Do not quote these as V3 numbers.** LongMemEval + ConvoMem are retrieval-diagnostic on a lexical path; LoCoMo + MemBench are M0-baseline-equivalent because the bench path doesn't reach memd retrieval. Canonical numbers land in J3.

## What's next (G3)

Generic `PublicBenchmarkBackend` enum replacing `LongMemEvalRetrievalBackend`:

```rust
enum PublicBenchmarkBackend {
    Lexical,
    Memd { base_url: String },
    Sidecar { base_url: String },
    Rrf,
}
```

Refactor `build_context_retrieval_run_report` (`crates/memd-client/src/benchmark/public_benchmark.rs:1373`) to dispatch by backend instead of hard-coding token intersection. Add `rank_locomo_corpus_via_memd`, `rank_membench_corpus_via_memd`, `rank_convomem_corpus_via_memd` as siblings of the existing `rank_longmemeval_corpus_via_memd` (`:2150`) template. Add `--backend` CLI flag to `benchmark public`. Parity test per bench: memd-backend ordering ≠ lexical ordering on a pinned fixture query.

Pass gate = lexical regression ±0.001 (no-op guarantee) + memd-backend ordering diff recorded.

## Risks / open questions

- **MQI weights** for MemBench not disclosed in public sources; H3 plans MC accuracy fallback + backlog item for weight resolution via upstream contact.
- **Judge cost** budgets TBD — GPT-4o on 500-item LongMemEval + 1986-item LoCoMo may push tens of USD per run. H3 pins `MEMD_BENCH_JUDGE_BUDGET_USD` env cap.
- **Floor might miss.** With canonical metrics and real memd retrieval, intrinsic sidecar-OFF numbers are unknown. V3 ships the honest result per J3; miss = ship-with-recovery-plan, not silent rerun.
- **Dogfood surfaces** unchanged — V3 completion gate's product dimension independent of this work.

## Pointers for next session

- Roadmap: `ROADMAP.md` Status Snapshot + V3 table + V3 completion gate
- Phase docs: `docs/phases/v3/phase-g3-…md` through `phase-j3-…md`
- Code entry points:
  - `crates/memd-client/src/benchmark/public_benchmark.rs:1373` (build_context_retrieval_run_report — to refactor)
  - `crates/memd-client/src/benchmark/public_benchmark.rs:1713` (LongMemEvalRetrievalBackend — to generalize)
  - `crates/memd-client/src/benchmark/public_benchmark.rs:2150` (rank_longmemeval_corpus_via_memd — sibling template)
  - `crates/memd-client/src/benchmark/public_benchmark.rs:1756` (rank_longmemeval_corpus — dispatcher pattern)
- Research notes: industry metric matrix inline in `phase-h3-canonical-metrics.md`
- Handoff index: `docs/handoff/INDEX.md`

## Memd continuity

- Checkpoint: `bedea20d-3727-4ea8-89de-e27cd96d7446` (F3-reopened status, 2026-04-21)
- Roadmap state: `current_phase=F3`, `phase_status=reopened_2026-04-21`, active_blockers: [bench-adapter-parity-gap, non-canonical-metric-gap, phantom-locomo-membench-scores]

Branch `research/mining` is clean at `564cea3` and ready for G3 implementation.
