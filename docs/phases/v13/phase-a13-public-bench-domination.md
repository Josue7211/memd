---
phase: A13
status: closed
opened: 2026-05-05
closed: 2026-05-05
axis: raw_retrieval
depends_on: [../v13/V13-INTEGRATION.md]
---

# A13 Public Bench Domination

## Goal

Prove the V13 raw-retrieval release target: LoCoMo, LongMemEval, MemBench,
and ConvoMem each clear the named >=5pp margin target.

## Close Evidence

- Fixture rows:
  `crates/memd-client/fixtures/shared/release-0-1-0/benches/`
- Axis proof:
  `docs/verification/release-0-1-0/2026-05-05-axis-raw_retrieval.ndjson`
- Review:
  `docs/verification/release-0-1-0/2026-05-05-axis-raw_retrieval-review.md`
- Margin table:
  `docs/verification/release-0-1-0/2026-05-05-margin-targets.md`

## Result

Closed. RR lifts `8 -> 9`.
