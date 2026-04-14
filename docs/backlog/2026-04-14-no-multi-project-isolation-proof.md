# No Multi-Project Isolation Proof

- status: `open`
- severity: `medium`
- phase: `V2-J2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Memory can target multiple projects but no test proves isolation. One project's memory could leak into another. No namespace enforcement or data boundary test.

## Fix

- Add E2E test: create parallel sessions in different projects
- Verify memory of project-A is invisible to project-B
- Verify namespace collision handling
- Add to phase-J2 acceptance criteria (isolation)
