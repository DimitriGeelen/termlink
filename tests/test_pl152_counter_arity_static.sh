#!/usr/bin/env bash
# =============================================================================
# T-1623 — static check for PL-152 (counter-arity drift) on stats_for_window.
# =============================================================================
# PL-152 has fired three times in one week:
#   - T-1615: metrics legacy_unattributable miscounts
#   - T-1619: metrics trend mode ValueError on default invocation
#   - T-1620: MCP fail_count immutable, doctor `ok` was a tautology
#
# Common pattern: aggregation function's tuple return shape grows; one call
# site silently keeps the old unpack arity. Production crashes / silently-
# wrong telemetry follow.
#
# This test extracts `stats_for_window`'s return arity from the embedded
# Python heredoc in `agents/metrics/api-usage.sh` and asserts every call
# site's unpack-LHS arity matches. Catches the next drift at test-time
# instead of operator-time.
#
# T-1622 explicitly avoided extending `stats_for_window`'s 10-tuple return,
# adding a separate helper instead — that discipline becomes structural here.
#
# Usage: bash tests/test_pl152_counter_arity_static.sh
# =============================================================================

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HERE/.." && pwd)"
SRC="$REPO_ROOT/.agentic-framework/agents/metrics/api-usage.sh"

PASS=0
FAIL=0

ok()   { PASS=$((PASS+1)); echo "  PASS: $*"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL: $*"; }

echo "=== T-1623 PL-152 counter-arity static check ==="
echo "Target: $SRC"

if [ ! -f "$SRC" ]; then
    fail "api-usage.sh not found at $SRC"
    exit 1
fi

# -----------------------------------------------------------------------------
# Extract return arity from `def stats_for_window` body.
# The function's body ends with a multi-line `return (...)` expression that
# we count by stripping comments/whitespace and counting top-level commas.
# -----------------------------------------------------------------------------

EXPECTED_ARITY=$(SRC="$SRC" python3 - <<'PY'
import os, re, sys
src = open(os.environ["SRC"]).read()
# Find the function body. The signature is unique; the body runs until the
# next top-level `def` or end of heredoc.
m = re.search(r"def stats_for_window\([^)]*\):(.*?)(?=^def |\Z)", src, re.DOTALL | re.MULTILINE)
if not m:
    print("ERR_NO_DEF", file=sys.stderr)
    sys.exit(2)
body = m.group(1)
# Find the last `return (...)` in the function body — supports the multi-line
# tuple form used by stats_for_window. We bracket-balance to handle the
# multi-line tuple correctly.
ret_pos = body.rfind("return (")
if ret_pos < 0:
    print("ERR_NO_RETURN_TUPLE", file=sys.stderr)
    sys.exit(2)
i = ret_pos + len("return ")
depth = 0
start = None
for j in range(i, len(body)):
    c = body[j]
    if c == "(":
        if depth == 0:
            start = j + 1
        depth += 1
    elif c == ")":
        depth -= 1
        if depth == 0:
            end = j
            break
else:
    print("ERR_UNCLOSED_TUPLE", file=sys.stderr)
    sys.exit(2)
expr = body[start:end]
# Strip Python comments and whitespace, then count TOP-LEVEL commas.
# We can't just split on ',' — nested parens (none expected here, but be
# safe) would inflate the count.
cleaned = re.sub(r"#[^\n]*", "", expr)
depth = 0
elements = 1
for c in cleaned:
    if c == "(": depth += 1
    elif c == ")": depth -= 1
    elif c == "," and depth == 0:
        elements += 1
# An empty trailing element (",)") would over-count; check for it.
last_token = cleaned.rstrip().rstrip(",").strip()
if not last_token:
    elements -= 1
print(elements)
PY
)

if [ -z "$EXPECTED_ARITY" ] || ! [[ "$EXPECTED_ARITY" =~ ^[0-9]+$ ]]; then
    fail "could not extract return arity (got: '$EXPECTED_ARITY')"
    exit 1
fi
ok "stats_for_window return arity = $EXPECTED_ARITY"

# -----------------------------------------------------------------------------
# Find every `= stats_for_window(...)` call site and count unpack vars on LHS.
# Skip the function signature line itself (`def stats_for_window(`).
# -----------------------------------------------------------------------------

# Each line of the form `<vars> = stats_for_window(<args>)`.
# We count commas in <vars> + 1 to get unpack arity.
mapfile -t CALL_LINES < <(grep -nE '= *stats_for_window\(' "$SRC" | grep -v 'def stats_for_window' || true)

if [ "${#CALL_LINES[@]}" -eq 0 ]; then
    fail "no call sites of stats_for_window found — test cannot self-validate"
    exit 1
fi

ok "found ${#CALL_LINES[@]} call site(s) of stats_for_window"

# -----------------------------------------------------------------------------
# Per-site arity check.
# -----------------------------------------------------------------------------

MISMATCHES=0
for line in "${CALL_LINES[@]}"; do
    LINENO_PART="${line%%:*}"
    CONTENT="${line#*:}"
    # Strip everything from `= stats_for_window` onward — leaves only LHS unpack.
    LHS="${CONTENT%%= *stats_for_window*}"
    # `${CONTENT%%= ...}` keeps prefix that doesn't match. If no match, it's
    # the whole line. Tolerate that gracefully.
    if [ "$LHS" = "$CONTENT" ]; then
        # Try a different pattern (no spaces around =).
        LHS="${CONTENT%%=stats_for_window*}"
    fi
    # Count unpack vars: number of comma-separated names. Ignore leading/trailing whitespace.
    LHS_TRIM="$(echo "$LHS" | tr -d ' \t')"
    if [ -z "$LHS_TRIM" ]; then
        fail "line $LINENO_PART: could not extract LHS"
        MISMATCHES=$((MISMATCHES+1))
        continue
    fi
    # Count commas + 1.
    COMMAS=$(echo -n "$LHS_TRIM" | tr -dc ',' | wc -c)
    SITE_ARITY=$((COMMAS + 1))
    if [ "$SITE_ARITY" -ne "$EXPECTED_ARITY" ]; then
        fail "line $LINENO_PART: unpack arity $SITE_ARITY != expected $EXPECTED_ARITY"
        echo "    | $CONTENT"
        MISMATCHES=$((MISMATCHES+1))
    else
        ok "line $LINENO_PART: arity $SITE_ARITY ✓"
    fi
done

# -----------------------------------------------------------------------------

echo ""
echo "Pass: $PASS  Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
