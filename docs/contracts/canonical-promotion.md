# Canonical Promotion Contract (V6 / C6)

This document is the V6 canonical-promotion contract: B6 candidates
(`stage=candidate`) → canonical records (`stage=canonical`) under a
deterministic rule card. Pairs with
`docs/contracts/semantic-distillation.md` (B6) and
`docs/contracts/type-taxonomy.md` (F5 12-kind boundary).

## 1. Pipeline position

```
A6 episodic ──► B6 distiller ──► dedupe ──► candidate store
                                              │ (stage=candidate)
                                              ▼
                                       C6 promotion rule
                                              │
                          ┌───────────────────┼─────────────────┐
                          ▼                   ▼                 ▼
                       PROMOTE              SKIP            CONTRADICTS
                  canonical_index.jsonl  reason logged   v4-c4-correction
                    (stage=canonical)    (telemetry)        path reused
```

Promotion is gated by the same V5 calendar gate as B6 runtime
activation (2026-05-02). Until that gate clears, scaffold-symmetric
landings (rule engine + canonical index + tests + this card) ship
without live runtime wiring.

## 2. Rule card (frozen as `PromotionRule::v1()`)

```yaml
rule_version: canonical-promotion/v1
corroboration_count: 2          # distinct source turns referencing the same canonical identity
confidence_min: 0.8             # candidate confidence floor (per-candidate)
session_age_min_turns: 3        # don't promote within the same originating turn window
contradiction_check: v4-c4-correction-reuse
```

Rule version is committed in the rule card and surfaced in promotion
telemetry. Bumping the card invalidates all canonical promotions
recorded under the prior version (re-run promotion to refresh).

## 3. Canonical identity

Two candidates target the **same canonical record** iff:

- `kind` matches (Fact = Fact, Decision = Decision, Preference =
  Preference — no cross-kind merging), and
- `content_hash` matches (sha256 of normalised content: trim, lowercase,
  collapse internal whitespace).

Identity is content-addressed. The canonical index keys records by
`(kind, content_hash)` — both lookups (corroboration count, dedupe on
write) and contradiction checks use the same address.

## 4. Promotion algorithm

Inputs: an ordered list of `CandidateRecord` (B6 output) for one
bench-run.

```
group_by canonical_identity (kind, content_hash):
    if   any candidate confidence < confidence_min   → REJECT(low_confidence)
    elif distinct source_turn_ids < corroboration_count → REJECT(insufficient_corroboration)
    elif min(turn_index spread) < session_age_min_turns → REJECT(within_session_window)
    elif contradiction_check matches existing canonical → REJECT(contradicts_canonical)
    else                                              → PROMOTE
```

`session_age_min_turns` is measured as the difference between the
**maximum** and **minimum** `turn_index` across the corroborating
source turns of one canonical identity (within one `session_id`). For
cross-session corroboration, the constraint is satisfied automatically.

## 5. C4 contradiction reuse

The C4 correction path (`crates/memd-core/src/correction/`) provides
`assert_c4_correction_provenance` and the `corrections.ndjson` event
log. C6 reuses the **same comparison shape**: a candidate
`CONTRADICTS` an existing canonical record when

- `kind` matches, and
- `content_hash` differs but normalised content overlaps on the
  semantic key (rule v1: identical first 6 normalised tokens or
  cosine ≥ 0.85 with conflicting confidence directions).

Contradicting candidates are **not** promoted; instead, they are
appended to `corrections.ndjson` via the C4 path with
`source = "c6-promotion"` so the existing C4 dashboard surfaces them.

## 6. Canonical record shape

```json
{
  "stage": "canonical",
  "kind": "Fact",
  "content": "User has a shiba inu named Nori",
  "content_hash": "<sha256 of normalised content>",
  "provenance": {
    "source_turn": "sess_b6::4",
    "captured_by": "c6-promotion/v1",
    "captured_at": "2026-04-27T00:00:00Z",
    "chain": ["sess_b6::4", "sess_b6::18"]
  },
  "rule": {
    "version": "canonical-promotion/v1",
    "corroboration_count": 2,
    "min_confidence": 0.86
  },
  "candidates": [
    {"prompt_version": "semantic-distillation/v1", "judge_model": "gpt-5.4", "confidence": 0.86, "source_turn_ids": ["sess_b6::4"]},
    {"prompt_version": "semantic-distillation/v1", "judge_model": "gpt-5.4", "confidence": 0.91, "source_turn_ids": ["sess_b6::18"]}
  ]
}
```

The `provenance` block is shaped to pass `audit_record` (E5 auditor) —
`source_turn`, `captured_by`, `captured_at`, plus optional `chain`.
`captured_by` is the rule version (`c6-promotion/v1`).

## 7. Telemetry (NDJSON)

Per bench-run: `.memd/benchmarks/public/results/promotion-<date>.ndjson`.

```json
{"ts":"2026-04-27T00:00:00Z","bench_id":"longmemeval","outcome":"promote","kind":"Fact","content_hash":"...","corroboration_count":2,"min_confidence":0.86,"rule_version":"canonical-promotion/v1","source_turn_ids":["sess_b6::4","sess_b6::18"]}
{"ts":"2026-04-27T00:00:01Z","bench_id":"longmemeval","outcome":"reject","reason":"low_confidence","kind":"Preference","content_hash":"...","min_confidence":0.62,"rule_version":"canonical-promotion/v1"}
```

Outcomes: `promote` | `reject` | `contradicts`. Rejection reason is one
of `low_confidence`, `insufficient_corroboration`,
`within_session_window`, `contradicts_canonical`.

## 8. Dry-run mode

`--promotion-dry-run` (or `MEMD_V6_PROMOTION_DRY_RUN=1`) emits the same
telemetry NDJSON but **does not** write to `canonical_index.jsonl`.
Used for soak testing the rule before opening the lane.

## 9. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V6_PROMOTION_DRY_RUN` | `0` | Force dry-run globally (overrides CLI). |

Runtime activation of `--typed-ingest=episodic+semantic+canonical`
inherits the V5 calendar gate. Until 2026-05-02 + A6.9 graduation, the
flag is parsed and the runtime notice surfaces — actual promotion
runs as a no-op (gate string mirrors B6).

## 10. Versioning

Bumping `rule_version`:

- Invalidates every canonical record carrying a prior `rule.version`.
- Re-run promotion replays the candidate store under the new rule and
  emits a `migration` telemetry block alongside per-record outcomes.
- Prior canonical records are kept (with their `rule.version`) under
  `.memd/benchmarks/public/canonical/archive/<old-version>/` for
  rollback.

## 11. Bench impact (deferred to V5 calendar gate)

Targets locked under fixture-driven proxies until live runtime
graduates:

- LME `qa_accuracy` ≥ +0.02 vs B6-only baseline (cumulative ≥ +0.04).
- MemBench `recall_at_k` ≥ +0.03 vs B6-only baseline.

Real bench locks land alongside B6 runtime activation post-2026-05-02.
