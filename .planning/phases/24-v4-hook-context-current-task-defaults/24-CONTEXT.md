# Phase 24 Context: `v4` Hook Context Current-Task Defaults

## Why This Phase Exists

The bundle launch surfaces already default to current-task intent, but the
installed hook-context path still defaulted to a more generic intent. That left
one of the most common integration paths behind the new short-term standard.

This phase brings hook-context onto the same current-task default.

## Inputs

- Phase 22: current-task resume defaults
- Phase 23: status-preview alignment
- integrations hook kit

## Constraints

- keep hook retrieval local and fast
- do not reintroduce semantic fallback
- align CLI defaults and shell-script defaults together

## Target Outcome

`memd hook context` and the installed hook scripts should default to
`current_task` intent.
