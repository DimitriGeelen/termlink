#!/bin/bash
# Fixtures for T-3316: checkpoint.sh verb surface.
#
# /resume (user-level copy, newer upstream) calls `checkpoint.sh budget`. This
# vendored build had no such verb, so the call fell to `*) exit 1`, and the
# T-821 crash trap reported it as "HOOK CRASHED" (51 logged crashes, all of them
# this one argument). Pinned here:
#   1. `budget` is accepted (rc 0) and prints the same reading shape as `status`;
#   2. `status` still works;
#   3. an unknown verb is a usage error: rc 1, usage on stderr, NO crash banner,
#      NO line appended to .hook-crashes.log.
#
# Load-bearing (T-2814 rule): the same budget/unknown-verb cases run against the
# PRE-FIX checkpoint.sh extracted from git and MUST fail there, so a suite that
# is green against both implementations cannot pass for the wrong reason. A
# re-vendor that drops the alias or the trap reset turns this suite red.
#
# Hermetic: all state goes to a scratch PROJECT_ROOT; nothing on the host is
# written. Run: bash tests/checkpoint-verb-fixtures.sh
set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FW="$REPO_ROOT/.agentic-framework"
PRE_FIX_REF="${T3316_PRE_FIX_REF:-f64d6d5c9}"   # last commit before the T-3316 change

pass=0
fail=0
ok()  { pass=$((pass + 1)); echo "  ok   $1"; }
bad() { fail=$((fail + 1)); echo "  FAIL $1"; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

# Build a framework tree whose checkpoint.sh is the one under test, with the
# rest of the framework symlinked in (checkpoint.sh resolves FRAMEWORK_ROOT
# from its own location).
make_tree() {  # $1 = name, $2 = path to the checkpoint.sh to install
    local t="$SCRATCH/$1"
    mkdir -p "$t/fw/agents/context" "$t/project/.context/working"
    ln -s "$FW/lib" "$t/fw/lib"
    ln -s "$FW/agents/context/lib" "$t/fw/agents/context/lib"
    cp "$2" "$t/fw/agents/context/checkpoint.sh"
    chmod +x "$t/fw/agents/context/checkpoint.sh"
    echo "$t"
}

run() {  # $1 = tree, $2 = verb -> sets RC, OUT, ERR, CRASHES
    local t="$1"
    : > "$t/project/.context/working/.hook-crashes.log"
    PROJECT_ROOT="$t/project" CONTEXT_DIR="$t/project/.context" FW_TRANSCRIPT_PATH=/nonexistent \
        "$t/fw/agents/context/checkpoint.sh" "$2" > "$t/out" 2> "$t/err"
    RC=$?
    OUT="$(cat "$t/out")"; ERR="$(cat "$t/err")"
    CRASHES=$(wc -l < "$t/project/.context/working/.hook-crashes.log")
}

echo "== current checkpoint.sh"
CUR="$(make_tree cur "$FW/agents/context/checkpoint.sh")"

run "$CUR" budget
[ "$RC" -eq 0 ] && ok "budget exits 0" || bad "budget exits 0 (rc=$RC)"
# Since the AEF 1.8.3 re-vendor (T-3370) `budget` is upstream's own verb with a structured
# reading (`level:` / `tokens:` …) instead of our T-3316 alias for `status`; accept either shape.
{ grep -q "Tool calls since last commit" <<< "$OUT" || grep -q "^level:" <<< "$OUT"; } && ok "budget prints a budget reading" || bad "budget prints a budget reading: $OUT"
[ "$CRASHES" -eq 0 ] && ok "budget logs no crash" || bad "budget logs no crash ($CRASHES)"

run "$CUR" status
[ "$RC" -eq 0 ] && grep -q "Context tokens:" <<< "$OUT" && ok "status still works" || bad "status still works (rc=$RC)"

run "$CUR" bogus-verb
[ "$RC" -eq 1 ] && ok "unknown verb exits 1" || bad "unknown verb exits 1 (rc=$RC)"
grep -q "Usage: checkpoint.sh" <<< "$ERR" && ok "unknown verb prints usage on stderr" || bad "unknown verb prints usage on stderr"
! grep -q "HOOK CRASHED" <<< "$ERR$OUT" && ok "unknown verb prints no crash banner" || bad "unknown verb prints no crash banner"
[ "$CRASHES" -eq 0 ] && ok "unknown verb logs no crash" || bad "unknown verb logs no crash ($CRASHES)"

echo "== pre-fix checkpoint.sh ($PRE_FIX_REF) — these MUST reproduce the defect"
if git -C "$REPO_ROOT" show "$PRE_FIX_REF:.agentic-framework/agents/context/checkpoint.sh" > "$SCRATCH/prefix.sh" 2>/dev/null; then
    OLD="$(make_tree old "$SCRATCH/prefix.sh")"
    run "$OLD" budget
    [ "$RC" -ne 0 ] && [ "$CRASHES" -gt 0 ] && ok "pre-fix: budget crashes (defect reproduced)" || bad "pre-fix: budget crashes (rc=$RC crashes=$CRASHES) — suite is not load-bearing"
    run "$OLD" bogus-verb
    grep -q "HOOK CRASHED" <<< "$ERR" && ok "pre-fix: unknown verb shows the false banner" || bad "pre-fix: unknown verb shows the false banner — suite is not load-bearing"
else
    bad "could not extract pre-fix checkpoint.sh at $PRE_FIX_REF"
fi

echo
echo "passed=$pass failed=$fail"
if [ "$fail" -eq 0 ]; then echo "ALL PASS"; exit 0; fi
exit 1
