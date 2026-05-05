---
run: V8-operator-ui-proof
date: 2026-05-05
status: pass
surface: operator
---

# V8 Operator UI Proof

Command:

```bash
/opt/homebrew/bin/node node_modules/astro/astro.js build
```

Browser proof:

```bash
NODE_PATH=/Users/aparcedodev/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules \
  /opt/homebrew/bin/node <playwright-operator-proof>
```

Assertions:

- `cost_ledger_visible=true`
- `budget_tunable=true`
- `provenance_depth_max=3`
- `correction_preview_visible=true`
- `rollback_actor_ui_visible=true`
- `memory_inspector_filter_visible_nodes=1`
- `console_errors=0`

Artifacts:

- `operator-desktop.png`
- `operator-mobile.png`

Note: Playwright package was present; Chromium for Testing downloaded, but headless-shell
download timed out. Proof used the downloaded Chrome executable path directly.
