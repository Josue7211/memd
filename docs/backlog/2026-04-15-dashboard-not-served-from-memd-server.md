# React Dashboard Not Served From memd-server

status: open
severity: critical
phase: Phase I2
opened: 2026-04-15

## Problem

The I2 React dashboard was built as a standalone Vite SPA at apps/dashboard/ on port
5173, completely disconnected from memd-server. Meanwhile, memd-server already serves
a full server-rendered HTML dashboard at GET / on port 8787 (crates/memd-server/src/ui/mod.rs).

Result: two dashboards, two processes, two ports. The React app proxies API calls
back to 8787 via Vite config, requiring a separate dev server running. In production
this means deploying two processes — one for API + old UI, one for new UI — which
is architecturally wrong. memd-server is the single gateway.

The React dashboard should be built to dist/ and served as static files by memd-server
itself, so there is one process (memd-server), one port (8787), serving both the API
and the dashboard.

## Evidence

- `apps/dashboard/vite.config.ts` line 22-29: proxy config forwarding /healthz, /memory, /atlas, /procedures, /coordination, /hive to MEMD_API_URL
- `crates/memd-server/src/ui/mod.rs`: ~2000 lines of server-rendered HTML templates at GET /
- `crates/memd-server/src/main.rs:495`: `.route("/", get(dashboard))` serving old UI
- `apps/dashboard/.env`: MEMD_API_URL pointing to Tailscale IP — proves it was never designed to run same-process
- Both UIs visible: port 8787 shows "Memory Home", port 5173 shows "memd dashboard"

## Fix

1. Add `tower-http` `ServeDir` to memd-server to serve `apps/dashboard/dist/` at `/dashboard/`
2. Add SPA catch-all fallback: any `/dashboard/*` path that doesn't match a static file → serve `index.html`
3. Remove API proxy from Vite config (only needed for dev HMR, not production)
4. Build step: `cd apps/dashboard && npm run build` before `cargo build` (or embed dist/ at compile time via `include_dir` or `rust-embed`)
5. Decide fate of old server-rendered UI at `/` — keep as lightweight inspection view, or redirect to `/dashboard/`
6. Update `apps/dashboard/vite.config.ts` to default MEMD_API_URL to `http://localhost:8787` for dev
7. Remove `.env` Tailscale IP hardcode
