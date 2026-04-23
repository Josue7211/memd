#!/usr/bin/env bash
# Fail if:
#  - any backlog file has no YAML frontmatter
#  - any backlog file has no `phase:` key
#  - any `phase:` value fails to resolve to a live phase doc (closed/ exempt)
#  - any open item has `phase: unassigned` (phase-A3 coverage gate)
#  - docs/backlog/INDEX.md is stale vs scripts/backlog-index.sh output

set -euo pipefail

BACKLOG_DIR="${BACKLOG_DIR:-docs/backlog}"
PHASES_DIR="${PHASES_DIR:-docs/phases}"

fail=0
problems=()

# 1. Frontmatter + phase: presence
while IFS= read -r f; do
    if ! head -1 "$f" | grep -q '^---$'; then
        problems+=("$f: missing YAML frontmatter")
        fail=1
        continue
    fi
    if ! awk 'NR==1 && /^---$/{fm=1;next} fm && /^---$/{exit} fm && /^phase:/{found=1} END{exit !found}' "$f"; then
        problems+=("$f: missing phase: key")
        fail=1
    fi
done < <(find "$BACKLOG_DIR" -maxdepth 3 -name '*.md' -not -name 'INDEX.md' -not -name 'TEMPLATE.md')

# 2. phase: value resolution — must be "unassigned" OR a live phase code.
# Build set of live phase codes from phase docs.
declare -A LIVE_PHASES
while IFS= read -r pf; do
    [ -f "$pf" ] || continue
    code=$(awk '/^phase:/{sub(/^phase:[[:space:]]*/,""); print; exit}' "$pf")
    [ -n "$code" ] && LIVE_PHASES["$code"]=1
done < <(find "$PHASES_DIR" -maxdepth 3 -name 'phase-*.md')

while IFS= read -r f; do
    phase=$(awk 'NR==1 && /^---$/{fm=1;next} fm && /^---$/{exit} fm && /^phase:/{sub(/^phase:[[:space:]]*/,""); print; exit}' "$f")
    [ -z "$phase" ] && continue
    # closed/ items are historical; coverage gate does not apply
    case "$f" in */closed/*) continue ;; esac
    if [ "$phase" = "unassigned" ]; then
        problems+=("$f: open item has phase: unassigned — assign a phase or move to closed/ (A3 coverage gate)")
        fail=1
        continue
    fi
    if [ -z "${LIVE_PHASES[$phase]:-}" ]; then
        problems+=("$f: phase '$phase' does not resolve to a live phase doc")
        fail=1
    fi
done < <(find "$BACKLOG_DIR" -maxdepth 3 -name '*.md' -not -name 'INDEX.md' -not -name 'TEMPLATE.md')

# 3. INDEX.md freshness
if [ -f "$BACKLOG_DIR/INDEX.md" ]; then
    expected=$(mktemp)
    trap 'rm -f "$expected"' EXIT
    BACKLOG_DIR="$BACKLOG_DIR" bash scripts/backlog-index.sh --out "$expected" >/dev/null
    if ! diff -q "$BACKLOG_DIR/INDEX.md" "$expected" >/dev/null; then
        problems+=("$BACKLOG_DIR/INDEX.md is stale — run \`make backlog-index\` and commit the result")
        fail=1
    fi
else
    problems+=("$BACKLOG_DIR/INDEX.md does not exist — run \`make backlog-index\`")
    fail=1
fi

if [ "$fail" = "1" ]; then
    echo "backlog-lint: FAIL" >&2
    for p in "${problems[@]}"; do
        echo "  - $p" >&2
    done
    exit 1
fi

total=$(find "$BACKLOG_DIR" -maxdepth 3 -name '*.md' -not -name 'INDEX.md' -not -name 'TEMPLATE.md' | wc -l)
echo "backlog-lint: ok — $total items; open items map to live phase docs (closed/ exempt); INDEX.md fresh"
