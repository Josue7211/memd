# Setup Guide

> For fresh-session recovery do not start here. Start with [[ROADMAP]] and
> [[docs/WHERE-AM-I.md|WHERE-AM-I]]. Use this doc once you already know what
> state the project is in.

This is the longer setup and usage path for `memd`. The README only keeps the
minimal happy path.

## Minimal Bundle Flow

Run the server:

```bash
cargo run -p memd-server
```

For the shared OpenClaw deployment, you must be on Tailscale or another
private VPN/private network and set `MEMD_BASE_URL=http://100.104.154.24:8787`
before running the client commands below.

Bootstrap a project bundle:

```bash
cargo run -p memd-client --bin memd -- setup --agent codex
```

If you already have a backend URL on the current machine, export it first:

```bash
export MEMD_RAG_URL=http://10.30.30.152:9621
cargo run -p memd-client --bin memd -- setup --agent codex
```

That keeps the setup portable: each PC can point at its own private backend
without hardcoding the address into the repo.

When you run this inside a repo, `memd setup` seeds `.memd/` from existing
project docs, planning files, and memory files when it can infer a project
root. That includes `AGENTS.md`, `CLAUDE.md`, `MEMORY.md`, `memory/*.md`, and
`docs/codebase/*` when they exist. Use `--global` if you want `~/.memd` instead of
a project bundle. Use `--project-root <path>` if you want to seed a different
repo.

Check readiness:

```bash
memd status --output .memd
memd status --output .memd --summary
```

`memd status --summary` now includes the self-evolution control-plane fields,
so operators can see the current loop and gate state without opening the raw
bundle.

For two Codex tabs in the same project, set a tab ID per tab so memory stays
session-aware and tab-aware instead of collapsing into one shared turn:

```bash
export MEMD_TAB_ID=tab-a
memd status --output .memd --summary
```

Resume the compact current-task lane:

```bash
memd resume --output .memd --intent current_task
```

Hive and claim summaries now show the live tab label too, so two Codex tabs can
stay separate in the same project:

```bash
memd hive --output .memd --summary
memd claims --output .memd --summary
```

Turn a repository into an opt-in project hive with:

```bash
memd hive-project --output .memd --enable --summary
memd hive-project --output .memd --status --summary
memd hive-project --output .memd --disable --summary
```

`memd hive` still joins and publishes the live session, while `memd hive-link`
remains the manual safe link for different projects.

The generated bundle now also writes viewable memory pages under
`.memd/compiled/memory/`, with per-item drilldown under
`.memd/compiled/memory/items/`.

It also writes a live event lane under `.memd/events.md` and the compiled
event index under `.memd/compiled/events/latest.md`. That is the read-once
event compiler path: the agent records events, memd compiles them into compact
event pages, and the visible memory surfaces refresh from those compiled views.
The first live emitters are `hook capture` and `checkpoint`.

Inspect those pages directly with:

```bash
memd memory --root .memd --query working --summary
memd memory --root .memd --open working --summary
memd memory --root .memd --lane working --summary
memd memory --root .memd --item working-01-61a03407 --summary
memd memory --root .memd --list --lanes-only --summary
memd memory --root .memd --list --items-only --summary
memd memory --root .memd --list --filter working --summary
memd memory --root .memd --list --grouped
memd memory --root .memd --list --grouped --expand-items
memd memory --root .memd --list --json
memd memory --root .memd --quality --summary
```

The JSON form includes structured lane/item entries so a UI can render the
memory browser without re-parsing the markdown paths. The quality summary
scores scope, coverage, retrieval, compactness, provenance, and semantic
alignment so you can track the gap to the target bar without eyeballing pages.

Inspect the event compiler lane directly with:

```bash
memd events --root .memd --summary
memd events --root .memd --list
memd events --root .memd --query working --summary
memd events --root .memd --open working --summary
```

Inspect the raw truth spine:

```bash
cat .memd/state/raw-spine.jsonl
```

The raw spine should show source-linked records from:

- `memd remember`
- `memd checkpoint`
- `memd ingest`
- `memd hook capture`
- `memd hook spill --apply`

## Codex Harness Pack

Codex is the first harness pack in memd. The pack keeps the bundle local-first:
it reads compiled memory before the turn, captures turn output after the turn,
and refreshes the visible wake/memory files from the compiled pages.

Claude Code is the native-import harness pack. It keeps the same bundle truth,
but centers the `CLAUDE_IMPORTS.md` bridge and Claude's `/memory` verification
flow.

The command catalog is available through:

```bash
memd commands --root .memd --summary
memd commands --root .memd --json
```

The same catalog is also written to `.memd/COMMANDS.md` when you initialize a
bundle, so the `$` and `/` command surfaces stay visible next to the other
bundle artifacts.

Check setup health and repair drift when you want to see what works, what is
missing, and what still needs attention:

```bash
memd doctor --output .memd --summary
memd doctor --output .memd --json
```

That check keeps the bundle surfaces aligned and tells you what needs repair
before the next turn.

Browse the visible harness packs with:

```bash
memd packs --root .memd --summary
memd packs --root .memd --query capture --summary
memd packs --root .memd --json
```

Use these exact paths today:

```bash
memd wake --output .memd --intent current_task --write
memd resume --output .memd --intent current_task
printf 'changed auth flow: keep optimistic UI disabled for now\n' | ./integrations/hooks/memd-capture.sh
```

The Codex bundle refreshes:

- `.memd/wake.md`
- `.memd/mem.md`
- `.memd/wake.md`
- `.memd/mem.md`

Repeated reads in the same turn reuse the turn-scoped cache. If backend recall
or capture fails, memd keeps the local bundle markdown on disk and continues
from that compact truth.

OpenClaw is the second harness pack. All packs now come from the shared
preset schema. OpenClaw uses the same bundle truth, but its primary loop is
compact context before the task and spill at compaction boundaries.

Hermes is the adoption-focused harness pack. It also comes from the shared
preset schema. It uses the same bundle truth, but its primary loop is
onboarding-friendly wake, capture, and spill with cloud-first reach and
self-host later.

Agent Zero is the zero-friction harness pack. It also comes from the shared
preset schema. It uses the same bundle truth, but its primary loop is fast
resume, durable remember, clean handoff, and spill for fresh sessions.

OpenCode is the shared-lane harness pack. It also comes from the shared
preset schema. It uses the same bundle truth, but its primary loop is resume,
remember, handoff, and spill for clients that want explicit continuity
commands.

## Core Bundle Commands

Persist a durable memory:

```bash
cargo run -p memd-client --bin memd -- remember --output .memd --kind decision --content "Prefer memd resume for Codex startup."
```

Capture short-term task state:

```bash
cargo run -p memd-client --bin memd -- checkpoint --output .memd --content "Current blocker: workspace handoff still needs better ranking."
```

Emit a compact handoff:

```bash
cargo run -p memd-client --bin memd -- handoff --output .memd --prompt
```

Print the attach snippet:

```bash
cargo run -p memd-client --bin memd -- attach --output .memd
```

## Agent Profiles

Switch between clients on the same bundle with the generated scripts:

```bash
.memd/agents/codex.sh
.memd/agents/claude-code.sh
.memd/agents/agent-zero.sh
.memd/agents/openclaw.sh
.memd/agents/opencode.sh
.memd/agents/hermes.sh
```

Or inspect the generated agent metadata:

```bash
cargo run -p memd-client --bin memd -- agent --output .memd --summary
```

Switch the active bundle agent and refresh memory files:

```bash
cargo run -p memd-client --bin memd -- agent --output .memd --name claude-code --apply --summary
```

For Claude Code, import the generated bridge from your project `CLAUDE.md`:

```md
@.memd/agents/CLAUDE_IMPORTS.md
```

Then run `/memory` in Claude Code to verify the native bridge is loaded.

## Semantic Backend

Bootstrap a project bundle with LightRAG configured:

```bash
cargo run -p memd-client --bin memd -- init --project demo --agent codex --rag-url http://127.0.0.1:9000
```

For real deployments, point `--rag-url` or `MEMD_RAG_URL` at the API base of
your private backend, typically a Tailscale or VPN address. Do not use the
`/webui/` path; `memd` needs the API root that serves `/healthz`,
`/v1/ingest`, and `/v1/retrieve`.

Pull semantic fallback only when you explicitly want deeper recall:

```bash
memd resume --output .memd --intent current_task --semantic
```

When you are still building the memory loop, keep `memd` as the source of
truth and treat LightRAG as an optional semantic backend behind the control
plane.

## Obsidian

Write a shared handoff into the Obsidian workspace:

```bash
cargo run -p memd-client --bin memd -- obsidian handoff --vault ~/vault --project demo --workspace team-alpha --visibility workspace --apply --open
```

For the full vault workflow, see [docs/core/obsidian.md](./obsidian.md).

## Eval And Improvement

Evaluate the bundle-backed memory lane:

```bash
cargo run -p memd-client --bin memd -- eval --output .memd --summary
```

Persist an evaluation snapshot:

```bash
cargo run -p memd-client --bin memd -- eval --output .memd --write --summary
```

Fail fast on quality regressions:

```bash
cargo run -p memd-client --bin memd -- eval --output .memd --summary --fail-below 80
cargo run -p memd-client --bin memd -- eval --output .memd --summary --fail-on-regression
```

Find the next highest-priority gaps:

```bash
cargo run -p memd-client --bin memd -- gap --output .memd
cargo run -p memd-client --bin memd -- gap --output .memd --write --summary
```

Run the bounded improvement loop:

```bash
cargo run -p memd-client --bin memd -- improve --output .memd
cargo run -p memd-client --bin memd -- improve --output .memd --apply
cargo run -p memd-client --bin memd -- improve --output .memd --apply --max-iterations 5
```

## Inspection

Open the built-in dashboard:

```bash
cargo run -p memd-server
```

Then open `http://127.0.0.1:8787/` for a local instance, or the shared
Tailscale/private-VPN URL when you are using the hosted deployment.

Inspect the inbox:

```bash
cargo run -p memd-client --bin memd -- inbox --project demo
```

Explain one memory item:

```bash
cargo run -p memd-client --bin memd -- explain --id <uuid>
```

Search the inspiration lane:

```bash
cargo run -p memd-client --bin memd -- inspiration --query LightRAG
```

That command searches the lane source files under `.memd/lanes/inspiration/`
so you can jump back into repo inspiration without re-scanning the web.

For research-heavy work, use the Karpathy-style loop:

- keep raw inputs in `raw/`
- compile them into markdown notes/wiki pages
- view and edit through Obsidian
- run lint/health/self-repair passes so the wiki improves over time

For agent-style workflows, the newer inspiration set points at:

- prompt-owned behavior
- persistent memory
- multi-agent cooperation
- cloud-first onboarding with self-host later

Inspect the visible skill lane:

```bash
cargo run -p memd-client --bin memd -- skills --summary
```

That command shows the built-in memd entrypoints as read-only skills and any
project-local custom skills under the active bundle.

Drill into one skill:

```bash
cargo run -p memd-client --bin memd -- skills --query memd-init
```

Use that when you want the file, status, and recommended next step for one
skill instead of the whole lane.

## Verification

When you change memory behavior, rerun the reusable verification contracts instead of trusting roadmap status alone.

- Canonical feature contracts: [docs/verification/FEATURES.md](./verification/FEATURES.md)
- Audit rules: [docs/verification/AUDIT-RULES.md](./verification/AUDIT-RULES.md)
- Regression runbook: [docs/verification/RUNBOOK.md](./verification/RUNBOOK.md)
- Backward audits:
  - [docs/verification/milestones/MILESTONE-v1.md](./verification/milestones/MILESTONE-v1.md)
  - [docs/verification/milestones/MILESTONE-v2.md](./verification/milestones/MILESTONE-v2.md)
  - [docs/verification/milestones/MILESTONE-v3.md](./verification/milestones/MILESTONE-v3.md)

## Memory Atlas

The atlas is the navigation layer over canonical memory. It lets you move from
wake packet to region to node to raw evidence without searching from scratch.

### Atlas CLI

```bash
# Generate regions from existing memory
memd atlas generate --project myproject --namespace main

# List regions
memd atlas regions --project myproject

# Explore a region (shows nodes, links, trails)
memd atlas explore --region <region-uuid>

# Explore a single node with neighborhood expansion
memd atlas explore --node <memory-uuid> --depth 1

# Filter by trust or kind
memd atlas explore --region <uuid> --min-trust 0.8 --kind decision

# Compile to Obsidian vault
memd atlas compile --project myproject --vault /path/to/vault
```

### Lanes

Lanes group memory by domain across kinds. Tag memory items with `lane:<name>`
to assign them to a lane. Recognized lane names:

- `lane:design` — design decisions, aesthetic choices
- `lane:architecture` — system structure, data flow
- `lane:research` — investigation, experiments, findings
- `lane:workflow` — process, automation, tooling
- `lane:preference` — user and team preferences
- `lane:inspiration` — ideas, references, vision

Items without lane tags are grouped by kind (facts, decisions, procedures,
continuity, patterns, model).

Source paths containing lane keywords also auto-assign lanes (e.g., a memory
with `source_path` containing "design" joins the design lane).

### Regions

Regions are meaningful memory neighborhoods. They are:

- **auto-generated** from lane tags and memory kinds
- **deterministic** — regenerating produces the same region IDs
- **user-nameable** — rename via the API for human curation

### Trails

Trails are ordered paths through nodes in a region:

- **salience** trail: highest confidence first
- **zoom** trail: shallowest depth first (core to periphery)

Minimum rerun loop after a memory-path change:

```bash
cargo test -p memd-server --quiet
cargo test -p memd-client --quiet
```
