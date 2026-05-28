# memd Setup

Secondary/reference doc. Start from [[ROADMAP]] for project truth, or README Quickstart for first install.

This is the beginner setup lane. It gets memd installed, configured, checked, and usable before internals.

## Best path

```bash
scripts/install-memd.sh
memd setup --guided --summary
memd setup --interactive
memd doctor --summary
memd status --output .memd --summary
memd setup-demo --summary
```

`memd setup --interactive` is the Hermes/OpenClaw-style setup surface: centered choices, arrow keys to move, Enter to select providers and harnesses.

## Read by goal

| Goal | Doc |
| --- | --- |
| Install from checkout | [Install](install.md) |
| Pick providers and harnesses | [Interactive Setup](interactive.md) |
| Verify first use | [First Run](first-run.md) |
| Fix an error | [Troubleshooting](troubleshooting.md) |
| Update safely | [Update](update.md) |
| Remove the binary safely | [Uninstall](uninstall.md) |
| Understand data/privacy | [Data and Privacy](data-and-privacy.md) |

## 10-star setup means

- Fresh developer starts from README.
- Installer works from checkout.
- Interactive setup lets them pick providers and harnesses without memorizing flags.
- Doctor gives exact next fixes.
- Status/resume prove the bundle works.
- `memd setup-demo --summary` proves a temp first run without touching repo memory.
- No live handholding on the happy path.
