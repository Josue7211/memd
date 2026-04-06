# Setup Guide

This is the longer setup and usage path for `memd`. The README only keeps the
minimal happy path.

## Minimal Bundle Flow

Run the server:

```bash
cargo run -p memd-server
```

For the shared OpenClaw deployment, set `MEMD_BASE_URL=http://100.104.154.24:8787`
before running the client commands below.

Bootstrap a project bundle:

```bash
cargo run -p memd-client --bin memd -- init --project demo --namespace main --agent codex
```

Check readiness:

```bash
cargo run -p memd-client --bin memd -- status --output .memd
```

Resume the compact current-task lane:

```bash
cargo run -p memd-client --bin memd -- resume --output .memd --intent current_task
```

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
.memd/agents/openclaw.sh
.memd/agents/opencode.sh
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

Pull semantic fallback only when you explicitly want deeper recall:

```bash
cargo run -p memd-client --bin memd -- resume --output .memd --intent current_task --semantic
```

When you are still building the memory loop, keep `memd` as the source of
truth and treat LightRAG as an optional semantic backend behind the control
plane.

## Obsidian

Write a shared handoff into the Obsidian workspace:

```bash
cargo run -p memd-client --bin memd -- obsidian handoff --vault ~/vault --project demo --workspace team-alpha --visibility workspace --apply --open
```

For the full vault workflow, see [docs/obsidian.md](./obsidian.md).

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

Then open `http://127.0.0.1:8787/`.

Inspect the inbox:

```bash
cargo run -p memd-client --bin memd -- inbox --project demo
```

Explain one memory item:

```bash
cargo run -p memd-client --bin memd -- explain --id <uuid>
```
