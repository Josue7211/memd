---
opened: 2026-05-09
phase: v20-evidence-ops
status: runtime-authority-complete
prev_handoff: 2026-05-06-dogfood-installer-m0-m4-ready.md
branch: main
code_commits:
  - 297c240 docs(strategy): add 25-star master roadmap
  - 709e14d feat(runtime): finish rag and mac bridge cleanup
  - 10f18ee feat(server): add gated authority memory search
  - 0ac72d7 chore(server): expose authority search deploy env
mode: 10-star-ceo
---

# Runtime Authority Complete

One sentence: M0-M4 dogfood installer work stayed closed, and the follow-up
RAG/runtime/server/Mac Bridge/authority-search code path is now implemented,
verified, and committed.

## Current Truth

- Current branch is `main`, ahead of `origin/main` by 53 commits.
- `1.0.0` is still blocked on real dated evidence artifacts; V20 evidence ops
  remains the active work.
- The 25-star roadmap is now formal docs, but V21+ is not active until honest
  `1.0.0` close.
- `crates/memd-rag` is now a direct HTTP DTO/client crate using rustls-only
  `reqwest`; it no longer re-exports `memd-sidecar`.
- Client RAG runtime maps retrieve modes explicitly and retries lookup with
  unscoped `route=all` when scoped project/namespace/workspace search returns
  empty.
- Server intrinsic dense embeddings are disabled in this build; sidecar RAG is
  the expected retrieval path.
- Expired-item GC is opt-in through `MEMD_GC_EXPIRED_ITEMS`.
- Mac Bridge is bundled under `integrations/mac-bridge/`, ignored for `.env`
  and `node_modules/`, and installer/docs now surface the Darwin install path.
- Authority inventory search is gated behind `MEMD_AUTHORITY_SEARCH=1` plus
  `MEMD_AUTHORITY_TOKEN`; Portainer compose exposes both env vars.

## Verification

- `cargo check -p memd-rag` passed.
- `cargo check -p memd-server` passed with existing warnings.
- `cargo check -p memd-client` passed with existing warnings.
- `cargo test -p memd-rag normalize_base_url -- --nocapture` passed.
- `cargo test -p memd-server expired_item_gc_is_opt_in -- --nocapture` passed.
- `cargo test -p memd-client lookup_route_all_retries_without_project_filters -- --nocapture` passed.
- `cargo test -p memd-server authority_search_is_opt_in_token_gated_and_reads_legacy_private_rows -- --nocapture` passed.
- `node --check integrations/mac-bridge/server.js` passed.
- `bash -n integrations/mac-bridge/install.sh` passed.
- `bash -n scripts/install-memd.sh` passed.
- `git diff --check` passed.

## Next Actions

- Resume V20 evidence gates:
  - 3 real users.
  - 3 harness-user pairs.
  - 3 devices.
  - V19 external auditor packet.
  - V20 third-party replay packet.
  - Weekly dated evidence notes.
- Do not tag `1.0.0` until those real artifacts land.
- Keep generated memd timestamp churn out of feature commits unless it is an
  intentional fixture update.

## Hard Stop

No V21+ status claims, hosted-product claims, or category-maturity claims until
`1.0.0` is closed with real evidence.
