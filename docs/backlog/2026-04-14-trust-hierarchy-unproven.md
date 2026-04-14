# Trust Hierarchy Unproven

- status: `open`
- severity: `high`
- phase: `V2-J2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Trust hierarchy defines human > canonical > promoted > candidate priority for item selection. Theory is sound but never proven with E2E test. No test creates items at each level, then verifies selection respects rank order under real working memory load.

## Fix

- Add E2E test: create items at each trust level with same content key
- Verify working memory always selects highest rank
- Verify low-rank items are suppressed, not deleted
- Add to phase-J2 acceptance criteria (trust enforcement)
