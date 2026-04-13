# healthz Masks Database Errors

- status: `closed`
- found: `2026-04-13`
- scope: memd-server

## Summary

The `/healthz` endpoint returns `{"status":"ok","items":0}` when the database
is broken. Load balancers and monitoring will not detect the failure.

## Symptom

- Broken DB → healthz returns 200 with 0 items
- Monitoring sees "healthy" when server can't read or write

## Root Cause

- `routes.rs:6` — `state.store.count().unwrap_or(0)`
- `unwrap_or(0)` converts any DB error into a silent zero

## Fix Shape

- Return 503 if `count()` fails: `state.store.count().map_err(internal_error)?`
- Or return a `status: "degraded"` field with the error

## Evidence

- `routes.rs:3-8` — healthz handler
