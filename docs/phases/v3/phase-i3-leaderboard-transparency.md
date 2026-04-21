---
phase: I3
name: Leaderboard Transparency
version: v3
status: complete
opened: 2026-04-21
closed: 2026-04-21
depends_on: [G3, H3]
backlog_items:
  - "2026-04-14-no-public-benchmark-parity"
---

# Phase I3: Leaderboard Transparency

## Goal

Every row in `docs/verification/PUBLIC_LEADERBOARD.md` carries a full method card — enough for a stranger (or auditor, or competitor) to reproduce the number or flag it as suspect. Competitor numbers are disclosed with the same discipline. Phantom historical scores are retracted.

## Why this phase exists

Industry research (2026-04-21) surfaced two problems: (a) MemPalace's 96.6% LongMemEval is contested in their own issue tracker because the benchmark wraps ChromaDB rather than exercising their library code; (b) Zep's 84% LoCoMo corrected to 58.44% when the harness was audited. Numbers >0.90 in this space are plausibly gaming unless the method is fully disclosed. memd's own leaderboard previously asserted `LoCoMo > 0.80` and `MemBench 0.993` — neither is reproducible from the repo head as of 2026-04-21. They must be retracted.

## Deliver

1. **Per-row method card.** Every leaderboard row includes:
   - Bench name + dataset split + dataset fixture SHA
   - Canonical metric name + formula reference (link to H3 doc or upstream paper section)
   - Backend (lexical / memd / sidecar / rrf) per G3
   - Judge model + version (if LLM-judged)
   - Git commit SHA that produced the number
   - Reproduction command (exact `cargo run -- …` invocation with env vars)
   - `verification: verified | replay-pending | recorded-unpinned | retracted`
   - Judge cost ledger (tokens, USD) when applicable
2. **Retraction log.** New top-level section `## Retracted Scores` listing every historical claim that does not reproduce at head, with: date claimed, code path, why retracted, replacement status. At minimum:
   - LoCoMo `>0.80` (user-confirmed 2026-04-21, code path: lexical via `build_context_retrieval_run_report`, head reproduces 0.4149 lexical)
   - MemBench `0.993` (release board, code path: unknown, head reproduces 0.3463 lexical)
   - LongMemEval `0.966 / 0.936` retained but re-stamped as retrieval-diagnostic (not canonical qa_accuracy) until H3 judge numbers ship
3. **Gaming-audit rule.** Any score ≥0.90 on a benchmark carries a mandatory `audit:` field: who audited, what was checked (train/test overlap, top_k vs corpus size, judge prompt lift, dataset split), when. Scores without audit trail are capped at `verification: recorded-unpinned` regardless of value.
4. **Competitor column discipline.** MemPalace / mem0 / supermemory / Letta competitor numbers include the same method card fields (or `unknown` where the competitor didn't disclose). MemPalace 96.6% rendered as `96.6% ⚠ contested` with link to their issue tracker.
5. **Leaderboard generator.** `scripts/regen-leaderboard.sh` reads `.memd/benchmarks/history/benchmark-runs.jsonl`, emits Markdown with the full method card. CI gate on V3 PRs: if retrieval code touched but leaderboard not regenerated, PR fails.
6. **Backlog item closure.** `docs/backlog/v3/2026-04-14-no-public-benchmark-parity.md` gets `resolved` stamp when this phase passes.

## Pass Gate

- pre: Leaderboard rows carry commit + fixture but no judge model, no backend, no reproduction command, no retraction log. Phantom numbers still live in ROADMAP prose.
- post: Every row has the 8 required method-card fields. Retraction log lists ≥2 phantom scores. ROADMAP prose references the retraction log instead of asserting the retracted numbers. CI gate blocks PRs that touch retrieval without leaderboard regen.
- regression budget: method-card addition is net-new text; no numeric change expected. If a row was claiming something it cannot reproduce, it moves to Retracted, not to a pretend-verified score.
- evidence: diff of `PUBLIC_LEADERBOARD.md` showing new columns; CI run failing on a test PR that edits `retrieval.rs` without regen; CI run passing after regen.

Non-bench gates:

- `bash scripts/regen-leaderboard.sh` emits valid Markdown, no broken links
- Retraction log rendered, not hidden in a collapsible
- ROADMAP.md `## Status Snapshot` updated to reference retraction log, not assert retracted numbers

## Evidence

- Leaderboard diff (before/after method cards)
- CI log showing the gate firing + passing
- Retraction log as its own reviewable artifact
- Stranger-test: an outsider picks one row, runs the reproduction command, gets the same number within ±0.01

## Product Win

A reader — stranger, employer, funder, contributor — can audit any memd claim in under 5 minutes. memd visibly refuses the "leaderboard theater" mode the AI memory space is already playing. That refusal is a product signal: if memd retracts its own phantom scores, its live scores are load-bearing.

## Fail Conditions

- Retraction log gets soft-pedaled ("these scores were from an older code path") → retract harder; the point is to be unmistakable
- Method card becomes optional or partial → CI gate must make it mandatory or the discipline rots
- CI gate fires on every PR regardless of scope → scope narrow; retrieval/bench-touching PRs only
- Competitor column treats MemPalace 96.6% as uncontested → add the ⚠ or omit the number

## Donor Anchors

- **I3-D1**: mempalace method-card rendering (if they have one) — audit before writing memd's
- **I3-D2**: verification tiers (`verified | replay-pending | recorded-unpinned`) from F3-original — extend with `retracted`
- **I3-D3**: `docs/verification/PUBLIC_BENCHMARKS.md` protocol — update for canonical metrics

## Rollback

Method-card generation is Markdown emission; revert is file-level. CI gate is config. Retraction log is text — once posted, intentionally hard to unpost (that's the point).

## Out of scope

- Actual V3 floor verification run (J3)
- Canonical metric definitions themselves (H3)
- Adapter parity (G3)
- MQI weight research (file backlog in H3)
