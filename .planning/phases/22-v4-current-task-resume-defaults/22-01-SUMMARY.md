# Phase 22 Summary: `v4` Current-Task Resume Defaults

## Completed

- updated attach snippets to use `--intent current_task`
- updated generated agent shell and PowerShell profiles the same way
- updated README and generated bundle docs to reflect the new default
- added regression coverage for current-task launch intent

## Verification

- `cargo test -q -p memd-client`
- `cargo test -q`

## Outcome

Short-term memory is now the default launch behavior across the bundle surface,
not just a capability the operator has to remember to ask for.
