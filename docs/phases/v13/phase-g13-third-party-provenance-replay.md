---
phase: G13
status: closed
opened: 2026-05-05
closed: 2026-05-05
axis: trust_provenance
depends_on: [../v13/V13-INTEGRATION.md]
---

# G13 Third Party Provenance Replay

## Goal

Export a release snapshot that an independent replay harness can verify without
access to the memd runtime.

## Close Evidence

- Export fixture:
  `crates/memd-client/fixtures/shared/release-0-1-0/export/full-session-9-export.json`
- Replay fixture:
  `crates/memd-client/fixtures/shared/release-0-1-0/replay/third-party-harness.py`
- Core primitive: `memd_core::v13::third_party_replay_export`
- Axis proof:
  `docs/verification/release-0-1-0/2026-05-05-axis-trust_provenance.ndjson`

## Result

Closed. TP lifts `8 -> 9`.
