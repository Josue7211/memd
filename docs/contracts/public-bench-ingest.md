# Public-Bench Ingest Contract (V6 / A6)

This document is the V6 typed-ingest contract for the four public benches
(LongMemEval, LoCoMo, MemBench, ConvoMem). It pairs with
`docs/contracts/type-taxonomy.md` (F5 12-kind boundary) and is the
authoritative reference for what `--typed-ingest=episodic` produces.

## 1. Schema policy

`MemoryKind::Episodic` is **not** a kind on `memd-schema`. The 12-kind F5
taxonomy stays untouched. Episodic is an adapter-layer concept:

- Each bench turn ingests as `MemoryKind::Fact`.
- The `EpisodicProvenance` sidecar rides on the record's metadata.
- Downstream retrieval / promotion (B6 / C6) reads the sidecar; the F5
  router does not.

## 2. `EpisodicTurn` shape

```rust
pub(crate) struct EpisodicTurn {
    pub content: String,
    pub provenance: EpisodicProvenance,
}

pub(crate) struct EpisodicProvenance {
    pub bench_id: String,       // longmemeval | locomo | membench | convomem
    pub session_id: String,     // bench-specific, must be non-empty
    pub turn_index: u32,
    pub speaker: String,        // user | assistant | <bench role>
    pub source_hash: String,    // sha256 hex of raw turn bytes
    pub captured_at: String,    // ISO-ish; empty when bench ships no date
}
```

## 3. Per-bench mapping

| Bench | session_id | speaker | captured_at |
| --- | --- | --- | --- |
| LongMemEval | `haystack_session_ids[i]` | turn `role` | `haystack_dates[i]` (fall back to `question_date`) |
| LoCoMo | `<sample_id>::session_<n>` | turn `speaker` | `session_<n>_date_time` |
| MemBench | `<category>::<tid>::list_<i>` | `user` / `assistant` | turn `time` |
| ConvoMem | `<item_id>::<conversation_id>` | message `speaker` | `""` (bench ships no per-message date) |

## 4. Determinism + baseline lock

Episodic counts are deterministic for a given fixture. The
`tests/fixtures/typed_ingest/a6/lme-sample-10turn.json` baseline is
locked in `crates/memd-client/src/benchmark/typed_ingest/ingest_card.rs`
as `BASELINE_LME_10TURN { turn_count: 10, session_count: 2 }`. Drift on
the comparator means the LME adapter changed shape — bump the baseline
deliberately in the same PR or fix the regression.

The plan's "no regression vs flat-RAG by >1%" applies to downstream
retrieval scores after A6.9 graduates the runtime wire — not to ingest
counts.

## 5. CLI surface

```
memd benchmark public <dataset> --typed-ingest=episodic
```

Accepted dataset ids: `longmemeval`, `locomo`, `membench`, `convomem`.
Flag is recognised today (A6.7). Runtime activation is gated by env
`MEMD_V6_TYPED_INGEST=1` and graduates in A6.9 once V5 closes
(2026-05-02 calendar gate).

## 6. Source of truth

- Adapters: `crates/memd-client/src/benchmark/typed_ingest/bench_loaders/`
- Trait + provenance: `crates/memd-client/src/benchmark/typed_ingest/episodic.rs`
- Dispatcher + report: `crates/memd-client/src/benchmark/typed_ingest/mod.rs`
- Card + baseline: `crates/memd-client/src/benchmark/typed_ingest/ingest_card.rs`
- Tests: `crates/memd-client/src/main_tests/typed_ingest_a6_tests/`
