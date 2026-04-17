---
status: open
severity: medium
phase: I2
opened: 2026-04-15
scope: unspecified
---
# Dashboard .env Hardcoded to Tailscale IP

status: open
severity: medium
phase: Phase I2
opened: 2026-04-15

## Problem

`apps/dashboard/.env` has `MEMD_API_URL=http://100.104.154.24:8787` — a Tailscale IP.
Breaks when Tailscale is down or on a different machine. Should be localhost.

Additionally, `vite.config.ts` line 8 defaults to `http://localhost:3080` when
MEMD_API_URL is unset. memd-server runs on 8787, not 3080.

## Evidence

- `apps/dashboard/.env`: `MEMD_API_URL=http://100.104.154.24:8787`
- `apps/dashboard/vite.config.ts:8`: `const MEMD_API = env.MEMD_API_URL || "http://localhost:3080"`

## Fix

1. Change .env to `MEMD_API_URL=http://localhost:8787`
2. Change vite.config.ts default to `http://localhost:8787`
3. Add .env to .gitignore, provide .env.example
4. Moot once dashboard-not-served-from-memd-server is fixed (no proxy needed in prod)
