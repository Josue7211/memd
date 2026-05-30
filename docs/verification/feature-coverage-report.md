> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Feature Coverage Report

Source: `docs/verification/features.registry.json` (`version: 2026-05-29.pillar-01`).

This is a static Pillar 01 coverage report. It summarizes registry coverage and honesty blockers; it is not a product proof report.

## Summary

- Registered first-class feature areas: 14
- Areas blocking any honest 25/25 claim: 14
- Areas with no executable proof commands listed: 0
- Areas with no proof artifacts listed: 0
- Areas externally verified: 0
- Areas with sustained/continuous dogfood: 0

## Status by Feature

| ID | Current | Proof | Dogfood | External | Primary blocker |
| --- | --- | --- | --- | --- | --- |
| `feature.setup_install_onboarding` | `partial` | `strong` | `ad_hoc` | `planned` | Needs external replay and broader lifecycle proof beyond local setup smoke. |
| `feature.docs_product_education` | `partial` | `strong` | `ad_hoc` | `none` | Strong local docs proof covers navigation, setup/CLI alignment, internal references, registry honesty, and unsupported 25/25 blockers; external replay and sustained dogfood remain pending. |
| `feature.doctor_status_recovery_update_uninstall` | `partial` | `strong` | `ad_hoc` | `planned` | Local lifecycle proof exists; destructive reset contract and external replay remain pending. |
| `feature.memory_core` | `partial` | `strong` | `ad_hoc` | `none` | Local proof maps capture/lookup/resume/corrections/provenance/trust, but external validation and production reliability remain unproven. |
| `feature.context_compiler_token_savings` | `partial` | `strong` | `ad_hoc` | `planned` | Local fixture proof records saved-token ledger, retained quality, and budget enforcement; independent external replay remains pending. |
| `feature.shared_research_cache` | `partial` | `strong` | `none` | `none` | Strong local inspiration cache proof covers hit/miss, source attribution, content-fingerprint invalidation, allowlist/root isolation, and no private fixture bleed; full RAG/donor extraction, dogfood, external replay, and arbitrary cross-repo sharing remain unproven. |
| `feature.hive_hivemind_coordination` | `partial` | `partial` | `ad_hoc` | `none` | Local proof maps archived hive coordination, roster/authority scripts, and no private context broadcast; sustained production and external hive review remain unproven. |
| `feature.competitor_public_benchmark_replay` | `partial` | `partial` | `none` | `planned` | Fresh local public fixture replay artifact exists; no same-day independent competitor/external replay is registered. |
| `feature.dogfood_reliability_windows` | `partial` | `partial` | `ad_hoc` | `none` | Ad hoc dated dogfood/reliability evidence exists, but no closed sustained reliability window is proven. |
| `feature.external_replay_auditor_proof` | `partial` | `partial` | `none` | `planned` | Local auditor-readiness bundle proof exists; no independent external replay artifact is registered. |
| `feature.product_ux_dashboard_cli_language` | `partial` | `partial` | `none` | `none` | Local CLI/help and dashboard-source language proof exists; no real dashboard browser walkthrough/dogfood/external validation is registered. |
| `feature.network_identity_federation_market` | `partial` | `partial` | `none` | `none` | Local identity-scope proof exists for one user/org across app surfaces; V26 artifact absent and federation/market remain bounded/planned. |
| `feature.release_claim_honesty_gates` | `partial` | `partial` | `ad_hoc` | `none` | Local honesty proof checks registry audit, release checklist hooks, executable proof commands, and unsupported 25/25 overclaim blockers; full release-flow integration and external evidence remain pending. |
| `feature.cross_harness_continuity` | `partial` | `partial` | `ad_hoc` | `none` | Cross-harness replay proof is planned but not recorded. |

## Honest Conclusion

Pillar 01 now provides a registry truth source and audit, but the registry itself confirms whole-app 25/25 is not achieved. Most areas are partial, unknown, planned, stale, or missing external/dogfood evidence.

## Refresh Procedure

1. Update `docs/verification/features.registry.json` first.
2. Keep `docs/verification/FEATURES.md` aligned with high-level registry status.
3. Refresh this report when feature status, proof artifacts, or blockers change.
4. Run `bash scripts/verify/feature-registry-audit.sh` and `git diff --check` before claiming registry truth is valid.
