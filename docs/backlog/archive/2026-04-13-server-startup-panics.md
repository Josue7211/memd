# Server Startup Panics on Failure

- status: `closed`
- found: `2026-04-13`
- scope: memd-server

## Summary

Server uses `expect()` and `panic!()` for startup errors. If the DB file is
locked, corrupt, or the port is taken, the process crashes with no structured
error output.

## Symptom

- DB locked → panic with "open memd sqlite store"
- Port taken → panic with "bind memd to 127.0.0.1:8787"
- Axum serve fails → panic with "serve memd"
- No graceful error, no retry, no structured log

## Root Cause

- `main.rs:310` — `SqliteStore::open(&db_path).expect("open memd sqlite store")`
- `main.rs:424` — `.unwrap_or_else(|_| panic!("bind memd to {}", bind_addr))`
- `main.rs:425` — `axum::serve(listener, app).await.expect("serve memd")`

## Fix Shape

- Use `match` or `?` with structured error logging
- Print actionable error message (e.g., "port 8787 already in use, set MEMD_BIND_ADDR")
- Exit with non-zero code instead of panic backtrace

## Evidence

- `main.rs:304-426`
