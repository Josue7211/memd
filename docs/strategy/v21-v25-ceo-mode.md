# V21-V25 CEO Mode

> Secondary/reference doc. For active project truth start with [[ROADMAP]].

Status: post-1.0 strategy seed; not active until `1.0.0` gates close.

## Thesis

V20 closes the memory OS substrate. V21-V25 should not add more "memory core"
just because we can. The next five versions should turn a 10/10 substrate into
a durable business, deployment, and ecosystem wedge.

## Rules

- Do not start V21 before honest `1.0.0` close.
- No new substrate axis credit exists after V20; every V21-V25 gate must be
  product, distribution, revenue, reliability, or governance proof.
- Code changes before `1.0.0` remain blocker fixes only.
- Each version needs one CEO metric, one kill criterion, and one proof artifact.

## V21: Hosted Reliability

Goal: memd becomes boring to run.

CEO metric: 99.9% monthly uptime on a hosted dogfood deployment.

Build:
- hosted control plane
- backup/restore drill automation
- migration runbooks
- incident review template
- support bundle export

Gate:
- 30-day hosted reliability window
- restore drill under 15 minutes
- zero data-loss incidents

Kill criterion: if hosted ops require manual SSH babysitting, do not advance.

## V22: Team Adoption

Goal: small teams use memd daily without founder support.

CEO metric: 5 teams, 30 active days, >=70% weekly retention.

Build:
- team onboarding path
- admin seats/roles
- team evidence dashboard
- support docs
- invite/revoke workflow

Gate:
- 5 real teams onboarded
- at least 3 teams complete a correction + replay + sync workflow
- support load under 30 minutes per team per week

Kill criterion: if team setup needs live handholding, fix onboarding before V23.

## V23: Ecosystem Integrations

Goal: memd becomes the default memory backend for harnesses.

CEO metric: 10 maintained integrations with replay proof.

Build:
- integration certification kit
- harness compatibility matrix
- versioned adapter contract
- partner integration examples
- public replay badges

Gate:
- 10 integrations pass certification
- 3 external maintainers contribute fixes
- adapter upgrade does not break prior certified integrations

Kill criterion: if integrations fork behavior, freeze and harden contract.

## V24: Enterprise Trust

Goal: buyers can approve memd.

CEO metric: complete SOC2-lite evidence pack accepted by one external reviewer.

Build:
- security review packet
- data retention controls
- audit-log export workflows
- admin compliance reports
- threat model refresh

Gate:
- external security/compliance review passes with no blocker severity findings
- retention/delete/export flows verified
- audit replay accepted by reviewer

Kill criterion: if trust story needs promises instead of artifacts, stop.

## V25: Revenue Engine

Goal: prove memd can sustain itself.

CEO metric: first 10 paying customers or equivalent signed pilots.

Build:
- pricing/package validation
- billing hooks
- customer success loop
- churn reason capture
- sales demo proof pack

Gate:
- 10 paying customers or signed pilots
- at least 3 customers renew or expand after first cycle
- support + infra margin documented

Kill criterion: if users love proof but will not pay, rethink package before more engineering.

## Order

V21 -> V22 -> V23 -> V24 -> V25.

Reliability before adoption. Adoption before ecosystem. Ecosystem before
enterprise trust. Trust before revenue scale.
