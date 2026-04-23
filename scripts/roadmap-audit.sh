#!/usr/bin/env bash
# Fail if any docs/backlog/**/*.md with status: open has a phase: that doesn't
# resolve to a live phase doc. Allows `status: deferred` as an explicit escape
# hatch (per phase-a3 Pass Gate wording: "assigned OR explicitly marked deferred").

set -euo pipefail

REPO_ROOT="${REPO_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
BACKLOG_DIR="$REPO_ROOT/docs/backlog"
PHASES_DIR="$REPO_ROOT/docs/phases"

# Live phase codes
declare -A LIVE_PHASES
while IFS= read -r pf; do
    [ -f "$pf" ] || continue
    code=$(awk '/^phase:/{sub(/^phase:[[:space:]]*/,""); print; exit}' "$pf")
    [ -n "$code" ] && LIVE_PHASES["$code"]=1
done < <(find "$PHASES_DIR" -maxdepth 3 -name 'phase-*.md' 2>/dev/null)

fail=0
total_open=0
deferred=0
problems=()

while IFS= read -r f; do
    status=$(awk 'NR==1 && /^---$/{fm=1;next} fm && /^---$/{exit} fm && /^status:/{sub(/^status:[[:space:]]*/,""); print; exit}' "$f")
    phase=$(awk 'NR==1 && /^---$/{fm=1;next} fm && /^---$/{exit} fm && /^phase:/{sub(/^phase:[[:space:]]*/,""); print; exit}' "$f")

    [ "$status" = "closed" ] && continue
    [ "$status" = "in_progress" ] && status="open"  # treat in_progress as open

    if [ "$status" = "deferred" ]; then
        deferred=$((deferred+1))
        continue
    fi

    total_open=$((total_open+1))

    if [ -z "$phase" ] || [ "$phase" = "unassigned" ]; then
        problems+=("$f: open item has phase='${phase:-<empty>}' — assign a live phase or mark status: deferred")
        fail=1
        continue
    fi

    if [ -z "${LIVE_PHASES[$phase]:-}" ]; then
        problems+=("$f: phase '$phase' does not resolve to a live phase doc under $PHASES_DIR/")
        fail=1
    fi
done < <(find "$BACKLOG_DIR" -maxdepth 3 -name '*.md' -not -name 'INDEX.md' -not -name 'TEMPLATE.md')

if [ "$fail" = "1" ]; then
    echo "roadmap-audit: FAIL" >&2
    for p in "${problems[@]}"; do echo "  - $p" >&2; done
    exit 1
fi

echo "roadmap-audit: ok — $total_open open items, all assigned to live phases ($deferred deferred)"
