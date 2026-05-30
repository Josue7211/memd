> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Feature product_ux_dashboard_cli_language 25-star local proof

[[ROADMAP]]: Verification artifact for the product UX/dashboard/CLI language feature slice. This is a strong local text, source, CLI-help, and static-build inspection; it is not a browser walkthrough, usability study, external validation, dogfood result, or polished-dashboard claim.

## Scope

Feature registry id: `feature.product_ux_dashboard_cli_language`

This proof inspects the in-repository user-facing language surfaces that are available locally:

- CLI help text for `memd`, `memd setup`, `memd setup-demo`, and `memd doctor`
- CLI source descriptions for setup, setup-demo, doctor, guided setup, and interactive setup
- setup/getting-started docs in `README.md` and `START-HERE.md`
- dashboard source/design artifacts under `apps/dashboard/`, including app bootstrap, router shell, status, ask, memory, atlas, empty-state, and harness-health components
- static UI contract checks that reject known unsupported completed-evidence phrases
- dashboard static build through `npm ci && npm run build` when local `npm` is available
- registry/report claim language for this feature row

## Local proof command

Run:

```bash
bash scripts/verify/feature-product-ux-dashboard-cli-language-proof.sh
```

The script builds/runs CLI help through the guarded cargo wrapper, checks the setup path is discoverable from both docs and CLI help, verifies dashboard source/design/routes/components exist, rejects unsupported dashboard copy that would imply external gates are complete, runs the dashboard static build, runs `scripts/doc-lint.sh`, and confirms the registry row remains honest about missing browser/external proof.

## Findings

| Axis | Local evidence inspected | Result | Boundary |
| --- | --- | --- | --- |
| CLI clarity | `memd --help`, `memd setup --help`, `memd setup-demo --help`, `memd doctor --help`, `crates/memd-client/src/cli/args.rs`, `args_memory.rs` | The primary setup commands now have top-level descriptions. Setup exposes plain-language flags for `--guided`, `--interactive`, `--summary`, and `--json`; doctor exposes `--repair`. | Many unrelated top-level subcommands may still have sparse descriptions, so this is not a polished whole-CLI copy claim. |
| Setup/getting-started path | `README.md#quickstart` and `START-HERE.md` | First-time install points to README Quickstart; guided setup, interactive setup, setup proof, and troubleshooting are named directly. | This does not prove a new external user completed setup unaided. |
| Dashboard source contract | `apps/dashboard/package.json`, `DESIGN.md`, `app/main.tsx`, `app/router.tsx`, `routes/__root.tsx`, `index.tsx`, `ask.tsx`, `memory.tsx`, `atlas.tsx`, `components/ui/empty-state.tsx`, `harness-health.tsx` | A dashboard app, route shell, navigation, status/control center, Ask memd, Memory Browser, Atlas, empty states, harness health, and status/readiness copy are present and use consistent product language. | No real browser session, screenshot, video, or end-to-end dashboard walkthrough artifact is recorded by this proof. |
| Static UI build | `npm ci && npm run build` in `apps/dashboard` | The dashboard compiles locally without requiring a browser. The observed build includes a known warning that `../tsconfig.json` extends missing `astro/tsconfigs/strict`, but Vite completed successfully. | Static build does not prove visual quality, accessibility compliance, data correctness, or browser interaction. |
| Unsupported/confusing claims | Dashboard source and registry/report claim language | Dashboard copy now says runtime status is local evidence only, release proof still requires real-user/device/auditor/third-party packets, and the dashboard does not claim those gates are complete. The proof rejects old completed-evidence phrases. | Browser/external/dogfood gates remain unproven and still block any honest 25/25 claim. |
| Consistent user-facing language | Docs, CLI help, dashboard routes/components, and registry/report | The same product vocabulary appears across docs, CLI, and dashboard source: `memd`, setup, guided/interactive setup, memory, status/readiness, ask/search, atlas, control center, and release gates. | The proof is local and textual/static; it does not evaluate visual hierarchy, accessibility in-browser, or user comprehension. |

## Current claim level

- `current_status`: partial
- `proof_status`: strong local proof for CLI/help/descriptions, docs, dashboard source/routes/components, unsupported-claim guard, and dashboard static build
- `dogfood_status`: none
- `external_status`: none
- Blocks 25/25 claim: yes

Allowed claim: strong local docs/CLI/dashboard source language and dashboard static build proof exists for setup discoverability, vocabulary consistency, route/component presence, and unsupported-claim guard.

Forbidden claim: do not claim polished product UX, complete dashboard, browser-verified walkthrough, accessibility compliance, dogfood completion, external validation, or 25/25 readiness from this strong local proof alone.
