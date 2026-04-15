# Dashboard .env Hardcoded to Tailscale IP

status: open
severity: medium
phase: Phase I2
opened: 2026-04-15

## Problem

`apps/dashboard/.env` has `MEMD_API_URL=http://100.104.154.24:8787` — a Tailscale IP.
This breaks when Tailscale is down or on a different machine. The Vite config falls
back to `http://localhost:3080` if env is unset, but memd-server runs on 8787, not 3080.

## Evidence

- `apps/dashboard/.env`: MEMD_API_URL=http://100.104.154.24:8787
- `apps/dashboard/vite.config.ts:8`: fallback is localhost:3080, should be localhost:8787

## Fix

1. Change .env to `MEMD_API_URL=http://localhost:8787`
2. Change vite.config.ts default to `http://localhost:8787`
3. Add .env to .gitignore, provide .env.example
4. This becomes moot once Bug #1 (serve from memd-server) is fixed
