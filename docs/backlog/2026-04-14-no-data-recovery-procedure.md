# No Data Recovery Procedure

- status: `open`
- severity: `high`
- phase: `V2-K2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Memory is stored in SQLite. Corruption = total loss. No backup/restore procedure documented or tested. No WAL mode, no checksums, no recovery fallback.

## Fix

- Enable SQLite WAL mode
- Implement backup-on-write (append-only log)
- Document restore procedure
- Test corruption recovery end-to-end
