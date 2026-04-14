# 5 Starter Lanes Have No Source Material

status: open
severity: medium
phase: Phase I
opened: 2026-04-14

## Problem

Theory lock defines 6 starter lanes. Only inspiration has source material.
Missing directories and seed content for:

| Lane | Purpose | Current State |
|---|---|---|
| Design | UI decisions, component patterns, visual direction | no directory |
| Architecture | system design, API shape, data flow, theory locks | directory created, needs multi-file seed |
| Research | findings, hypotheses, evidence, conclusions | no directory |
| Workflow | operating procedures, CI/CD, automation patterns | no directory |
| Preference | user corrections, style choices, tool preferences | no directory |

## Fix

1. Create `.memd/lanes/<name>/` directories for each
2. Seed with existing content (theory locks → architecture, procedures → workflow, etc.)
3. Each lane gets a `<NAME>-LANE.md` overview + domain-specific source files
4. Ingestion pipeline will compile these into DB items

## Depends On

- no-source-ingestion-pipeline (for compilation)
- But directories and seed files can be created now
