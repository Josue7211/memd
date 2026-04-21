#!/usr/bin/env bash
# regen-leaderboard.sh — regenerate / validate docs/verification/PUBLIC_LEADERBOARD.md
#
# I3 contract:
#   - Every row has the 8 method-card fields.
#   - Scores >=0.90 carry an `audit:` note or drop to recorded-unpinned.
#   - Retraction log exists and is not collapsed.
#   - Reproduction command on every row.
#
# Modes:
#   regen-leaderboard.sh --check   → lint only, exit 1 on violations (CI gate)
#   regen-leaderboard.sh --regen   → refresh the summary table's primary values
#                                    from .memd/benchmarks/history/benchmark-runs.jsonl
#                                    (replay-pending → numeric once H3 canonical
#                                     metric shows up in the latest manifest row)
#
# The method-card sections and retraction log are hand-curated; the generator
# validates structure and refuses to silently erase them.

set -euo pipefail

MODE="${1:---check}"
LEADERBOARD="${LEADERBOARD:-docs/verification/PUBLIC_LEADERBOARD.md}"
HISTORY="${HISTORY:-.memd/benchmarks/history/benchmark-runs.jsonl}"

if [ ! -f "$LEADERBOARD" ]; then
    echo "regen-leaderboard: missing $LEADERBOARD" >&2
    exit 1
fi

fail=0
fail_reason() {
    echo "regen-leaderboard: FAIL — $1" >&2
    fail=1
}

# ---- structural checks ----

required_sections=(
    "## Summary Table"
    "## Retracted Scores"
    "## Method Cards"
    "### LongMemEval Method Card"
    "### LoCoMo Method Card"
    "### MemBench Method Card"
    "### ConvoMem Method Card"
    "## Scope and Limits"
)
for section in "${required_sections[@]}"; do
    if ! grep -qF -- "$section" "$LEADERBOARD"; then
        fail_reason "missing required section: $section"
    fi
done

required_fields_per_card=(
    "Dataset fixture SHA"
    "Canonical metric"
    "Formula reference"
    "Backend"
    "Commit SHA"
    "Reproduction command"
    "Verification"
    "Cost ledger"
)
for card in "LongMemEval Method Card" "LoCoMo Method Card" "MemBench Method Card" "ConvoMem Method Card"; do
    section_body=$(awk -v s="### $card" '
        $0 == s {in_s=1; next}
        /^### / && in_s {exit}
        /^## / && in_s {exit}
        in_s {print}
    ' "$LEADERBOARD")
    if [ -z "$section_body" ]; then
        fail_reason "card '$card' body is empty"
        continue
    fi
    for field in "${required_fields_per_card[@]}"; do
        if ! printf '%s\n' "$section_body" | grep -qF -- "$field"; then
            fail_reason "card '$card' missing required field: $field"
        fi
    done
done

# ---- gaming-audit rule (>=0.90) ----
# Any row in the summary table or competitor row with a primary value
# >= 0.90 must either (a) carry an explicit `audit:` annotation in the
# same card section or competitor line, or (b) be marked
# `recorded-unpinned` / `⚠ contested`.

# Grab numeric fractions and %-values near decimals in the doc.
# Heuristic: a bare token like "0.93" or "96.6%" must live on a line
# that also mentions "audit", "contested", "recorded-unpinned", or
# "retracted". Lines matching common false positives (±0.01, USD $,
# regression budget 0.020, version numbers) are skipped.
in_retraction=0
while IFS= read -r line; do
    # section-switch tracking: every number inside ## Retracted Scores is,
    # by definition, retracted and passes the gate unconditionally.
    if printf '%s' "$line" | grep -qE '^## '; then
        if printf '%s' "$line" | grep -qF '## Retracted Scores'; then
            in_retraction=1
        else
            in_retraction=0
        fi
    fi
    if [ "$in_retraction" = "1" ]; then
        continue
    fi
    stripped=$(printf '%s' "$line" | sed -E 's#±[0-9.]+##g; s#\$[0-9.]+##g; s#budget[^.]*#budget#gi')
    if printf '%s' "$stripped" | grep -qE '(^|[^0-9.])(0?\.9[0-9]+|9[0-9]\.[0-9]+%|100%|100\.0%)'; then
        if printf '%s' "$line" | grep -qiE 'audit|contested|recorded-unpinned|retracted|pending — upstream|audit: pending'; then
            continue
        fi
        if printf '%s' "$line" | grep -qiE 'gaming-audit|0\.90 gaming threshold|0\.90 without'; then
            continue
        fi
        fail_reason "ungated >=0.90 number without audit/contested/pending: $(printf '%s' "$line" | sed 's/^[ \t]*//')"
    fi
done < "$LEADERBOARD"

# ---- reproduction command sanity ----
repro_blocks=$(grep -cE '^[[:space:]]+cargo run -p memd-client' "$LEADERBOARD" || true)
if [ "$repro_blocks" -lt 4 ]; then
    fail_reason "expected >=4 reproduction commands, got $repro_blocks"
fi

# ---- optional regen: refresh summary-table primary values from manifest ----
if [ "$MODE" = "--regen" ] && [ -f "$HISTORY" ]; then
    # Placeholder: numeric regen from the manifest lands together with
    # the first canonical-metric run in J3. For now the script validates
    # structure and signals the file is up-to-date.
    echo "regen-leaderboard: --regen noop (no canonical-metric rows in history yet; pending J3)"
fi

if [ "$fail" -ne 0 ]; then
    echo "regen-leaderboard: leaderboard failed I3 transparency checks" >&2
    exit 1
fi

echo "regen-leaderboard: OK ($LEADERBOARD passes I3 transparency checks)"
