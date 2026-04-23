#!/usr/bin/env bash
# Resolve every [[wikilink]] in docs/**/*.md and .memd/**/*.md (excluding .memd/hooks/*).
# Wiki-links resolved by basename (path-agnostic): [[name]] matches any file
# named "name.md" or any directory named "name" anywhere under the roots.
#
# Broken links → report + exit 1.
# An allowlist at scripts/lint-links.allowlist (one token per line) is exempt.

set -euo pipefail

DOC_ROOT="${DOC_ROOT:-.}"
ALLOWLIST="${LINT_LINKS_ALLOWLIST:-scripts/lint-links.allowlist}"

# Roots to scan
declare -a SCAN_ROOTS=(
    "$DOC_ROOT/docs"
    "$DOC_ROOT/ROADMAP.md"
    "$DOC_ROOT/START-HERE.md"
    "$DOC_ROOT/README.md"
    "$DOC_ROOT/AGENTS.md"
    "$DOC_ROOT/CLAUDE.md"
)

# Roots to match targets against (basename-matchable)
declare -a MATCH_ROOTS=(
    "$DOC_ROOT/docs"
    "$DOC_ROOT/ROADMAP.md"
    "$DOC_ROOT/START-HERE.md"
    "$DOC_ROOT/README.md"
    "$DOC_ROOT/AGENTS.md"
    "$DOC_ROOT/CLAUDE.md"
    "$DOC_ROOT/.memd/docs"
    "$DOC_ROOT/.memd/lanes"
    "$DOC_ROOT/.memd/agents"
)

# Build basename → exists map
declare -A EXISTS
for r in "${MATCH_ROOTS[@]}"; do
    [ -e "$r" ] || continue
    if [ -f "$r" ]; then
        base=$(basename "$r")
        EXISTS["$base"]=1
        EXISTS["${base%.md}"]=1
    else
        while IFS= read -r p; do
            base=$(basename "$p")
            EXISTS["$base"]=1
            EXISTS["${base%.md}"]=1
        done < <(find "$r" -type f -name '*.md' -o -type d)
    fi
done

# Allowlist
declare -A ALLOWED
if [ -f "$ALLOWLIST" ]; then
    while IFS= read -r line; do
        [ -z "$line" ] && continue
        [[ "$line" =~ ^# ]] && continue
        ALLOWED["$line"]=1
    done < "$ALLOWLIST"
fi

broken=0
problems=()

# Extract wiki-link tokens and check each
while IFS= read -r source; do
    [ -f "$source" ] || continue
    # Find all [[...]] tokens with line numbers
    while IFS=: read -r lineno rawtoken; do
        # Strip outer [[ ]]
        token=$(echo "$rawtoken" | sed -E 's/.*\[\[([^]]+)\]\].*/\1/')
        # Strip |display
        token="${token%%|*}"
        # Strip #anchor
        token="${token%%#*}"
        # Strip trailing spaces
        token=$(echo "$token" | sed 's/[[:space:]]*$//;s/^[[:space:]]*//')
        [ -z "$token" ] && continue

        # Skip external URLs
        if [[ "$token" =~ ^https?:// ]]; then continue; fi
        if [[ "$token" =~ ^/ ]]; then continue; fi

        # Allowlist
        if [ -n "${ALLOWED[$token]:-}" ]; then continue; fi

        # Try path-based match first (for tokens like "docs/foo/bar.md")
        if [ -e "$DOC_ROOT/$token" ] || [ -e "$DOC_ROOT/$token.md" ]; then continue; fi

        # Try basename match
        base=$(basename "$token")
        if [ -n "${EXISTS[$base]:-}" ] || [ -n "${EXISTS[${base%.md}]:-}" ]; then continue; fi

        problems+=("broken: $source:$lineno: [[$token]]")
        broken=$((broken+1))
    done < <(grep -nE '\[\[[^]]+\]\]' "$source" 2>/dev/null | awk -F: '{print $1 ":" substr($0, index($0,$2))}')
done < <(
    for r in "${SCAN_ROOTS[@]}"; do
        [ -e "$r" ] || continue
        if [ -f "$r" ]; then echo "$r"; else find "$r" -type f -name '*.md'; fi
    done
)

if [ "$broken" -gt 0 ]; then
    echo "lint-links: FAIL — $broken broken links" >&2
    for p in "${problems[@]}"; do echo "  - $p" >&2; done
    exit 1
fi

echo "lint-links: ok"
