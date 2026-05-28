# Setup Troubleshooting

Secondary/reference doc. Start from [[ROADMAP]] for project truth, or README Quickstart for first install.

Use the first matching symptom. Each fix is safe from the memd checkout unless marked otherwise.

## `cargo: command not found`

Cause: Rust is missing.

Fix: install Rust from <https://rustup.rs/>, then open a new shell.

Verify:

```bash
cargo --version
scripts/install-memd.sh
```

## `memd: command not found` after install

Cause: `~/.local/bin` is not on PATH.

Fix:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Verify:

```bash
command -v memd
memd doctor --summary
```

## Interactive setup does not open

Cause: shell is non-interactive or stdin is not a TTY.

Fix: use the non-interactive path:

```bash
memd setup --summary --agent codex
```

Verify:

```bash
memd doctor --summary
```

## `.memd` is missing

Cause: bundle initialization did not run or ran from the wrong directory.

Fix:

```bash
memd setup --summary
```

Verify:

```bash
test -d .memd
memd status --output .memd --summary
```

## `memd doctor --summary` is red

Fix safe drift:

```bash
memd doctor --repair --summary
```

Verify:

```bash
memd doctor --summary
```

## Permission denied writing `.memd`

Inspect ownership first:

```bash
ls -ld . .memd .memd/mem.md .memd/wake.md 2>/dev/null
```

Do not run `sudo chmod -R 777`.

## Still blocked

Capture:

```bash
memd doctor --summary
memd status --output .memd --summary
scripts/verify/setup-experience-smoke.sh
```

Do not paste secrets.
