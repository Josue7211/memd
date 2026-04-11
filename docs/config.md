# Config Guide

`memd` setup should be mostly CLI-driven. The bundle files exist so agents and
other tools can consume stable values, not so you have to hand-edit everything.

## Files That Matter

- `config.json`
  - canonical bundle defaults
- `env`
  - shell exports for Unix-like environments
- `env.ps1`
  - shell exports for PowerShell
- `backend.env`
  - backend-specific exports such as semantic memory settings
- `state/last-resume.json`
  - last local resume snapshot
- `state/heartbeat.json`
  - latest published short-term presence snapshot

## What To Change Through The CLI

Prefer commands over editing `config.json` by hand:

- initialize a bundle:
  - `memd setup --project demo --namespace main --agent codex`
- switch the active agent:
  - `memd agent --output .memd --name claude-code --apply --summary`
- set or change the semantic backend:
  - rerun `memd setup ... --rag-url <url>` for new bundles
  - or update the bundle with the dedicated bundle-setting command when available in your workflow
- inspect readiness:
  - `memd status --output .memd`

## Heartbeat Model

Bundles now carry a dedicated `heartbeat_model` separate from the main agent
model. This is the cheap coordination lane used for:

- presence / heartbeat sync
- shared short-term cowork awareness
- low-cost cross-harness status updates

The bundle defaults it to `llama-desktop/qwen` as the lightweight heartbeat
preference currently used in the adjacent OpenClaw runtime path. Treat this as
the 0.8B-class lane, not the primary reasoning lane.

## What `status` Should Tell You

Use `memd status --output .memd` before editing config manually.

It now tells you:

- whether the setup is ready
- which bundle files are missing
- whether the backend is reachable
- whether the hot resume lane is returning useful local state
- which heartbeat model the bundle will publish through shared presence

## When Manual Edits Are Reasonable

Manual edits are acceptable when you are:

- repairing a broken local bundle
- scripting bundle generation outside the CLI
- debugging a config parsing issue

If you are doing normal setup, prefer the CLI. The intent is that `memd`
configuration should feel small even if the underlying bundle exports several
files for different agents and shells.
