#!/usr/bin/env bash
# =============================================================================
# Run All E2E Tests — discovers and runs level*.sh scripts in order
# =============================================================================
# Usage: ./tests/e2e/run-all.sh [--level N] [--from N] [--to N]
#
# Options:
#   --level N   Run only level N
#   --from N    Start from level N (default: 1)
#   --to N      Stop after level N (default: all)
#
# Exit codes:
#   0 — all tests passed
#   1 — one or more tests failed
# =============================================================================

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Parse arguments
FROM_LEVEL=1
TO_LEVEL=99
SINGLE_LEVEL=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --level) SINGLE_LEVEL="$2"; shift 2 ;;
        --from)  FROM_LEVEL="$2"; shift 2 ;;
        --to)    TO_LEVEL="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [--level N] [--from N] [--to N]"
            echo ""
            echo "Options:"
            echo "  --level N   Run only level N"
            echo "  --from N    Start from level N (default: 1)"
            echo "  --to N      Stop after level N (default: all)"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

if [[ -n "$SINGLE_LEVEL" ]]; then
    FROM_LEVEL="$SINGLE_LEVEL"
    TO_LEVEL="$SINGLE_LEVEL"
fi

# Discover level scripts
SCRIPTS=()
for script in "$SCRIPT_DIR"/level*.sh; do
    [[ -f "$script" ]] || continue
    # Extract level number from filename (e.g., level5-role-specialists.sh → 5)
    basename="$(basename "$script")"
    level="${basename#level}"
    level="${level%%-*}"
    if [[ "$level" -ge "$FROM_LEVEL" && "$level" -le "$TO_LEVEL" ]]; then
        SCRIPTS+=("$level:$script")
    fi
done

# Sort by level number
IFS=$'\n' SCRIPTS=($(sort -t: -k1 -n <<<"${SCRIPTS[*]}")); unset IFS

if [[ ${#SCRIPTS[@]} -eq 0 ]]; then
    echo "No e2e test scripts found for levels ${FROM_LEVEL}–${TO_LEVEL}"
    exit 1
fi

echo "╔══════════════════════════════════════════╗"
echo "║        TermLink E2E Test Runner          ║"
echo "╚══════════════════════════════════════════╝"
echo ""
echo "Running ${#SCRIPTS[@]} test(s): levels ${FROM_LEVEL}–${TO_LEVEL}"
echo ""

PASSED=0
FAILED=0
FAILED_LEVELS=()

for entry in "${SCRIPTS[@]}"; do
    level="${entry%%:*}"
    script="${entry#*:}"
    name="$(basename "$script" .sh)"

    echo "━━━ Level $level: $name ━━━"

    if bash "$script"; then
        PASSED=$((PASSED + 1))
        echo "  ✓ PASS"
    else
        FAILED=$((FAILED + 1))
        FAILED_LEVELS+=("$level")
        echo "  ✗ FAIL"
    fi
    echo ""
done

# Summary
echo "╔══════════════════════════════════════════╗"
echo "║  Results: $PASSED passed, $FAILED failed"
if [[ $FAILED -gt 0 ]]; then
    echo "║  Failed levels: ${FAILED_LEVELS[*]}"
fi
echo "╚══════════════════════════════════════════╝"

[[ $FAILED -eq 0 ]]
