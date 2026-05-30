> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Feature Slice 25: Network Identity, Federation, and Market Layer

## Honest status

Status: **strong local proof**.

This slice verifies the current repository documents and runs a deterministic generated-bundle check proving one shared local memory identity/control plane across priority harness surfaces (Codex, Claude Code, OpenClaw, OpenCode, and Hermes). It also records the current boundaries for federation and market claims.

It does **not** prove live cross-org federation, billing, a public network identity service, a marketplace business layer, or independent external validation. Federation/market capability remains mostly roadmap/planned or earlier local/synthetic proof unless a newer V26/V27/V28 proof artifact is present and cited by the executable proof.

## What this proof validates

The executable proof is `scripts/verify/feature-network-identity-federation-market-proof.sh`. It builds/uses the local `memd` binary, generates a temporary bundle with `project=local-25-5-single-org` and `namespace=org-alpha`, and validates:

1. **Single user/org across app surfaces**
   - Codex uses `memd` as the shared memory control plane and reads/writes the same bundle-local `.memd/wake.md`, `.memd/mem.md`, and `.memd/events.md` files.
   - Hermes uses the same `memd` memory control plane and the same visible bundle files.
   - The hook kit exposes one bootstrap path with `memd setup --output .memd --project <project> --namespace <namespace> --agent <agent>` and generated entrypoints for Codex, Claude Code, OpenClaw, OpenCode, and Hermes.
   - The deterministic generated-bundle check asserts all generated app entrypoints source the same bundle env, use the same wake path, and do not fork `MEMD_PROJECT`/`MEMD_NAMESPACE` per app.
   - This is the corrected scope: one user/org memory identity across app surfaces, not separate identities per harness and not the old three-users mistake.

2. **Network identity proof linkage when present**
   - If a V26 network identity proof script exists, this proof runs it and reports `script_ran`.
   - If dated V26 network identity artifacts exist under `docs/verification/v26-proof-runs/`, this proof cites them.
   - On the current baseline used for this slice, no V26 proof script or dated V26 proof artifact is present, so V26 is reported as `absent_pending_not_claimed` rather than silently assumed.

3. **Federation and market boundaries**
   - Existing V17 routine marketplace proof artifacts are cited as local/synthetic evidence only: marketplace schema/search/install smoke, synthetic 1000-user federation isolation, and a remaining 30-day dogfood gate.
   - 25-star contract/ledger documents keep V27 federation and V28 agent-work-market as explicit future proof gates.
   - The registry row for this feature keeps `blocks_25_25: true` and `external_status: none`.

## Evidence boundaries

Allowed claim: memd has strong local proof that priority harness surfaces are wired to one shared local memory control plane/bundle identity, and federation/market claims are explicitly bounded by existing local/synthetic or planned proof gates.

Forbidden claim: do not claim active network identity service, cross-org federation, public marketplace, billing/market economics, independent external verification, or production-safe federation/market behavior from this proof alone.

## Commands

```bash
bash scripts/verify/feature-network-identity-federation-market-proof.sh
bash scripts/verify/feature-registry-audit.sh
bash scripts/doc-lint.sh
git diff --check
```

## Current related artifacts

- `integrations/codex/README.md`
- `integrations/hermes/README.md`
- `integrations/hooks/README.md`
- `docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.md`
- `docs/verification/v17-proof-runs/2026-05-12-routine-marketplace-suite.ndjson`
- `docs/verification/25-star-CONTRACT.md`
- `docs/verification/25-star-phase-ledger.md`
