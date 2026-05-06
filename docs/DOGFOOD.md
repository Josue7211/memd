# memd Dogfood

Use this when you are helping test `memd` on a real machine.

## Install

From a memd checkout:

```bash
scripts/install-memd.sh
```

Then enroll this machine:

```bash
memd dogfood enroll --user-id <your-name> --consent --summary
```

## Daily Use

Use your normal agent workflow. When something feels wrong, run:

```bash
memd doctor --output .memd --summary
memd dogfood status --output .memd --summary
```

Send the output plus what you expected to happen.

## Evidence Rules

- Real use only.
- No private content in bug reports unless you choose to share it.
- Keep the same machine enrolled for the whole window.
- Add every extra machine with `memd device add --summary`.

## Gates

- 3 real users
- 3 harness-user pairs
- 3 devices on current `main`
- weekly evidence review notes
