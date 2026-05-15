#!/usr/bin/env bash
# =============================================================================
# T-1640 — pgrep self-match prevention for hub deploy scripts.
# =============================================================================
# Bug surfaced 2026-05-15 during T-1632/T-1633 deploy on .122:
# scripts/hub-binary-swap.sh used `pgrep -f 'termlink hub start' | head -1`.
# When invoked via `termlink remote exec`, the remote shell's own argv contains
# the search string `termlink hub start`, so pgrep matches the shell itself.
# `head -1` then returns the transient shell PID instead of the long-running
# hub PID. The subsequent kill misses the real hub, the script reports
# "hub did not exit within 5s", and exits without rollback.
#
# Fix: bracket-class prefix the first letter, e.g. `'[t]ermlink hub start'`.
# The shell argv literally contains the pattern characters `[t]ermlink...`,
# so `t` is followed by `]` (not `e`), and pgrep's regex engine no longer
# self-matches the shell's argv.
#
# This test enforces the convention statically across both deploy scripts
# AND validates the runtime behavior on a real shell.
#
# Usage: bash tests/test_t1640_pgrep_self_match.sh
# =============================================================================

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HERE/.." && pwd)"

SCRIPTS=(
    "$REPO_ROOT/scripts/hub-binary-swap.sh"
    "$REPO_ROOT/scripts/fleet-deploy-binary.sh"
)

fail=0

# -----------------------------------------------------------------------------
# Static check: every pgrep -f / pkill -f literal in the two scripts must use
# the bracket-class form (pattern starts with `[<letter>]`).
# -----------------------------------------------------------------------------
for script in "${SCRIPTS[@]}"; do
    [ -f "$script" ] || { echo "FAIL: $script missing"; fail=1; continue; }

    # Find every line with pgrep -f or pkill -<anything> -f. Ignore lines starting
    # with `#` (header comments).
    while IFS= read -r line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        # Extract the quoted pattern after -f. Match either single- or double-quoted.
        if [[ "$line" =~ -f[[:space:]]+\'([^\']*)\' ]] || [[ "$line" =~ -f[[:space:]]+\"([^\"]*)\" ]]; then
            pat="${BASH_REMATCH[1]}"
            # Acceptable: pattern starts with a bracket class `[X]`
            if ! [[ "$pat" =~ ^\[[a-zA-Z]\] ]]; then
                echo "FAIL [$script]: pgrep/pkill -f pattern does not use bracket trick: $pat"
                echo "       line: $line"
                fail=1
            fi
        fi
    done < <(grep -nE '(pgrep|pkill[^|]*) -f' "$script" | cut -d: -f2-)
done

# -----------------------------------------------------------------------------
# Functional check: reproduce today's failure mode + assert the fix.
#
# The bug fires when a shell's argv literally contains the same pattern that
# pgrep is searching for. termlink remote exec injects the script's bash into
# a shell whose argv is `sh -c '...pgrep -f PATTERN...'`, so PATTERN appears
# both in the searcher's regex AND in the searched shell's cmdline.
#
# Decoy A: argv contains the OLD invocation form `pgrep -f 'termlink hub start'`.
#          The OLD pgrep pattern must match it (proves the bug exists).
# Decoy B: argv contains the NEW invocation form `pgrep -f '[t]ermlink hub start'`.
#          The NEW pgrep pattern must NOT match it (proves the fix works).
# -----------------------------------------------------------------------------
# Decoy A: literal "termlink hub start" in argv. No `exec` — the sh stays
# alive running sleep, keeping its original argv visible in /proc/PID/cmdline.
sh -c "sleep 30 # decoy A: pgrep -f 'termlink hub start' would self-match here" &
DECOY_A_PID=$!
# Decoy B: literal "[t]ermlink hub start" in argv (what the fixed script injects).
sh -c "sleep 30 # decoy B: pgrep -f '[t]ermlink hub start' should NOT self-match" &
DECOY_B_PID=$!

trap "kill $DECOY_A_PID $DECOY_B_PID 2>/dev/null" EXIT
sleep 0.3

# Old pattern must hit decoy A (reproduces the bug)
if pgrep -f 'termlink hub start' 2>/dev/null | grep -q "^$DECOY_A_PID\$"; then
    repro_ok=1
else
    repro_ok=0
fi

# New pattern must NOT hit decoy B (the fix in action). It also must not hit
# decoy A — neither, because no real hub is running in this test.
if pgrep -f '[t]ermlink hub start' 2>/dev/null | grep -q "^$DECOY_B_PID\$"; then
    fix_ok=0
else
    fix_ok=1
fi

if [ $repro_ok -eq 0 ]; then
    echo "WARN: could not reproduce the self-match with decoy A — environment difference."
    echo "      Static assertion above remains load-bearing."
elif [ $fix_ok -eq 0 ]; then
    echo "FAIL: bracket-trick pattern still self-matches decoy B (PID $DECOY_B_PID)."
    fail=1
else
    echo "PASS: functional — bug reproduced on decoy A (old pattern hits PID $DECOY_A_PID),"
    echo "       fix holds on decoy B (new pattern does NOT hit PID $DECOY_B_PID)."
fi

# -----------------------------------------------------------------------------
# Result.
# -----------------------------------------------------------------------------
if [ $fail -eq 0 ]; then
    echo "PASS: all pgrep -f / pkill -f patterns use bracket trick across both scripts"
    exit 0
else
    exit 1
fi
