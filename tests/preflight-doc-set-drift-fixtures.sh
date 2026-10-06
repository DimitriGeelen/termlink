#!/usr/bin/env bash
# guard-layer: source
#
# T-3379 — fixtures for scripts/check-preflight-doc-set-drift.sh, pinning the stream
# contract: a finding (exit 1) goes to STDOUT, a tooling error (exit 2) to STDERR.
# Cron routes stdout to the firing log and stderr to the .stderr sink (T-2685), and
# /canaries reads any .stderr content as ERRORING (T-2842). The detector used to print
# its DETECTED table to stderr, so the real 6-vs-7 drift of T-3378 read as "could not
# run" for three days. Hermetic: the five surfaces are copied into a temp REPO_ROOT.
set -uo pipefail
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/check-preflight-doc-set-drift.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

SURFACES="scripts/substrate-preflight.sh CLAUDE.md .claude/commands/preflight.md
docs/operations/substrate-cron-recipes.md docs/operations/substrate-getting-started.md"

mkrepo() { # dir — a consistent copy of the five surfaces
    for p in $SURFACES; do
        mkdir -p "$1/$(dirname "$p")"
        cp "$PROJECT_ROOT/$p" "$1/$p" || { echo "TOOLING: cannot copy $p" >&2; exit 2; }
    done
}
run() { # script repo args... -> sets RC, OUT, ERR
    REPO_ROOT="$2" bash "$1" --no-heartbeat "${@:3}" >"$TMP/out" 2>"$TMP/err"
    RC=$?; OUT=$(cat "$TMP/out"); ERR=$(cat "$TMP/err")
}

AGREE="$TMP/agree"; mkrepo "$AGREE"
run "$SCRIPT" "$AGREE"
if [ "$RC" -ne 0 ]; then
    echo "TOOLING: the real surfaces do not agree (rc=$RC); fix drift before running fixtures" >&2
    printf '%s\n%s\n' "$OUT" "$ERR" >&2; exit 2
fi
[ -z "$ERR" ] && grep -q "agree" <<< "$OUT" && ok "1 agreement: exit 0, 'agree' on stdout, stderr empty" \
    || bad "1 agreement: rc=$RC out=[$OUT] err=[$ERR]"

run "$SCRIPT" "$AGREE" --quiet
[ "$RC" -eq 0 ] && [ -z "$OUT" ] && [ -z "$ERR" ] && ok "2 agreement --quiet: exit 0, no output at all" \
    || bad "2 agreement --quiet: rc=$RC out=[$OUT] err=[$ERR]"

DRIFT="$TMP/drift"; mkrepo "$DRIFT"
# Change the skill surface's count word to a different number, whatever it is now.
sed -i -E 's/Run all [a-z]+ checks/Run all ten checks/' "$DRIFT/.claude/commands/preflight.md"
run "$SCRIPT" "$DRIFT"
[ "$RC" -eq 1 ] && grep -q "DETECTED" <<< "$OUT" && [ -z "$ERR" ] \
    && ok "3 drift: exit 1, DETECTED on stdout, stderr empty (a finding, not a tooling error)" \
    || bad "3 drift: rc=$RC out=[$OUT] err=[$ERR]"
grep -q "skill.*10 <-- DRIFT" <<< "$OUT" && ok "4 drift: the drifted surface is named" \
    || bad "4 drift: drift marker missing in [$OUT]"

run "$SCRIPT" "$DRIFT" --quiet
[ "$RC" -eq 1 ] && grep -q "DETECTED" <<< "$OUT" && [ -z "$ERR" ] \
    && ok "5 drift --quiet: the finding still reaches stdout (the cron firing log)" \
    || bad "5 drift --quiet: rc=$RC out=[$OUT] err=[$ERR]"

BROKEN="$TMP/broken"; mkrepo "$BROKEN"; rm "$BROKEN/.claude/commands/preflight.md"
run "$SCRIPT" "$BROKEN" --quiet
[ "$RC" -eq 2 ] && grep -q "ERROR" <<< "$ERR" && [ -z "$OUT" ] \
    && ok "6 tooling error: exit 2, ERROR on stderr, stdout empty (never a false finding)" \
    || bad "6 tooling error: rc=$RC out=[$OUT] err=[$ERR]"

# Mutant: the pre-T-3379 shape (finding printed to stderr) must turn case 3 red.
MUT="$TMP/mutant.sh"
sed 's/^echo "preflight-doc-set-drift: DETECTED"$/echo "preflight-doc-set-drift: DETECTED" >\&2/' "$SCRIPT" > "$MUT"
if cmp -s "$SCRIPT" "$MUT"; then
    bad "M1 mutant could not be built (DETECTED line not found)"
else
    run "$MUT" "$DRIFT"
    if grep -q "DETECTED" <<< "$OUT" && [ -z "$ERR" ]; then
        bad "M1 mutant (finding to stderr) was NOT caught"
    else
        ok "M1 mutant (finding to stderr) is caught by case 3's assertion"
    fi
fi

echo "fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
