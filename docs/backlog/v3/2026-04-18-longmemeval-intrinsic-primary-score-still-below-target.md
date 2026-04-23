---
status: open
severity: high
phase: B3
opened: 2026-04-18
scope: retrieval-quality
---
# LongMemEval Intrinsic Primary Score Still Below Target

- status: `open`
- severity: `high`
- phase: `B3`
- opened: `2026-04-18`
- scope: `retrieval-quality`

## Problem

The product-path LongMemEval primary gate now runs to completion in a
reasonable window, but the score is still not good enough.

Measured on 2026-04-18 with the memd-backed 500-question primary-gate
run (`--retrieval-backend memd`, `--mode raw`, `--top-k 20`,
`turn_diagnostics=false`):

- `session_recall_any@5 = 0.828`
- `session_recall_any@10 = 0.882`
- `session_recall_any@30 = 0.978`
- `session_recall_any@50 = 0.998`
- duration: `1468764 ms` (~24.5 min)

The B3 target is `LongMemEval ≥ 0.92 intrinsic`, so close-out is still
red by `0.092`.

## Why this matters

- B3 cannot be closed honestly while the primary intrinsic target is
  still missed.
- The harness/runtime improvement removed the excuse that this was only a
  tooling problem; what remains is product retrieval quality.
- Sidecar acceleration cannot be allowed to hide an intrinsic miss,
  because B3's contract is "great without RAG."

## Fix

- Analyze the 500-question miss set by question type and failure shape on
  the memd-backed product path.
- Compare product-path candidate generation/ranking against the old
  client-side lexical baseline to find where the server path is losing
  signal.
- Improve retrieval quality before spending more time on optics or
  close-out docs.
- After intrinsic moves materially, rerun the 500-question gate again and
  only then record accelerated-mode deltas.

## Acceptance

- Full 500-question intrinsic LongMemEval primary-gate run reaches
  `session_recall_any@5 ≥ 0.92`.
- Phase and roadmap docs can point at a green intrinsic rerun without
  caveats about the primary metric being red.
- Any accelerated/sidecar result remains secondary evidence rather than a
  substitute for the intrinsic pass.
