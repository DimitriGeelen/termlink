#!/usr/bin/env bash
# guard-layer: source
# T-2969 — fixtures for scripts/check-decisions-register.sh.
#
# Hermetic: every case writes a throwaway register. No live project state.
#
# The load-bearing case is T5: a register with DUPLICATE IDS that parses
# perfectly. That is the axis the pre-push gate (T-1599/T-1610) structurally
# cannot see, and it is half of what actually went wrong on 2026-09-22 — the
# generator restarted its counter at PD-001 while PD-001 already existed. Only
# the indentation gave it away. Had the indentation been right, the register
# would have silently carried two decisions under one id.
set -u

CHK="${CHK:-scripts/check-decisions-register.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "T1: --help exits 0"
bash "$CHK" --help >/dev/null 2>&1 && pass "T1" || fail "T1 expected 0"

echo "T2: unknown argument exits 2"
bash "$CHK" --bogus >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T2 rc=$rc" || fail "T2 expected 2 got $rc"

echo "T3 [fail-closed]: a missing register exits 2, never clean"
bash "$CHK" --register "$WORK/nope.yaml" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T3 rc=$rc" || fail "T3 expected 2 got $rc"

echo "T4: a well-formed register with unique ids is clean"
cat > "$WORK/ok.yaml" <<'EOS'
- id: PD-001
  decision: "first"
  scope: project
- id: PD-002
  decision: "second"
  scope: project
EOS
bash "$CHK" --register "$WORK/ok.yaml" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 0 ] && pass "T4 rc=$rc" || fail "T4 expected 0 got $rc"

echo "T5 [THE INVISIBLE AXIS]: duplicate ids that PARSE CLEANLY must fire"
cat > "$WORK/dup.yaml" <<'EOS'
- id: PD-001
  decision: "the original"
  scope: project
- id: PD-002
  decision: "another"
  scope: project
- id: PD-001
  decision: "a DIFFERENT decision under an id that already exists"
  scope: project
EOS
# Prove the premise first: this file is valid YAML, so the pre-push gate passes it.
python3 -c "import yaml,sys; yaml.safe_load(open('$WORK/dup.yaml')); print('valid')" >/dev/null 2>&1 \
    && pass "T5a premise: the duplicate-id register IS valid YAML" \
    || fail "T5a fixture is not valid YAML — the test would prove nothing"
OUT="$(bash "$CHK" --register "$WORK/dup.yaml" 2>&1)"; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$OUT" | grep -q 'PD-001'; then
    pass "T5b duplicate id detected and named (rc=$rc)"
else
    fail "T5b THE QUIET AXIS IS NOT DETECTED — rc=$rc: $OUT"
fi

echo "T6: a parse failure fires with a line number"
cat > "$WORK/bad.yaml" <<'EOS'
- id: PD-001
  decision: "fine"
  scope: project

  - id: PD-002
    decision: "indented one level too deep — the 2026-09-22 shape"
    scope: project
EOS
OUT="$(bash "$CHK" --register "$WORK/bad.yaml" 2>&1)"; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$OUT" | grep -q 'PARSE FAILURE'; then
    pass "T6 parse failure reported (rc=$rc)"
else
    fail "T6 expected 1/PARSE FAILURE got $rc: $OUT"
fi

echo "T7: parse failure output warns against --no-verify"
printf '%s' "$OUT" | grep -q 'no-verify' \
    && pass "T7 the remediation names the trap" \
    || fail "T7 remediation does not warn against bypassing the only detector"

echo "T8 [fail-closed]: a register that parses to an unexpected shape exits 2"
printf 'just_a_string\n' > "$WORK/shape.yaml"
bash "$CHK" --register "$WORK/shape.yaml" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T8 rc=$rc" || fail "T8 expected 2 got $rc"

echo "T9 [fail-closed]: an empty list is tooling, not a clean bill"
printf -- '[]\n' > "$WORK/empty.yaml"
bash "$CHK" --register "$WORK/empty.yaml" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T9 rc=$rc (zero ids never reads green)" \
               || fail "T9 expected 2 got $rc"

echo "T10: the mapping shape (decisions: key) is tolerated"
cat > "$WORK/map.yaml" <<'EOS'
decisions:
  - id: PD-001
    decision: "first"
EOS
bash "$CHK" --register "$WORK/map.yaml" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 0 ] && pass "T10 rc=$rc" || fail "T10 expected 0 got $rc"

echo "T11: --json emits a parseable envelope with the scope disclaimer"
OUT="$(bash "$CHK" --register "$WORK/ok.yaml" --json 2>/dev/null)"
if printf '%s' "$OUT" | jq -e '.ok == true and .scope' >/dev/null 2>&1; then
    pass "T11 json ok=true with scope stated"
else
    fail "T11 json envelope wrong: $OUT"
fi

echo "T12: --json on duplicates carries the duplicate list"
OUT="$(bash "$CHK" --register "$WORK/dup.yaml" --json 2>/dev/null)"
if printf '%s' "$OUT" | jq -e '.ok == false and (.duplicates | index("PD-001"))' >/dev/null 2>&1; then
    pass "T12 json names the duplicate"
else
    fail "T12 json duplicates missing: $OUT"
fi

echo "T13 [regression]: the real register is clean after the 4th repair"
if [ -f .context/project/decisions.yaml ]; then
    bash "$CHK" >/dev/null 2>&1; rc=$?
    [ "$rc" -eq 0 ] && pass "T13 live register clean (rc=$rc)" \
                   || fail "T13 live register still corrupt (rc=$rc)"
else
    pass "T13 skipped — no live register in this tree"
fi

echo
echo "decisions-register fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
