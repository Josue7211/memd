---
phase: E13
status: closed
opened: 2026-05-05
closed: 2026-05-05
axis: correction_retention
depends_on: [../v13/V13-INTEGRATION.md]
---

# E13 Multi Hop Corrections

## Goal

Apply correction X to downstream derived items Y and Z, then prove next-session
behavior uses corrected downstream truth.

## Close Evidence

- Fixture:
  `crates/memd-client/fixtures/shared/release-0-1-0/corrections/multihop-x-to-y-z.jsonl`
- Core primitive: `memd_core::v13::apply_multi_hop_correction`
- Test: `v13::tests::multi_hop_correction_updates_downstream_next_session`
- Axis proof:
  `docs/verification/release-0-1-0/2026-05-05-axis-correction_retention.ndjson`

## Result

Closed. CR lifts `7 -> 8`.
