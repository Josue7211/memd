---
phase: I2
name: Human Dashboard
version: v2
status: pending
depends_on: [D2, E2, G2]
backlog_items: []
---

# Phase I2: Human Dashboard

## Goal

Humans can browse, correct, and navigate memory through a web UI.

## Deliver

- Memory browser (list, search, filter by kind/lane/tag)
- Correction UI (edit, supersede, mark contested)
- Atlas graph view (regions, entities, links)
- Procedure viewer
- Status dashboard with honest scoring

## Pass Gate

- User can find a specific fact in < 3 clicks
- User can correct a wrong fact through the UI
- Atlas graph renders with clickable nodes
- Status shows real eval score, not lies
- Zero console errors in browser

## Evidence

- Browser test screenshots
- Correction flow walkthrough
- Graph rendering proof
- Console error check

## Fail Conditions

- UI renders but doesn't connect to live data
- Corrections don't persist
- Graph is empty or unnavigable

## Rollback

- Revert UI changes that break API
