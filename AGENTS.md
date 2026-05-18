# AGENTS.md

<!-- memd-managed:start -->
These instructions are managed by memd.

## memd voice bootstrap

- Treat `.memd/config.json` as the source of truth for this repo's active `voice_mode`.
- Valid repo voice modes are `normal`, `caveman-lite`, `caveman-full`, `caveman-ultra`, `wenyan-lite`, `wenyan-full`, and `wenyan-ultra`.
- If the user asks which voice is active, answer from `.memd/config.json`.
- Do not tell the user to manually enable a voice that `.memd/config.json` already sets.
- Do not invent a second source of truth for voice mode.
- Do not slip from the repo voice mode; stay in `caveman-ultra` unless `.memd/config.json` changes.
- Caveman modes mean compressed wording, not broken spelling.
- Keep normal spelling and exact technical terms even when voice mode is `caveman-lite` or `caveman-ultra`.
- Reply style is derived from config. If your draft is not in `caveman-ultra`, stop and rewrite it before sending.

## current repo default

- The current bundle file `.memd/config.json` sets `voice_mode` to `caveman-ultra`.
- Until that bundle setting changes, use `caveman-ultra` by default in this repo.

## memd runtime

- memd is the memory/bootstrap dependency for this repo.
- Treat memd bundle state as startup truth before answering.
- Start from `.memd/wake.md` before relying on transcript recall.
- Use `.memd/mem.md` for the deeper compact memory view.
- Use `.memd/events.md` for the event log.
- Durable truth beats transcript recall.
- For decisions, preferences, project history, or prior corrections, run `memd lookup --output .memd --query "..."` before answering.
- Use `memd hook spill --output .memd --stdin --apply` at compaction boundaries to turn turn-state deltas into durable candidate memory.
- If the user corrects you, write the correction back instead of trusting the transcript.
- Keep responses short, direct, and token-efficient unless the user asks for detail.

<!-- memd-managed:end -->
## shared dev-server guard

- Before starting any webapp/dev server, check whether the intended port is already listening.
- Use `scripts/dev-server-guard.sh --port <port> -- <command...>` instead of raw `npm run dev`, `astro dev`, `vite`, or similar.
- If the guard reports an existing server, reuse that URL. Do not start another server on a new port unless the user asks for a second instance.
- The guard uses the Rust CLI lease `resource:dev-server:<repo-hash>:127.0.0.1:<port>` so other agents can see the running-server footstep.
- Release happens automatically when the guarded command exits.
- Conflicts are hard-blocking. Treat raw dev-server launch commands as unsafe unless the user explicitly asks for another instance.
- Contract: `docs/contracts/dev-server-guard.md`.

## memd cargo isolation

- For memd builds/tests/proof scripts, use `scripts/memd-cargo-guard.sh -- <cargo args>` or source `scripts/lib/memd-cargo-env.sh` before invoking Cargo.
- Do not use raw Cargo for memd work when another repo/app can be running. memd must use its own Cargo home/target so ClawControl and other projects do not block on memd package-cache locks.
- If Cargo work appears blocked, check hive/awareness status first and report the collision instead of killing unrelated processes.

## memd live map and hive collision guard

- Treat `.memd/state/codebase-live-map.json` and `.memd/state/codebase-live-map-events.ndjson` as the live codebase map/diff surface.
- Hook/file-interaction paths must be recorded as live-map events so other agents can see edits before the next heartbeat.
- Before broad Git, Cargo, test, or repo-scan work, run `scripts/memd-host-io-guard.sh`. Exit `75` means same-repo/filesystem/unknown host I/O is blocked; do not keep piling on work.
- Sibling project I/O on the same volume is awareness, not a default hard stop. Do not kill or restart ClawControl, AgentShell, or another repo to make memd compile.
- The host guard bounds `ps` with `MEMD_HOST_IO_PS_TIMEOUT_SECS` and treats `project_hint=host-process-scan state=timeout` as a blocker, not a clear signal.
- Rust awareness bounds host process scans with `MEMD_HOST_PROCESS_SCAN_TIMEOUT_MS`; `host_process_scan_timeout` is also a hive-blocking diagnostic.
- `project_hint=app-git`, `cargo-tooling`, `native-tooling`, or `node-tooling` means a host/app-owned tool is blocking the shared volume; coordinate or wait, do not kill unrelated apps as a memd workaround.
- The guard writes `.memd/state/host-io-guard.txt`; use it as the last cheap blocker snapshot during handoff/status checks.
- Run `scripts/memd-continuity-status.sh` for a cheap continuity packet over `wake.md`, deploy preflight, live map, and host guard before doing expensive recall or repo scans.
- Fresh blocked host reports are reused read-only for `MEMD_HOST_IO_REPORT_TTL_SECS` seconds. `project_hint=host-io-report state=cached` means the blocker came from that snapshot; it must not refresh the report timestamp by itself.
- `memd lookup` remote recall is bounded by `MEMD_LOOKUP_REMOTE_TIMEOUT_MS` and should fall back to local continuity from `wake.md`, `.memd/state/codebase-live-map.json`, and `.memd/state/host-io-guard.txt`; do not let lookup hang forever before reporting live blockers.
- If `.memd/state/codebase-live-map.json` says `status=blocked` or `reread_required=true`, treat that as current shared state. Do not reread the whole repo until host guard is clear.
- `scripts/deploy-memd-server-preflight.sh` emits `MEMD_CODEBASE_LIVE_MAP_FRESH`, `MEMD_CODEBASE_LIVE_MAP_AGE_SECS`, `MEMD_CODEBASE_LIVE_MAP_TTL_SECS`, and `MEMD_CODEBASE_LIVE_MAP_ACTION`. Follow `MEMD_CODEBASE_LIVE_MAP_ACTION` before trusting or rereading codebase state.
- `scripts/deploy-memd-server-preflight.sh` must not run local Git on `/Volumes/...` without a fresh clear host report. If the report is missing, stale, or blocked, `MEMD_GIT_DIRTY=unknown` with `MEMD_GIT_STATUS_BLOCKERS` is the correct safe result.
- `MEMD_HOST_IO_GUARD=0` is the intentional override for rare manual recovery work; do not set it for normal agent builds/tests.
- If awareness reports `codebase_live_map status=blocked`, `host_process_blocked`, `host_process_scan_timeout`, `host_filesystem_blocked`, or `awareness_scan_skipped`, stop broad repo scans, Git, and Cargo work for same-repo/filesystem/unknown blockers. Report sibling project hints as awareness and coordinate with the owning agent without treating them as memd build blockers.
