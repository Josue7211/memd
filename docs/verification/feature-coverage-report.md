> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Feature Coverage Report

Source: `docs/verification/features.registry.json` (`version: 2026-05-29.pillar-01`).

This is a static Pillar 01 coverage report. It summarizes registry coverage and honesty blockers; it is not a product proof report.

## Summary

- Registered first-class feature areas: 14
- Areas blocking any honest 25/25 claim: 14
- Areas with no executable proof commands listed: 5
- Areas with no proof artifacts listed: 8
- Areas externally verified: 0
- Areas with sustained/continuous dogfood: 0

## Status by Feature

| ID | Current | Proof | Dogfood | External | Primary blocker |
| --- | --- | --- | --- | --- | --- |
| `feature.setup_install_onboarding` | `partial` | `smoke` | `ad_hoc` | `none` | Needs external replay and broader lifecycle proof beyond local setup smoke. |
| `feature.docs_product_education` | `partial` | `smoke` | `ad_hoc` | `none` | Docs need ongoing alignment with registry and executable proof. |
| `feature.doctor_status_recovery_update_uninstall` | `unknown` | `none` | `none` | `none` | Implementation/proof status is unknown; lifecycle safety not proven. |
| `feature.memory_core` | `partial` | `strong` | `ad_hoc` | `none` | Local proof maps capture/lookup/resume/corrections/provenance/trust, but external validation and production reliability remain unproven. |
| `feature.context_compiler_token_savings` | `partial` | `planned` | `ad_hoc` | `none` | Token savings need reproducible measurements and artifacts. |
| `feature.shared_research_cache` | `unknown` | `none` | `none` | `none` | Implementation/proof status is unknown; contamination controls unproven. |
| `feature.hive_hivemind_coordination` | `partial` | `stale` | `ad_hoc` | `none` | Existing hive proof surfaces are stale until re-run. |
| `feature.competitor_public_benchmark_replay` | `partial` | `stale` | `none` | `planned` | Competitor/public benchmark claims require current public replay artifacts. |
| `feature.dogfood_reliability_windows` | `partial` | `planned` | `ad_hoc` | `none` | No sustained dated reliability window is registered. |
| `feature.external_replay_auditor_proof` | `partial` | `planned` | `none` | `planned` | No independent external replay artifact is registered. |
| `feature.product_ux_dashboard_cli_language` | `unknown` | `none` | `none` | `none` | UX/dashboard status and walkthrough proof are unknown. |
| `feature.network_identity_federation_market` | `unknown` | `none` | `none` | `none` | Network/federation/market implementation and proof are unknown. |
| `feature.release_claim_honesty_gates` | `partial` | `smoke` | `ad_hoc` | `none` | Registry audit exists, but release gates need integration with release flow. |
| `feature.cross_harness_continuity` | `partial` | `planned` | `ad_hoc` | `none` | Cross-harness replay proof is planned but not recorded. |

## Honest Conclusion

Pillar 01 now provides a registry truth source and audit, but the registry itself confirms whole-app 25/25 is not achieved. Most areas are partial, unknown, planned, stale, or missing external/dogfood evidence.

## Refresh Procedure

1. Update `docs/verification/features.registry.json` first.
2. Keep `docs/verification/FEATURES.md` aligned with high-level registry status.
3. Refresh this report when feature status, proof artifacts, or blockers change.
4. Run `bash scripts/verify/feature-registry-audit.sh` and `git diff --check` before claiming registry truth is valid.
