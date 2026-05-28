# Install memd

Secondary/reference doc. Start from [[ROADMAP]] for project truth, or README Quickstart for first install.

## Before you start

You need a memd checkout and Rust/Cargo for source install.

```bash
cargo --version
```

If that fails, install Rust from <https://rustup.rs/> and open a new shell.

## Install

```bash
scripts/install-memd.sh
```

Expected end state:

```text
memd install: ready
memd install: next proof: run 'memd setup --interactive' or 'memd doctor --summary'
```

## Configure

For the guided product path:

```bash
memd setup --interactive
```

Use arrow keys to move through centered choices. Press Enter to pick provider and harness options.

## Verify

```bash
command -v memd
memd doctor --summary
memd status --output .memd --summary
```
