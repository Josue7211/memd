# Semantic Distillation Contract (V6 / B6)

This document is the V6 distillation contract: episodic turns →
semantic candidates (`Fact` / `Decision` / `Preference`) with
provenance. Pairs with `docs/contracts/public-bench-ingest.md` (A6
ingest) and `docs/contracts/type-taxonomy.md` (F5 12-kind boundary).

## 1. Pipeline position

```
A6 episodic turn  ──► B6 distiller ──► dedupe ──► candidate store (stage=candidate)
                       (codex-lb)       (hash+cos)   (sidecar provenance)
                                                            │
                                                            ▼
                                                      C6 promotion → canonical
```

B6 emits **candidates only**. Promotion to canonical is C6's job.

## 2. Prompt card

Identifier: `semantic-distillation/v1`.

Inputs: a single `EpisodicTurn` (content + provenance). The system
prompt frames the model as a **fact extractor**, not a chatbot. It must
emit ONLY a JSON object matching §3 — no prose, no markdown fences.

System prompt (frozen as `PROMPT_CARD_V1` constant):

```
You extract durable facts, decisions, and preferences from a single
conversation turn. Return ONLY a JSON object {"candidates": [...]}.
Each candidate has fields: kind ("Fact"|"Decision"|"Preference"),
content (one self-contained sentence), confidence (0.0-1.0),
source_turn_ids (array of strings), rationale (≤120 chars).

Rules:
- Skip filler (greetings, acks, "ok", "thanks", chit-chat). Emit zero candidates.
- Speaker = user → preferences and facts about the user.
- Speaker = assistant → only emit if the assistant states a durable
  decision or fact the user agreed to.
- One candidate = one self-contained claim. Split compound claims.
- confidence < 0.5 means "skip" — drop the candidate entirely.
- source_turn_ids MUST contain the provenance.session_id::turn_index pair.
```

Prompt-version is part of the cache key — bumping the card invalidates
all cached extractions.

## 3. Output schema

```json
{
  "candidates": [
    {
      "kind": "Fact|Decision|Preference",
      "content": "string",
      "confidence": 0.0,
      "source_turn_ids": ["session_id::turn_index"],
      "rationale": "string (≤120 chars)"
    }
  ]
}
```

Validator rejects: unknown `kind`, `confidence` outside [0, 1], empty
`content`, missing `source_turn_ids`, extra top-level keys.

## 4. Cache contract

Cache key: `sha256(prompt_version || turn.provenance.source_hash)`.

Cache directory: `.memd/benchmarks/public/cache/distill/`.

Cache file: one JSON object per turn — `{key, model, milli_usd, candidates, ts}`.
Read-on-miss → call → write-back. Cache hits never charge budget.

Disable with `MEMD_V6_DISTILL_CACHE=0` for forced re-extraction.

## 5. Dedupe

Two-stage:

1. **Hash**: `sha256(lowercase(content).trim())` — exact dedupe.
2. **Cosine**: vector similarity ≥ 0.85 collapses near-duplicates.
   Reuses memd-server's `cosine_on_unit` semantics. Embedding source
   pluggable; B6 ships hash-only by default and cosine when an embedder
   is wired (post-V6 e-channel).

Within a single bench run, the dedupe table is per-`session_id` — two
sessions can independently surface the same fact.

## 6. Candidate persistence

Candidates land as `MemoryRecord { stage: candidate, kind: Fact|Decision|Preference }`.
The `EpisodicProvenance` from the originating turn rides as a sidecar
inside record metadata, with the additional B6 fields:

- `distill.prompt_version`
- `distill.judge_model`
- `distill.confidence`
- `distill.rationale`
- `distill.source_turn_ids`

Candidates do NOT graduate to canonical without C6.

## 7. CLI surface

```
memd benchmark public <dataset> --typed-ingest=episodic+semantic \
  [--distill-model gpt-5.4] \
  [--distill-budget-milli-usd 100] \
  [--distill-cache-dir .memd/benchmarks/public/cache/distill/]
```

Accepted dataset ids: `longmemeval`, `locomo`, `membench`, `convomem`.

Flag is recognised today (B6.5). Runtime activation is gated by
`MEMD_V6_TYPED_INGEST=1` (shared with A6.9) and the V5 calendar gate
(2026-05-02). Until then the flag is scaffold-symmetric: parsed,
counted, telemetry emitted, but no live ingest.

## 8. Telemetry

Per-turn NDJSON at `.memd/benchmarks/public/results/distill-<date>.ndjson`,
emitted by `append_distill_telemetry` and locked by test
`distill_telemetry_line_appends_ndjson`:

```json
{"ts":"2026-04-27T00:00:00Z","bench_id":"longmemeval","turn_id":"sess_b6::0",
 "judge_model":"gpt-5.4","prompt_tokens":410,"completion_tokens":48,
 "milli_usd":2,"candidate_count":1,"cache":"miss"}
```

`cache` is `"hit"` or `"miss"`. Aggregator picks this up alongside the
A6 ingest card; B6.7 ships the format, runtime emission graduates with
A6.9 once `MEMD_V6_TYPED_INGEST=1` is active in nightly.

## 9. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_TYPED_INGEST` | unset | `=1` graduates routing from scaffold to live (shared with A6.9). |
| `MEMD_V6_DISTILL_MODEL` | `gpt-5.4` | Override judge model per run. |
| `MEMD_V6_DISTILL_CACHE` | `1` | `=0` disables cache (forced re-extraction). |

## 10. Source of truth

- Prompt card constant: `crates/memd-client/src/benchmark/typed_ingest/distiller.rs`
- Schema validator: `crates/memd-client/src/benchmark/typed_ingest/distiller.rs`
- Cache layer: same module
- Dedupe: `crates/memd-client/src/benchmark/typed_ingest/dedupe.rs`
- Candidate store: `crates/memd-client/src/benchmark/typed_ingest/candidate_store.rs`
- Tests: `crates/memd-client/src/main_tests/typed_ingest_b6_tests/`
