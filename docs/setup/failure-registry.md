# Setup Failure Registry

Secondary/reference doc. Start from [[ROADMAP]] for project truth, or README Quickstart for first install.

Use this table when `memd doctor --summary` is red or setup feels confusing.

| Code | Symptom | Cause | Fix | Verify |
| --- | --- | --- | --- | --- |
| SETUP-PATH-001 | `memd: command not found` | install prefix not on PATH | `export PATH="$HOME/.local/bin:$PATH"` | `command -v memd` |
| SETUP-RUST-001 | `cargo: command not found` | Rust missing | install from <https://rustup.rs/> then open a new shell | `cargo --version` |
| SETUP-BUNDLE-001 | `.memd` missing | setup not run or wrong directory | `memd setup --summary --force` | `test -f .memd/config.json` |
| SETUP-BUNDLE-002 | bundle exists but status not ready | missing generated files or stale config | `memd doctor --repair --summary` | `memd doctor --summary` |
| SETUP-TTY-001 | interactive screen cannot open | non-TTY shell | `memd setup --guided --summary` then `memd setup --summary --agent codex` | `memd status --output .memd --summary` |
| SETUP-SERVER-001 | server red/unreachable | memd server not running or wrong base URL | start server or set `MEMD_BASE_URL`; local setup can still use bundle proof | `memd healthz` or `memd doctor --summary` |
| SETUP-PERM-001 | permission denied writing `.memd` | repo or bundle owned by another user | inspect `ls -ld . .memd`; fix ownership narrowly | `memd setup --summary --force` |
| SETUP-HARNESS-001 | harness bridge missing | harness CLI/config not present | install target harness or use guided setup for another harness | `memd status --output .memd --summary` |
| SETUP-PRIVACY-001 | user does not know where data lives | docs skipped | read `docs/setup/data-and-privacy.md` | user can explain local bundle + backend |
| SETUP-UPDATE-001 | fear of losing memory on update | update path unclear | `scripts/update-memd.sh --dry-run` then normal update | `.memd/config.json` still exists |
| SETUP-UNINSTALL-001 | fear uninstall deletes memory | uninstall path unclear | `scripts/uninstall-memd.sh --dry-run` | output says memory preserved |

If a new setup failure appears, add it here before claiming local 25-star again.
