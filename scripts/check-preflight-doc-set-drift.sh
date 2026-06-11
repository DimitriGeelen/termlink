#!/usr/bin/env bash
# T-2188 — /preflight check-count drift canary.
#
# The /preflight verb has its check enumeration described in 4 surfaces.
# When a new check ships (T-2181 added Check 4, T-2184 added Check 5),
# each surface must be updated independently. T-2185/T-2186/T-2187 each
# existed because three of the four surfaces fell out of sync.
#
# This canary catches the drift class before it hits operators. Each
# surface declares "N checks" or has an enumerated list; we extract the
# count from each and demand unanimity.
#
# Surfaces audited:
#   1. scripts/substrate-preflight.sh        — the script's own usage()
#   2. CLAUDE.md                              — /preflight catalog row
#   3. .claude/commands/preflight.md          — skill description + "What it checks" table
#   4. docs/operations/substrate-cron-recipes.md — cron-recipe "N checks are categorical"
#
# Exit codes:
#   0    All surfaces agree
#   1    Drift detected (diagnostic table on stderr)
#   2    Tooling error (missing file, no match found)
#
# Pure read; no network; no state mutation. Safe in any context.

set -u

REPO_ROOT="${REPO_ROOT:-$(pwd)}"
SURFACES=(
    "script:scripts/substrate-preflight.sh"
    "catalog:CLAUDE.md"
    "skill:.claude/commands/preflight.md"
    "cron:docs/operations/substrate-cron-recipes.md"
)

# Map English number words to digits. Five is the current target; we cover
# common values so the canary keeps working as checks accrete or shrink.
declare -A NUM_WORDS=(
    [one]=1 [two]=2 [three]=3 [four]=4 [five]=5
    [six]=6 [seven]=7 [eight]=8 [nine]=9 [ten]=10
)

word_to_num() {
    local w="${1,,}"
    if [[ -n "${NUM_WORDS[$w]:-}" ]]; then
        echo "${NUM_WORDS[$w]}"
    else
        echo "$w"
    fi
}

# Each surface has a distinct extraction recipe. The patterns target the
# canonical statement of intent ("N checks", "Five checks", "X checks
# are categorical") rather than per-check enumeration, because the table
# rows aren't a count — they're the manifest.
extract_count() {
    local name="$1" path="$2" body word
    if [ ! -r "$path" ]; then
        echo "ERROR"
        return
    fi
    body=$(cat "$path" 2>/dev/null)
    case "$name" in
        script)
            # usage() block: "Checks:\n  1. ... \n  2. ... \n  3. ..."
            # Count "^  N." lines under the Checks: heading. Simpler: grep the
            # explicit "Check N:" docstring entries.
            local count
            count=$(printf '%s\n' "$body" | grep -cE '^#\s+Check [0-9]+:')
            if [ "$count" -gt 0 ]; then
                echo "$count"
            else
                echo "ERROR"
            fi
            ;;
        catalog)
            # CLAUDE.md /preflight row: "Five checks:" near the start of the cell.
            word=$(printf '%s\n' "$body" | grep -oE '(One|Two|Three|Four|Five|Six|Seven|Eight|Nine|Ten) checks: ' | head -n 1 | awk '{print $1}')
            if [ -n "$word" ]; then
                word_to_num "$word"
            else
                echo "ERROR"
            fi
            ;;
        skill)
            # Skill doc: "Run all five checks" + enumerated table rows "| N |".
            # Prefer the explicit "Run all N checks" claim for symmetry with cron.
            word=$(printf '%s\n' "$body" | grep -oE 'Run all (one|two|three|four|five|six|seven|eight|nine|ten) checks' | head -n 1 | awk '{print $3}')
            if [ -n "$word" ]; then
                word_to_num "$word"
            else
                echo "ERROR"
            fi
            ;;
        cron)
            # cron-recipes: "the N checks are categorical"
            word=$(printf '%s\n' "$body" | grep -oE 'the (one|two|three|four|five|six|seven|eight|nine|ten) checks are categorical' | head -n 1 | awk '{print $2}')
            if [ -n "$word" ]; then
                word_to_num "$word"
            else
                echo "ERROR"
            fi
            ;;
    esac
}

declare -A COUNTS
declare -a ORDER
ERRORS=0
for entry in "${SURFACES[@]}"; do
    name="${entry%%:*}"
    path="${entry#*:}"
    n=$(extract_count "$name" "$REPO_ROOT/$path")
    COUNTS[$name]="$n"
    ORDER+=("$name")
    if [ "$n" = "ERROR" ]; then
        ERRORS=$((ERRORS + 1))
    fi
done

if [ "$ERRORS" -gt 0 ]; then
    echo "preflight-doc-set-drift: ERROR — failed to extract count from one or more surfaces" >&2
    for name in "${ORDER[@]}"; do
        printf '  %-8s %s\n' "$name" "${COUNTS[$name]}" >&2
    done
    exit 2
fi

# Unanimity check.
first="${COUNTS[${ORDER[0]}]}"
drifted=0
for name in "${ORDER[@]}"; do
    if [ "${COUNTS[$name]}" != "$first" ]; then
        drifted=1
        break
    fi
done

if [ "$drifted" -eq 0 ]; then
    echo "preflight-doc-set: all 4 surfaces agree on $first checks"
    exit 0
fi

echo "preflight-doc-set-drift: DETECTED" >&2
echo "" >&2
printf '  %-8s %s\n' "Surface" "Claimed count" >&2
printf '  %-8s %s\n' "-------" "-------------" >&2
for name in "${ORDER[@]}"; do
    marker=""
    [ "${COUNTS[$name]}" != "$first" ] && marker=" <-- DRIFT"
    printf '  %-8s %s%s\n' "$name" "${COUNTS[$name]}" "$marker" >&2
done
echo "" >&2
echo "Resync: pick the authoritative source (typically the script's docstring)" >&2
echo "and bring the other three surfaces into agreement. T-2185/T-2186/T-2187" >&2
echo "show the per-surface edit pattern." >&2
exit 1
