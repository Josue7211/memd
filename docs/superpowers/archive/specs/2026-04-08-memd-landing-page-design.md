# memd Landing Page Design

## Background
`memd` is a memory control plane for agents. The repo already has a strong product story, but it does not yet have a public landing page to explain the value proposition, compare pricing, or convert visitors into users.

The landing page should support the first monetization pass: OSS local core, paid hosted sync, and team/enterprise upsell.

## Goal
Create a focused, conversion-oriented landing page that explains `memd`, shows why it is different from generic memory APIs, and drives visitors to start free, view pricing, or join a beta.

## Non-Goals

- Do not build a blog.
- Do not build auth or account management.
- Do not build the paid backend yet.
- Do not redesign the Rust workspace or core memory engine.
- Do not add a CMS or marketing automation system in the first pass.

## Audience

- Solo AI builders using agent tools daily.
- Small AI startups with multiple engineers.
- Technical users who care about source-linked memory, continuity, and trust.

## Core Message

`memd` is not a memory bucket. It is the control plane that makes memory reliable across sessions, machines, and teams.

The page should communicate:

- memory survives real work
- context can be resumed cleanly
- source links and provenance matter
- local OSS is free
- hosted cloud is the paid path

## Page Structure

1. Top nav with logo, Pricing, Docs, GitHub, and primary CTA.
2. Hero with headline, subhead, and two CTAs.
3. Problem section with the pain points `memd` solves.
4. How it works section with the capture-route-compact-recall-verify loop.
5. Differentiators section for source-linked memory, intent-aware routing, local-first OSS, and hosted continuity.
6. Use cases section for solo builders, teams, and multi-machine workflows.
7. Pricing section with Free, Pro, Team, and Enterprise tiers.
8. Trust section for provenance, freshness, auditability, and rollback.
9. Final CTA block.
10. Footer with docs, repo, license, and contact links.

## Visual Direction

- Use a strong, systems-oriented look rather than a generic SaaS template.
- Prefer warm neutral backgrounds, deep ink text, and one distinct accent color.
- Use expressive typography, not default system stacks.
- Add one hero visual that suggests a control plane or command surface.
- Keep the page readable on mobile first, then scale up cleanly on desktop.

## Content Requirements

- Explain the product in one sentence above the fold.
- State the free OSS path clearly.
- Show pricing on the homepage.
- Make the differentiator explicit: source-linked, intent-aware, visible memory.
- Keep copy technical enough for builders but simple enough for a first-time visitor.

## Success Criteria

- A visitor can understand what `memd` is in under 30 seconds.
- A visitor can tell the difference between `memd` and a generic memory API.
- A visitor can see the free vs paid path without hunting.
- The page has a clear CTA and a coherent information hierarchy.

## Implementation Constraint

The landing page should live in a separate app folder inside the same repo so it stays isolated from the Rust workspace while remaining easy to ship.
