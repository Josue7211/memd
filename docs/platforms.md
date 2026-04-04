# Platform Support

`memd` is intended to run on:

- Linux
- macOS
- Windows

## What Is Cross-Platform

- `memd-server`
- `memd-client`
- `memd-worker`
- the shared schema and compaction logic
- the HTTP API

## What Is Platform-Specific

- `deploy/systemd/` is Linux-only deployment glue
- macOS should use `launchd` or a user agent wrapper
- Windows should use a service wrapper or Task Scheduler

## Packaging Rule

The core product must stay platform-neutral.
Platform-specific helpers are allowed, but they live beside the core, not inside it.

## CI Rule

GitHub Actions should validate the core on:

- Ubuntu
- macOS
- Windows

That keeps accidental Linux-only assumptions from creeping back in.
