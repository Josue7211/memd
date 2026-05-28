# First Run

Secondary/reference doc. Start from [[ROADMAP]] for project truth, or README Quickstart for first install.

## 1. Configure

```bash
memd setup --interactive
```

Pick provider and harnesses with arrow keys and Enter.

## 2. Health check

```bash
memd doctor --summary
```

## 3. Status check

```bash
memd status --output .memd --summary
```

## 4. Resume check

```bash
memd resume --output .memd --intent current_task
```

Done means `memd` is on PATH, `.memd` exists, doctor/status/resume run, and selected harness docs were generated.
