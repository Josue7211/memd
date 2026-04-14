# TTL Enforcement: Items Expire but Never Get Removed

status: open
severity: medium
phase: Phase I
opened: 2026-04-14

## Problem

apply_lifecycle() marks items as Expired. Never removes them from DB.
No GC pass. Expired items pile up forever. Related to inbox-never-drains
(#29 — partially fixed with drain endpoint, but no automatic GC).

## Fix

1. Add GC pass: delete expired items older than grace period
2. Run GC in worker maintenance loop
3. Validate TTL on admission
