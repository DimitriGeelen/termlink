#!/usr/bin/env bash
# tests/verification-pipefail-check-fixtures.sh — T-2818 regression fixtures.
#
# Pins scripts/check-verification-pipefail.sh against a scratch .tasks tree:
#
#   1. risky `cmd | grep -q` in ## Verification   -> FIRES, exit 1
#   2. `echo "$out" | grep -q` safe shape         -> clears (SIGPIPE-immune upstream)
#   3. redirect-to-file shape                     -> clears
#   4. comment line                               -> ignored
#   5. no ## Verification block                   -> ignored
#   6. --active-only scopes to .tasks/active/
#   7. --json shape
#   8. missing detector                           -> exit 2, NEVER 0 (fail-closed)
#
# Assertion 8 is the important one. A detector that reports "clean" because it could not
# load turns an unknown into a false assurance — strictly worse than no detector, because
# someone will read the green and stop looking. The script exits 2 on import failure.
#
# The heuristic itself is NOT tested here: it belongs to the framework's
# lib/reviewer/static_scan.py::detect_l387_sigpipe_risk, which this script deliberately
# wraps rather than reimplements. These fixtures pin the WRAPPER — scoping, exit codes,
# output shape, and the fail-closed contract.
#
# Host-independent (PL-213): builds its own task tree; uses the real framework detector via
# --framework-root, and skips (exit 0, reported) if this checkout does not have it.
#
# Usage: bash tests/verification-pipefail-check-fixtures.sh
# Exit:  0 = all pass (or skipped), 1 = a fixture regressed.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/check-verification-pipefail.sh"
FW="$REPO_ROOT/.agentic-framework"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  PASS  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1" >&2; [ -n "${2:-}" ] && printf '          %s\n' "$2" >&2; }

[ -r "$SCRIPT" ] || { echo "verification-pipefail-check-fixtures: cannot read $SCRIPT" >&2; exit 2; }

echo "T-2818 verification-pipefail check fixtures"
echo

if [ ! -r "$FW/lib/reviewer/static_scan.py" ]; then
    echo "  SKIP  framework detector not present in this checkout"
    echo "        ($FW/lib/reviewer/static_scan.py)"
    echo "        The wrapper's fail-closed path is still exercised below."
    SKIP_REAL=1
else
    SKIP_REAL=0
fi

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

T="$SCRATCH/.tasks"
mkdir -p "$T/active" "$T/completed"

mk() { # path, verification-body
    local p="$1"; shift
    mkdir -p "$(dirname "$p")"
    {
        echo "---"
        echo "id: T-9999"
        echo "---"
        echo ""
        echo "## Verification"
        echo ""
        printf '%s\n' "$@"
        echo ""
        echo "## Decisions"
    } > "$p"
}

run() { bash "$SCRIPT" --tasks-dir "$T" --framework-root "$FW" "$@" 2>&1; }

if [ "$SKIP_REAL" = "0" ]; then
    # --- 1. risky shape fires -----------------------------------------------
    mk "$T/active/T-risky.md" 'termlink fleet doctor 2>&1 | grep -q PASS'
    out=$(run); rc=$?
    if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "T-risky.md"; then
        ok "risky 'cmd | grep -q' fires (exit 1) and names the file"
    else
        bad "risky shape fires" "exit $rc; out: $out"
    fi
    rm -f "$T/active/T-risky.md"

    # --- 2. echo-upstream safe shape clears ---------------------------------
    mk "$T/active/T-safe.md" 'out=$(termlink fleet doctor 2>&1); echo "$out" | grep -q PASS'
    out=$(run); rc=$?
    if [ "$rc" -eq 0 ]; then
        ok "echo-upstream safe shape clears (exit 0)"
    else
        bad "echo-upstream safe shape clears" "exit $rc; out: $out"
    fi
    rm -f "$T/active/T-safe.md"

    # --- 3. redirect-to-file shape clears -----------------------------------
    mk "$T/active/T-redir.md" 'termlink fleet doctor > /tmp/.o 2>&1' 'grep -q PASS /tmp/.o'
    out=$(run); rc=$?
    if [ "$rc" -eq 0 ]; then
        ok "redirect-to-file shape clears (exit 0)"
    else
        bad "redirect-to-file shape clears" "exit $rc; out: $out"
    fi
    rm -f "$T/active/T-redir.md"

    # --- 4. comment line ignored --------------------------------------------
    mk "$T/active/T-comment.md" '# termlink fleet doctor 2>&1 | grep -q PASS'
    out=$(run); rc=$?
    if [ "$rc" -eq 0 ]; then
        ok "commented-out risky line is ignored"
    else
        bad "commented-out risky line is ignored" "exit $rc; out: $out"
    fi
    rm -f "$T/active/T-comment.md"

    # --- 5. file with no Verification block ignored -------------------------
    printf -- "---\nid: T-8888\n---\n\n## Context\n\nfoo | grep -q bar\n" > "$T/active/T-noverif.md"
    out=$(run); rc=$?
    if [ "$rc" -eq 0 ]; then
        ok "file without a ## Verification block is ignored"
    else
        bad "file without a Verification block is ignored" "exit $rc; out: $out"
    fi
    rm -f "$T/active/T-noverif.md"

    # --- 6. --active-only scoping -------------------------------------------
    mk "$T/completed/T-old.md" 'cargo test 2>&1 | grep -q ok'
    out=$(run); rc_all=$?
    out_active=$(run --active-only); rc_active=$?
    if [ "$rc_all" -eq 1 ] && [ "$rc_active" -eq 0 ]; then
        ok "--active-only excludes completed/ (default includes it)"
    else
        bad "--active-only scoping" "all=$rc_all active=$rc_active; out: $out"
    fi

    # --- 7. --json shape ----------------------------------------------------
    jout=$(run --json)
    if printf '%s' "$jout" | grep -q '"ok": false' \
       && printf '%s' "$jout" | grep -q '"finding_count": 1' \
       && printf '%s' "$jout" | grep -q 'T-old.md'; then
        ok "--json carries ok/finding_count/findings[]"
    else
        bad "--json shape" "got: $jout"
    fi
    rm -f "$T/completed/T-old.md"
else
    echo "  (assertions 1-7 skipped: no framework detector in this checkout)"
fi

# --- 8. fail-closed on missing detector -------------------------------------
mk "$T/active/T-risky2.md" 'termlink fleet doctor 2>&1 | grep -q PASS'
out=$(bash "$SCRIPT" --tasks-dir "$T" --framework-root "$SCRATCH/no-such-fw" 2>&1); rc=$?
if [ "$rc" -eq 2 ]; then
    ok "missing detector exits 2, never 0 (fail-closed)"
else
    bad "missing detector exits 2" "exit $rc (a 0 here would be a false all-clear); out: $out"
fi

echo
echo "verification-pipefail-check-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
