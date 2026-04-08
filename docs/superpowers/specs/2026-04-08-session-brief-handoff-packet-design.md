# Session Brief and Handoff Packet Design

## Background
Memd already has session-aware status, resume, handoff, tab IDs, compiled memory pages, and visible truth surfaces. The remaining quality gap is not raw capability. It is that a new session can still take too long to understand:

- what is happening now
- what changed last
- what the next actions are
- what matters for this project/session/tab
- what proof exists

The goal of this slice is to make a new session useful in one read, with no guessing and no hidden context.

## Goal
Create a single compact session brief packet that becomes the default high-signal view for:

- `status`
- `resume`
- `handoff`
- new-session bootstrap

The packet must answer, quickly and consistently:

- what we are doing now
- what changed last
- why it matters
- what to do next
- what identity and scope this belongs to
- what proof links back to source

## Non-Goals

- Do not replace compiled memory pages.
- Do not move the source of truth into a chat transcript.
- Do not add a new semantic backend.
- Do not redesign the harness packs.
- Do not broaden into a full analytics dashboard.

## Proposed Shape

The session brief is a compact artifact rendered from existing memd state. It should be stable, inspectable, and cheap to read.

### Header
The top line is the identity and freshness layer:

- project
- namespace
- session
- tab
- goal
- freshness
- blocker

### Body
The body is ranked by importance:

1. `what we are doing now`
2. `last handoff + why`
3. `next 3 actions`
4. `open loops`
5. `contradictions`

### Proof
The packet ends with evidence links:

- source pages
- raw evidence
- last change or last refresh timestamp

## Data Rules

The packet should be built from existing runtime and bundle state, not invented text.

- If a field exists, show it.
- If a field is missing, label it missing.
- If freshness is low, surface that explicitly.
- If contradictions exist, keep them visible rather than hiding them.
- If a claim cannot be supported, do not promote it into the brief.

## Ranking Rules

The packet must rank fields by utility for a new session:

- `now` first
- `why` second
- `next` third
- `proof` last

Secondary data must not bury the current task.

## Surfaces

The same packet should appear in these places:

- `status --summary`
- `resume --summary`
- `handoff --summary`
- new-session bootstrap output
- `MEMD_SESSION_BRIEF.md` as the durable artifact

The brief should also be linkable from the visible memory surface so a user can drill down without losing scope.

## Session Bootstrap

On a fresh session, the brief should be the first thing read after identity and scope are loaded.

Expected bootstrap order:

1. project
2. namespace
3. session
4. tab
5. brief
6. drilldown links

This keeps the session from guessing about the current task.

## Handoff Behavior

Handoff should use the same brief format, but include:

- the last completed action
- the next action after that
- any unresolved blocker
- the most relevant proof links

That makes the handoff packet a continuation of the session brief, not a separate format.

## Quality Criteria

The packet is good if it:

- fits in one quick read
- helps a new session act correctly without re-asking
- distinguishes two tabs in the same project
- makes stale state obvious
- keeps proof one click away
- reduces repeated explanation across sessions

The packet is bad if it:

- repeats everything from compiled memory pages
- hides freshness
- buries the current task under history
- invents context from prior conversation

## Acceptance Criteria

This design is complete when:

- the session brief structure is fixed
- the header/body/proof split is defined
- `status`, `resume`, and `handoff` can all render the same packet shape
- new sessions can read the same brief first
- proof and freshness are visible

## Notes

This is intentionally a control-plane artifact, not a knowledge-base replacement. The compiled memory pages remain the long-form truth. The session brief is the fast front door.
