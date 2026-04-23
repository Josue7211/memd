#!/usr/bin/env bash
# Migrate docs/backlog/**/*.md from body-bullet metadata to YAML frontmatter.
#
# Idempotent: files already starting with `---\n` are skipped.
# Preserves body bullets (human-readable mirror).
# Normalizes legacy V2-* phase codes to V3 phase codes.
#
# Usage:
#   scripts/backlog-frontmatter-migrate.sh [--dry-run] [<file>...]
#   scripts/backlog-frontmatter-migrate.sh --apply
#
# Exits 0 on success. Exits non-zero if any legacy phase value cannot be mapped.

set -euo pipefail

DRY_RUN=1
TARGETS=()

for arg in "$@"; do
    case "$arg" in
        --apply) DRY_RUN=0 ;;
        --dry-run) DRY_RUN=1 ;;
        *) TARGETS+=("$arg") ;;
    esac
done

if [ "${#TARGETS[@]}" -eq 0 ]; then
    mapfile -t TARGETS < <(find docs/backlog -maxdepth 3 -name '*.md' -not -name 'INDEX.md' -not -name 'TEMPLATE.md' -not -name 'PART3-AUDIT.md')
fi

# Build reverse-lookup: slug -> owning phase code (from docs/phases/**/*.md frontmatter).
# A3 overrides other claims (2026-04-17 directive).
declare -A SLUG_OWNER

build_ownership() {
    local preferred_order=(a3 b3 c3 d3 e3 f3 i2 m2 n2 j2 k2 l2 o2 p2 b2 c2 d2 e2 f2 g2 h2 a2)
    for stem in "${preferred_order[@]}"; do
        for pf in docs/phases/phase-${stem}-*.md; do
            [ -f "$pf" ] || continue
            local phase_code
            phase_code=$(awk '/^phase:/{sub(/^phase:[[:space:]]*/,""); print; exit}' "$pf")
            [ -z "$phase_code" ] && continue
            # Pull backlog_items list
            while IFS= read -r line; do
                # Line looks like: `  - "2026-04-17-slug"` — extract quoted slug
                local slug
                slug=$(echo "$line" | sed -n 's/^[[:space:]]*-[[:space:]]*"\([^"]*\)".*/\1/p')
                [ -z "$slug" ] && continue
                # Only set if not already claimed (preferred_order ensures A3 wins)
                if [ -z "${SLUG_OWNER[$slug]:-}" ]; then
                    SLUG_OWNER[$slug]="$phase_code"
                fi
            done < <(awk '/^backlog_items:/{flag=1;next} /^[a-z_]+:/{flag=0} /^---$/{flag=0} flag' "$pf")
        done
    done
}

build_ownership

normalize_phase() {
    local raw="$1"
    local cleaned
    cleaned=$(echo "$raw" | sed -e 's/`//g' -e "s/'//g" -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')

    # Substring-match legacy V2-X codes, V3 A3/B3/etc, and prose variants
    if echo "$cleaned" | grep -qE 'V2-M2-evo|V2-M2[^0-9]|V2-m2-evo'; then echo "M2-evo"; return; fi
    if echo "$cleaned" | grep -qE 'V2-N2|V2-n2'; then echo "N2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-K2|V2-k2'; then echo "K2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-L2|V2-l2'; then echo "L2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-I2|V2-i2'; then echo "I2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-J2|V2-j2'; then echo "J2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-O2|V2-o2'; then echo "O2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-P2|V2-p2'; then echo "P2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-B2|V2-b2'; then echo "B2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-C2|V2-c2'; then echo "C2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-D2|V2-d2'; then echo "D2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-E2|V2-e2'; then echo "E2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-F2|V2-f2'; then echo "F2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-G2|V2-g2'; then echo "G2"; return; fi
    if echo "$cleaned" | grep -qE 'V2-H2|V2-h2'; then echo "H2"; return; fi
    if echo "$cleaned" | grep -qE '\bA3\b'; then echo "A3"; return; fi
    if echo "$cleaned" | grep -qE '\bB3\b'; then echo "B3"; return; fi
    if echo "$cleaned" | grep -qE '\bC3\b'; then echo "C3"; return; fi
    if echo "$cleaned" | grep -qE '\bD3\b'; then echo "D3"; return; fi
    if echo "$cleaned" | grep -qE '\bE3\b'; then echo "E3"; return; fi
    if echo "$cleaned" | grep -qE '\bF3\b'; then echo "F3"; return; fi
    if echo "$cleaned" | grep -qE 'Phase I2|\bPhaseI2\b|Phase I\b|\bI2\b|Phase H\b'; then echo "I2"; return; fi
    if echo "$cleaned" | grep -qiE 'Procedural Learning|Phase G'; then echo "unassigned"; return; fi
    if [ -z "$cleaned" ] || [ "$cleaned" = "core" ] || [ "$cleaned" = "dogfood" ] || [ "$cleaned" = "V2" ]; then
        echo "unassigned"; return
    fi
    # Fallback: try first phase-code-shaped token
    local token
    token=$(echo "$cleaned" | grep -oE '\b[A-Z][0-9](-[a-z0-9]+)?\b' | head -1 || true)
    if [ -n "$token" ]; then
        echo "$token"
    else
        echo "unassigned"
    fi
}

extract_field() {
    local file="$1"
    local key="$2"
    # Match either `- key: \`val\`` or `key: val` or `Key: val`
    local val
    val=$(awk -v key="$key" '
        BEGIN { IGNORECASE=1 }
        $0 ~ "^-[[:space:]]+"key":" { sub(/^-[[:space:]]+[^:]+:[[:space:]]*/, ""); print; exit }
        $0 ~ "^"key":" { sub(/^[^:]+:[[:space:]]*/, ""); print; exit }
    ' "$file" 2>/dev/null || true)
    # Strip backticks
    echo "$val" | sed 's/`//g' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//'
}

migrate_one() {
    local f="$1"

    # Already has YAML frontmatter? Skip.
    if head -1 "$f" | grep -q '^---$'; then
        return 0
    fi

    local status severity phase opened scope
    status=$(extract_field "$f" "status")
    severity=$(extract_field "$f" "severity")
    phase_raw=$(extract_field "$f" "phase")
    opened=$(extract_field "$f" "opened")
    scope=$(extract_field "$f" "scope")

    # Alternate field names
    if [ -z "$status" ]; then
        # Some use "Status:" prose
        status=$(extract_field "$f" "Status")
    fi
    if [ -z "$opened" ]; then
        opened=$(extract_field "$f" "found")
        [ -z "$opened" ] && opened=$(extract_field "$f" "Created")
        [ -z "$opened" ] && opened=$(extract_field "$f" "created")
    fi
    if [ -z "$phase_raw" ]; then
        phase_raw=$(extract_field "$f" "Phase")
    fi

    # Defaults
    [ -z "$status" ] && status="open"
    [ -z "$severity" ] && severity="medium"
    [ -z "$opened" ] && opened=$(basename "$f" | sed 's/^\([0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}\).*/\1/')

    # Normalize status
    local status_norm
    case "$status" in
        open) status_norm=open ;;
        closed|done) status_norm=closed ;;
        deferred*|deferred-phase*) status_norm=deferred ;;
        *in_progress*|*in-progress*|wip|WIP) status_norm=in_progress ;;
        verified|complete) status_norm=closed ;;
        *) status_norm="$status" ;;
    esac

    # Check phase-doc ownership first (authoritative). Slug = filename minus .md.
    local slug phase_norm
    slug=$(basename "$f" .md)
    if [ -n "${SLUG_OWNER[$slug]:-}" ]; then
        phase_norm="${SLUG_OWNER[$slug]}"
    else
        phase_norm=$(normalize_phase "$phase_raw")
    fi

    # Closed/deferred items get phase from their original, else unassigned
    if [ "$status_norm" = "closed" ] || [ "$status_norm" = "deferred" ]; then
        [ -z "$phase_norm" ] || [ "$phase_norm" = "closed" ] && phase_norm="unassigned"
    fi

    # If archive/ dir, force status=closed
    if [[ "$f" == *"docs/backlog/archive/"* ]]; then
        status_norm=closed
    fi

    # Clean scope
    [ -z "$scope" ] && scope="unspecified"

    # Build frontmatter
    local fm
    fm=$(cat <<EOF
---
status: $status_norm
severity: $severity
phase: $phase_norm
opened: $opened
scope: $scope
---

EOF
)

    if [ "$DRY_RUN" = "1" ]; then
        echo "=== $f ==="
        echo "$fm"
    else
        local tmp
        tmp=$(mktemp)
        printf '%s\n' "$fm" > "$tmp"
        cat "$f" >> "$tmp"
        mv "$tmp" "$f"
    fi
}

for f in "${TARGETS[@]}"; do
    migrate_one "$f"
done

if [ "$DRY_RUN" = "1" ]; then
    echo ""
    echo "DRY RUN — rerun with --apply to write changes."
fi
