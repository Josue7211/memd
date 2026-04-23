---
version: v9
kind: integration-plan
status: ready-to-execute
opened: 2026-04-22
revised: 2026-04-22
scope: A9..G9
depends_on: [../../verification/0.1.0-CONTRACT.md, ../../verification/milestones/MILESTONE-v9.md, ../../contracts/federated-memory-visibility.md]
---

# V9 Integration — Multi-User Adversarial Suite

> Read after all seven `phase-{a9..g9}-plan.md` specs. This doc covers what no
> single phase plan owns: shared multi-user fixtures, cross-user harness state
> isolation schema, adversarial suite construction, scorecard strict-mode
> regenerator requirements, and the pre-ship dry-run checklist.

## 1. Execution-order discipline

Phase-level dependency (strict):

```
A9 ──► B9 ──┐
       │    │
       └► C9 ──► D9 ──┐
              │        │
              └──► E9 ─┤
                        │
                        ▼
                        F9 ──► G9
```

Rules:

- A9 tasks 1–5 land first (per-user state isolation + multi-harness schema). B9 cannot start until A9 Task A9.4 (handoff).
- C9 requires B9 read-path negative controls (user A private invisible to B).
- D9 requires C9 multi-harness flip proof (agent identity isolation).
- E9 requires C9 + D9 adversarial fixtures.
- F9 requires all A9..E9 code (pre-ship rehearsal).
- G9 requires everything + full adversarial suite wired.

No phase may short-circuit a prior dependency. If blocked, file backlog + surface at next handoff.

## 2. Shared multi-user test fixtures

Lives in `crates/memd-client/fixtures/shared/multi-user/`, referenced by A9..G9:

| Fixture | Owner | Shared with | Content |
| --- | --- | --- | --- |
| `ua-ub-ua-3session.jsonl` | A9 | B9, C9, D9, G9 | 3-session round-trip (user A → user B → user A) |
| `cross-user-corrections.jsonl` | C9 | D9, E9, G9 | user A correction, user B sees it with attribution |
| `identity-collision-10turn.jsonl` | D9 | G9 | same claim hash from user A + user B, dedup proof |
| `scope-escalation-negative.jsonl` | D9 | G9 | Local→Project automatic attempt, must fail |
| `agent-spoofing-negative.jsonl` | D9 | G9 | agent B writes with A's agent_id, must fail |
| `cross-workspace-leak-negative.jsonl` | E9 | G9 | workspace W1 correction does not appear in W2 |
| `per-scope-retention-negative.jsonl` | E9 | G9 | workspace-scoped correction does not leak to global |
| `flip-ua-ub-ua.jsonl` | C9 | G9 (anchor) | canonical divergence test across 3 harnesses |
| `federated-visibility-matrix.json` | D9 | G9 (reference) | (scope, visibility, agent_id) × (read, write, cross-user) truth table |

Convention: each phase plan's `fixtures/<phase>/` contains **only** phase-local fixtures. Multi-user shared fixtures consolidate after A9 lands.

## 3. Cross-user harness state schema (A9 substrate plumbing)

Three schema locks in A9 unblock multi-user correctness. All structural, no axis credit.

### 3.1 User-scoped focus bucket

Every `focus_bucket` row carries `user_id` (NOT agent_id). Wake reconstruction filters by
current_user before querying wake_context. Prevents user B from observing user A's focus
on resume.

Column added (A9 Task A9.2):
- `focus_buckets.user_id` TEXT NOT NULL
- `focus_buckets.user_id_session_seq` INTEGER NOT NULL (monotonic per user_id + session_id)

Index: `(user_id, session_id, user_id_session_seq)` for user-scoped isolation.

Rationale: session_continuity owns the SC axis lift in V9. If B's wake includes A's focus
without filtering, SC bleed is real and the +1 is phantom.

### 3.2 Agent identity immutability

Every `memory_items` row carries `agent_id` + `harness_preset`. Neither field is
mutable post-insert. Correction/supersede creates a new row with same agent_id;
does not change the original author's identity.

Unique constraint added (A9 Task A9.1):
- `UNIQUE (memory_id, agent_id, harness_preset)` — prevents lateral write escalation

Constraint check: `UPDATE memory_items SET agent_id = X WHERE agent_id = Y` is forbidden.
Only `INSERT` is allowed; mutations are `INSERT` with supersede flag.

Rationale: agent_id spoofing (D9 adversarial test 5) requires immutable agent identity.

### 3.3 Workspace-scoped visibility enforcement

`memory_items.workspace_id` is binding for Workspace-visibility reads. A query from
workspace W1 with visibility=Workspace never observes items where workspace_id != W1,
even if the items are in the same project or global scope. The rule is: visibility wins
over scope when there's a conflict (stricter axis). Workspace ⊂ Project ⊂ Global.

Column present (inherited from V4+), enforcement added (B9):
- `memory_items.workspace_id` TEXT (nullable for project/global scope)

Read-path filter (B9 scope):
```
if memory.visibility == Workspace:
  AND memory.workspace_id == query.workspace_id
```

Rationale: per-scope retention override (D9 test 7) requires query-time enforcement,
not just schema constraint.

## 4. Multi-user adversarial suite (G9 gate)

8 scenarios from `federated-memory-visibility.md`, wired as G9 harness. Each scenario
is a (user A, user B) pair with (agent α, agent β) in (harness 1, harness 2).

### Scenario 1: Cross-user read leak

**Setup:** User A writes Local/Private memory. User B queries same workspace.
**Assertion:** User B retrieval **does not include** user A's Local/Private memory.
**Fixture:** `agent-a-local-private.jsonl`
**Negative control:** Drop workspace_id visibility filter → assertion fires.

### Scenario 2: Cross-user write escape

**Setup:** User B attempts to insert with agent_id="user-a-agent-1".
**Assertion:** Write **fails** with permission_denied on agent_id mismatch.
**Fixture:** `agent-spoofing-negative.jsonl`
**Negative control:** Drop immutability constraint → assertion fires.

### Scenario 3: Correction provenance preservation

**Setup:** User A issues correction in Workspace W1. User B queries same W1.
**Assertion:** User B sees correction **with source attribution to user A**.
**Fixture:** `cross-user-corrections.jsonl`
**Negative control:** Drop provenance field → assertion fires.

### Scenario 4: Content-hash dedup with co-attribution

**Setup:** User A inserts "Postgres primary key is UUID". User B inserts identical.
**Assertion:** Single record with both (user_a_agent, user_b_agent) in co-authors; neither silent-deleted.
**Fixture:** `identity-collision-10turn.jsonl`
**Negative control:** Drop co-author dedup → assertion fires (duplicates accumulate).

### Scenario 5: Agent_id spoofing test (immutability)

**Setup:** Agent B's harness attempts `memd capture --agent-id "user-a-agent-1" ...`.
**Assertion:** API **rejects** with "agent_id must match caller identity".
**Fixture:** `agent-spoofing-negative.jsonl`
**Negative control:** Remove agent_id validation → write succeeds (bad).

### Scenario 6: Scope escalation (Local → Project automatic prevention)

**Setup:** Agent A inserts Local/Private memory. Agent B calls lookup with `--promote=global`.
**Assertion:** Lookup **does not auto-promote**; returns error requiring `--promote-trusted` flag + user confirm.
**Fixture:** `scope-escalation-negative.jsonl`
**Negative control:** Remove promotion API guard → assertion fires.

### Scenario 7: Per-scope retention override (Workspace does not leak Global)

**Setup:** User A corrects a claim in Workspace W1. User B queries same claim in Global scope.
**Assertion:** Global query **does not include** W1-scoped correction; W1 boundary respected.
**Fixture:** `per-scope-retention-negative.jsonl`
**Negative control:** Drop workspace_id filter on Workspace visibility → assertion fires.

### Scenario 8: Multi-user flip test (canonical consistency across 3 harnesses)

**Setup:** User A (claude-code, S1) → User B (codex, S2) → User A (claude-code, S3). Both write to shared canonical.
**Assertion:** S3 canonical matches S2 canonical; no branching; user A focus does not leak into B's S2 wake.
**Fixture:** `flip-ua-ub-ua.jsonl`
**Negative control:** Remove user_id isolation on focus → assertion fires (A's focus visible in B's S2).

## 5. Cross-harness multi-user scenario outline (3-session, 2 users, 3 agents)

### Session 1 — "establish" (user A, claude-code, 5 turns)

| Turn | Actor | Utterance | Phases | User | Agent |
| --- | --- | --- | --- | --- | --- |
| T1 | user A | "API returns 200 on success." | A9, C9 | A | α (claude-code) |
| T2 | user A | "No, 201 actually." | C9 (correction) | A | α |
| T3 | user A | "Focus: finish auth service." | A9 (focus) | A | α |
| T4 | agent α | Read `src/auth.rs` | A9 (ledger) | A | α |
| T5 | system | SessionStop → PreCompact | A9, B9 | A | α |

Cut 1: user A focus ≠ {} in focus_buckets; correction T2 in corrections table; both tagged user_id=A.

### Session 2 — "cross-user" (user B, codex, 5 turns)

| Turn | Actor | Utterance | Phases | User | Agent |
| --- | --- | --- | --- | --- | --- |
| — | system | Wake (must NOT include user A focus) | A9 (isolation), C9 (flip start) | B | β (codex) |
| T6 | user B | "What's the API response code?" | B9 (read cross-user), E9 (provenance) | B | β |
| T7 | agent β | Read `src/api.rs` | A9 (cross-harness ledger) | B | β |
| T8 | user B | "Actually, let's use 204 No Content." | C9 (correction), E9 (cross-user source) | B | β |
| T9 | system | SessionStop → PreCompact | A9, B9, C9 | B | β |

Cut 2: user A focus still isolated (not visible to B's wake); user B observes user A's correction T2 in lookup with attribution=A; user B's correction T8 recorded with agent=β; canonical may show divergence (T2 vs T8) surfaced in D9.

### Session 3 — "round-trip" (user A, claude-code, 5 turns)

| Turn | Actor | Utterance | Phases | User | Agent |
| --- | --- | --- | --- | --- | --- |
| — | system | Wake (user A context restored, user B wake invisible) | A9 (restoration), C9 (flip round-trip) | A | α |
| T10 | user A | "What did user B say about the code?" | E9 (cross-user + provenance) | A | α |
| T11 | agent α | Read `src/api.rs` again | A9 | A | α |
| T12 | user A | "OK, 204 it is." | C9 (merge divergence or supersede) | A | α |
| T13 | system | SessionStop | A9 | A | α |

Cut 3: Canonical resolved (T2 vs T8 reconciled or both visible with divergence marker); user A focus matches pre-S2 (T3); zero A/B bleed.

## 6. Scorecard strict-mode regenerator (G9 requirement)

G9 Task G9.4 writes to `docs/verification/MEMD-10-STAR.md`. Regenerator enforces:

1. **Axis cap**: CH score ≤ 6 (owned by V9). Rejects if > 6.
2. **SC score ≤ 6**: Rejects if > 6 (owned by V9).
3. **Non-owned axes immutable**: CR, PR, RR, TE, TP must match V8 post-values (5, 4, 7, 5, 6). Fails if regenerator tries to adjust.
4. **G9 evidence path**: All 8 adversarial scenario results in `docs/verification/v9-proof-runs/YYYY-MM-DD-adversarial-suite.ndjson`. Fails if missing.

Regenerator failure → V9 does not close, V10 entry blocked.

Target table post-V9:

```markdown
## 10-Star Composite Scorecard

| Axis | Weight | Score | Status |
|------|--------|-------|--------|
| Session continuity | 20% | 6/10 | A9 user context isolation + C9 multi-user flip |
| Correction retention | 15% | 5/10 | unchanged — V10 scope |
| Procedural reuse | 15% | 4/10 | unchanged — V10 scope |
| Cross-harness continuity | 15% | 6/10 | C9 multi-user flip + D9 adversarial suite (8 scenarios) |
| Raw retrieval strength | 15% | 7/10 | unchanged — stable |
| Token efficiency | 10% | 5/10 | unchanged — stable |
| Trust + provenance | 10% | 6/10 | unchanged — stable |

**Composite: 5.60 (V9 gate requirement) — regenerated YYYY-MM-DD by G9 harness run <id>**

Evidence: docs/verification/v9-proof-runs/YYYY-MM-DD-adversarial-suite.ndjson
```

## 7. Pre-ship dry-run checklist (V9 close = RC entry gate)

G9 closes when **all** checklist items pass:

- [ ] 8 adversarial scenarios run; all 8 assertions pass
- [ ] Negative controls fire correctly (inject failure → test catches it for each scenario)
- [ ] Multi-user flip test: user A → user B → user A round-trip, canonical consistent
- [ ] User A's focus ≠ user B's focus in Cut 2 wake (isolation proven)
- [ ] User B observes user A's correction T2 with attribution=user-a (provenance proven)
- [ ] Scorecard regenerator strict-mode enforces axis caps (CH ≤ 6, SC ≤ 6)
- [ ] `docs/verification/MEMD-10-STAR.md` composite = 5.60, all axis scores match contract
- [ ] `docs/verification/v9-proof-runs/YYYY-MM-DD-adversarial-suite.ndjson` written and signed
- [ ] federated-memory-visibility.md enforcement surface live in code (B9 read filters, A9 schema, D9 identity checks)
- [ ] Release-candidate artifact ready: V9 → V10 entry gate open

Passing this gate = pre-ship window opened. V10 executes self-improvement (SC +1, CR +1, PR +2, RR +1); if V10 completes by target, 0.1.0 ships.

## 8. Commit strategy

### Plan-spec land phase (this task)

Eight atomic commits on `research/mining`, one per file:

1. `docs(v9): phase-a9-plan implementation spec`
2. `docs(v9): phase-b9-plan implementation spec`
3. `docs(v9): phase-c9-plan implementation spec`
4. `docs(v9): phase-d9-plan implementation spec`
5. `docs(v9): phase-e9-plan implementation spec`
6. `docs(v9): phase-f9-plan implementation spec`
7. `docs(v9): phase-g9-plan implementation spec`
8. `docs(v9): V9-INTEGRATION cross-phase plan`

### Execution commits per phase

Each phase plan produces commits per task (A9 = 6 commits, B9 = 7, etc.). Those commits are produced by future agents executing the phases, **not** by this plan-spec-land task.

### Handoff commit

After the 8 docs commits, one final commit:

```
docs(handoff): V9 plan specs landed, next agent executes A9
```

Content: new file `docs/handoff/YYYY-MM-DD-v9-plan-spec-complete-next-execute.md`.

## 9. Cross-phase API surface summary

| Introduced in | Symbol / Path | Consumed by |
| --- | --- | --- |
| A9 | `memd_core::focus_bucket::*` with user_id filter | B9, C9, D9, G9 |
| A9 | `memory_items.user_id_session_seq` column | A9..G9 (ordering) |
| A9 | `docs/contracts/multi-user-harness-state.md` (new) | B9, C9, D9 |
| B9 | `memd_core::query::apply_workspace_filter` | C9, D9, E9, G9 |
| B9 | `.memd/logs/visibility-audit.ndjson` | G9 |
| C9 | Multi-harness flip test fixture | D9, G9 (anchor) |
| D9 | Adversarial scenario fixtures (8) | G9 (assertion battery) |
| D9 | `docs/contracts/federated-visibility-matrix.json` | G9 reference |
| E9 | Cross-user correction propagation API | G9 (scenario 3 anchor) |
| F9 | Dry-run harness variant | G9 (pre-ship rehearsal) |
| G9 | `docs/verification/v9-proof-runs/*.ndjson` | V10 entry gate |

## 10. Open questions for next executor

Surface in phase kickoff notes:

- Multi-user UI story — does operator surface for divergence already exist? Check D9 scope before design.
- Fixture anonymization — does `scripts/dev/` have scrubber for multi-user logs? Investigate before A9.
- Workspace vs Project vs Global precedence — theory-lock or pragmatic? Confirm against `docs/theory/locks/`.
- `co_authors` table schema — list of (agent_id, harness_preset) or aggregated JSON? Decide before A9.
- CI substrate for G9 — 8-scenario battery runtime? Estimate before F9.

## 11. Federated-memory contract enforcement binding

V9 is the enforcement milestone. Every test failure against
`docs/contracts/federated-memory-visibility.md` is a P0 bug against the
contract, not against the code. Contract violations (spec that contradicts
the doc) fail review and require theory-lock revision.

Contract text is **immutable during V9 execution**. Requests to weaken the
contract (e.g., "allow workspace to leak to global") are backlog items for
V10+; they do not change V9 spec.

## 12. Exit criteria for V9 as a milestone

All seven phase exit criteria met AND G9 exit criteria met AND:

- 10-STAR composite ≥ 5.60 written to `docs/verification/MEMD-10-STAR.md` by G9 scorecard regenerator.
- No axis score above targets in MILESTONE-v9.md (regenerator fails if violated).
- SC = 6, CH = 6 (both owned axes at target).
- All 8 adversarial scenarios passing, negative controls firing.
- `docs/verification/milestones/MILESTONE-v9.md` filled with evidence paths.
- `ROADMAP.md` V9 → closed, V10 → in progress.
- No open backlog tagged `axis: session_continuity` or `axis: cross_harness` at severity blocker.
- Multi-user flip test round-trip complete + canonical consistent.
- federated-memory-visibility.md enforcement live (B9, A9, D9 code shipped).
- Pre-ship checklist all green.
- Handoff doc points at `docs/phases/v10/` (to be created in V10 plan-spec phase).

## 13. Changelog

- 2026-04-22 initial spec. Contract-enforced multi-user suite. 8 adversarial scenarios. Pre-ship entry gate at G9. Axis caps: SC +1, CH +2 (5.60 composite target).
