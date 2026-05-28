# Interactive Setup

Secondary/reference doc. Start from [[ROADMAP]] for project truth, or README Quickstart for first install.

`memd setup --interactive` is the guided setup surface.

## What it feels like

```text
                  memd setup

        Pick a provider

          › Local only
            Shared memd server
            Custom MEMD_BASE_URL

        ↑/↓ move   Enter select   q quit
```

Then it asks for harnesses:

```text
                  memd setup

        Pick harness

          › Codex
            Claude Code
            Hermes
            OpenClaw
            OpenCode
            Done

        ↑/↓ move   Enter toggle/select   q quit
```

## Providers

- Local only: best first install, no network assumptions.
- Shared memd server: use configured shared authority when available.
- Custom `MEMD_BASE_URL`: for private server deployments.

## Harnesses

Pick the agent surfaces you actually use:

- Codex
- Claude Code
- Hermes
- OpenClaw
- OpenCode

Setup writes normal `.memd` bundle files and then tells you the next proof commands.

## Non-interactive fallback

CI and scripts should keep using flags:

```bash
memd setup --summary --agent codex
```
