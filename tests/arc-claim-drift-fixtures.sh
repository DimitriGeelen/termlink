#!/usr/bin/env bash
# guard-layer: source
# T-3066 — fixtures for scripts/check-arc-claim-drift.sh.
#
# Hermetic: every case builds a throwaway arcs directory. No live register, no
# hub, no network (PL-213).
#
# Weighted to the FIRING cases and to the two false-readings that would make the
# check worthless:
#   M1  demo_evidence accepted as a claim binding  (it is prose; it cannot fail)
#   M2  a bound-but-FAILING prover read as clean   (the binding going stale)
# Plus the fail-closed contract, because a checker that reports green because it
# could not look is the disease it exists to cure.
set -u

CHK="${CHK:-scripts/check-arc-claim-drift.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
EMPTY_ALLOW="$WORK/empty-allowlist"; : > "$EMPTY_ALLOW"

mkarc() { # mkarc <dir> <file> <yaml-body>
    mkdir -p "$1"; printf '%s\n' "$3" > "$1/$2"
}

run() { # run <arcs-dir> [extra args...] -> rc, output in $OUT
    local d="$1"; shift
    OUT="$(bash "$CHK" --arcs-dir "$d" --allowlist "$EMPTY_ALLOW" "$@" 2>&1)"
    return $?
}

echo "T1: --help exits 0"
bash "$CHK" --help >/dev/null 2>&1 && pass "T1" || fail "T1 expected 0"

echo "T2: unknown argument exits 2"
bash "$CHK" --bogus >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T2 rc=$rc" || fail "T2 expected 2 got $rc"

echo "T3 [fail-closed]: missing arcs dir exits 2, never clean"
bash "$CHK" --arcs-dir "$WORK/nope" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T3 rc=$rc" || fail "T3 expected 2 got $rc"

echo "T4 [fail-closed]: an EMPTY arcs dir exits 2, not 0"
mkdir -p "$WORK/empty"
run "$WORK/empty"; rc=$?
[ "$rc" -eq 2 ] && pass "T4 rc=$rc (zero arcs is never a clean bill)" \
                || fail "T4 expected 2 got $rc — a parse that stopped matching would read green forever"

echo "T5: an in-progress arc with no prover does NOT fire"
d="$WORK/t5"; mkarc "$d" a.yaml 'id: arc-x
status: in-progress
decision: "some claim"'
run "$d" --no-run; rc=$?
[ "$rc" -eq 0 ] && pass "T5 rc=$rc" || fail "T5 expected 0 got $rc: $OUT"

echo "T6: a CLOSED arc with no prover FIRES"
d="$WORK/t6"; mkarc "$d" a.yaml 'id: arc-y
status: closed
decision: "GO — no silent loss"'
run "$d" --no-run; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$OUT" | grep -q 'UNBOUND'; then
    pass "T6 rc=$rc UNBOUND"
else
    fail "T6 expected 1/UNBOUND got $rc: $OUT"
fi

echo "T7 [M1]: demo_evidence alone does NOT count as a binding"
d="$WORK/t7"; mkarc "$d" a.yaml 'id: arc-z
status: closed
decision: "GO — proven"
demo_evidence: "docs/reports/whatever.md"'
run "$d" --no-run; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$OUT" | grep -q 'cannot fail later'; then
    pass "T7 prose evidence is not a prover, and the check says why"
else
    fail "T7 DEMO_EVIDENCE WAS ACCEPTED AS A CLAIM BINDING — rc=$rc: $OUT"
fi

echo "T8: a closed arc with a PASSING bound prover does not fire"
d="$WORK/t8"; mkarc "$d" a.yaml 'id: arc-ok
status: closed
decision: "GO — works"
prover: "true"'
run "$d"; rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$OUT" | grep -q 'VERIFIED'; then
    pass "T8 rc=$rc VERIFIED"
else
    fail "T8 expected 0/VERIFIED got $rc: $OUT"
fi

echo "T9 [M2]: a closed arc with a FAILING bound prover FIRES"
d="$WORK/t9"; mkarc "$d" a.yaml 'id: arc-bad
status: closed
decision: "GO — no silent loss"
prover: "false"'
run "$d"; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$OUT" | grep -q 'CLAIM-FAILED'; then
    pass "T9 a binding that stopped passing is loud (rc=$rc)"
else
    fail "T9 A STALE BINDING READ AS CLEAN — rc=$rc: $OUT"
fi

echo "T10: --no-run does NOT execute the prover (binding checked only)"
d="$WORK/t10"; mkarc "$d" a.yaml 'id: arc-nr
status: closed
decision: "GO"
prover: "false"'
run "$d" --no-run; rc=$?
[ "$rc" -eq 0 ] && pass "T10 rc=$rc (binding present, not executed)" \
               || fail "T10 expected 0 got $rc: $OUT"

echo "T11: an allowlisted closed arc is acknowledged, not fired"
d="$WORK/t11"; mkarc "$d" a.yaml 'id: arc-ack
status: closed
decision: "GO"'
al="$WORK/t11-allow"; printf 'arc-ack  # superseded by arc-009, nothing depends on the claim\n' > "$al"
OUT="$(bash "$CHK" --arcs-dir "$d" --allowlist "$al" --no-run 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$OUT" | grep -q 'ACKNOWLEDGED'; then
    pass "T11 rc=$rc, and the reason is echoed"
else
    fail "T11 expected 0/ACKNOWLEDGED got $rc: $OUT"
fi

echo "T12 [fail-closed]: an unparseable arc exits 2, never clean"
d="$WORK/t12"; mkdir -p "$d"; printf 'id: [unclosed\n  : : :\n' > "$d/bad.yaml"
run "$d" --no-run; rc=$?
[ "$rc" -eq 2 ] && pass "T12 rc=$rc" || fail "T12 expected 2 got $rc: $OUT"

echo "T13: mixed corpus counts closed arcs only"
d="$WORK/t13"
mkarc "$d" a.yaml 'id: arc-1
status: in-progress'
mkarc "$d" b.yaml 'id: arc-2
status: closed
decision: "GO"
prover: "true"'
mkarc "$d" c.yaml 'id: arc-3
status: closed
decision: "GO"'
run "$d"; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$OUT" | grep -q 'closed: 2'; then
    pass "T13 counts closed=2 and fires on the unbound one"
else
    fail "T13 expected closed:2 and rc=1, got $rc: $OUT"
fi

echo "T14: --json emits a parseable envelope carrying the scope disclaimer"
d="$WORK/t14"; mkarc "$d" a.yaml 'id: arc-j
status: closed
decision: "GO"'
OUT="$(bash "$CHK" --arcs-dir "$d" --allowlist "$EMPTY_ALLOW" --no-run --json 2>/dev/null)"
if printf '%s' "$OUT" | jq -e '.scope and (.ok == false) and (.firing == 1)' >/dev/null 2>&1; then
    pass "T14 json ok=false firing=1 with scope stated"
else
    fail "T14 json envelope wrong: $OUT"
fi

echo "T15 [regression]: an empty field must not shift later columns"
# Tab is an IFS WHITESPACE char, so bash `read` collapses runs of tabs and every
# field after an empty one silently shifts left. That printed UNBOUND with a blank
# reason on this check's first run. The reason must be present and non-empty.
d="$WORK/t15"; mkarc "$d" a.yaml 'id: arc-shift
status: closed
decision: "GO"
demo_evidence: "docs/x.md"'
run "$d" --no-run
if printf '%s' "$OUT" | grep -qE 'UNBOUND +no prover bound'; then
    pass "T15 the reason survives the empty prover column"
else
    fail "T15 FIELD SHIFT REGRESSION — reason missing: $OUT"
fi

echo
echo "arc-claim-drift fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
