#!/usr/bin/env bash
# planted-default-gate-fixtures.sh (T-2855)
#
# Fixture suite for scripts/check-planted-default-gate.sh.
#
# WHY FIXTURES AND NOT ONLY THE REAL TREE. 577 @766: "RED-THEN-GREEN ON REAL STATE IS
# NECESSARY AND NOT SUFFICIENT. It proves a guard discriminates on the instance in front
# of it. It cannot show the discriminator covers the class, because the only negative
# evidence available is the corpus." Ours holds exactly two firing shapes (voi_score,
# target_blast_radius — same template, same gate, same ratio) and one spared one
# (workflow_type). A corpus-only test would prove only that those three separate.
#
# So cases D, F and H below are shapes that DO NOT OCCUR in this repo at all:
#   D  gate-required but NOT planted  (every gate-required field here IS planted)
#   F  dominant but under-populated   (both real firing fields are far over MIN_POP)
#   H  placeholder / empty / []       (never reaches the domination stage here)
#
# And 010 @764: a self-test's fixture set is a claim about which failures the author
# imagined, read as a claim about correctness. These are the ones I imagined; that is
# the honest scope of this file.
set -uo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo /opt/termlink)" || exit 2

CHECK="scripts/check-planted-default-gate.sh"
pass=0; fail=0
ok()   { echo "  ok   $1"; pass=$((pass+1)); }
bad()  { echo "  FAIL $1"; fail=$((fail+1)); }
chk()  { if [ "$1" = "$2" ]; then ok "$3 (rc=$1)"; else bad "$3 — expected rc=$2 got rc=$1"; fi; }

# Build a scratch world: templates/, gates/, tasks/{active,completed}/
# field=value pairs planted in the template; gate lines that "require" a field;
# and N tasks each carrying a given value.
mkworld() {
  W=$(mktemp -d) || exit 2
  mkdir -p "$W/templates" "$W/gates" "$W/tasks/active" "$W/tasks/completed"
}
tpl() { # tpl <name> <line...>
  local n="$1"; shift
  { echo "---"; for l in "$@"; do echo "$l"; done; echo "---"; echo "# body"; } > "$W/templates/$n.md"
}
gate() { # gate <field>  -> a line the narrow anchor accepts
  echo "        errors.append(\"$1 missing (required float 0..1)\")" >> "$W/gates/check-schema.py"
}
tasks() { # tasks <count> <line...>
  local n="$1"; shift
  for i in $(seq 1 "$n"); do
    { echo "---"; echo "id: T-$i"; for l in "$@"; do echo "$l"; done; echo "---"; } \
      > "$W/tasks/active/t$i.md"
  done
}
run() { PLANTED_ALLOWLIST="${ALLOW:-/dev/null}" bash "$CHECK" \
          --templates "$W/templates" --gates "$W/gates" --tasks-dir "$W/tasks" "$@"; }

echo "== A: planted + gate-required + dominant fires =="
mkworld; tpl inception "voi_score: 0.5"; gate voi_score; tasks 20 "voi_score: 0.5"
run > "$W/out" 2>&1; chk $? 1 "A1 dominant planted default fires"
grep -q "voi_score" "$W/out" && ok "A2 names the field" || bad "A2 names the field"
grep -q "20/20" "$W/out" && ok "A3 reports the domination ratio" || bad "A3 reports the ratio"
rm -rf "$W"

echo "== B: planted + required but SPREAD is spared (the negative control) =="
mkworld; tpl inception "voi_score: 0.5"; gate voi_score
tasks 20 "voi_score: 0.5"
for i in $(seq 1 15); do sed -i 's/voi_score: 0.5/voi_score: 0.9/' "$W/tasks/active/t$i.md"; done
run > "$W/out" 2>&1; chk $? 0 "B1 a spread field does not fire"
rm -rf "$W"

echo "== C: planted but NOT gate-required is not exposed =="
mkworld; tpl inception "horizon: now"; gate voi_score; tasks 20 "horizon: now"
run > /dev/null 2>&1; chk $? 0 "C1 no gate requires it -> not exposed"
rm -rf "$W"

echo "== D: gate-required but NOT planted  [SHAPE ABSENT FROM OUR CORPUS] =="
mkworld; tpl inception "voi_score:"; gate voi_score; tasks 20 "voi_score: 0.5"
run > /dev/null 2>&1; chk $? 0 "D1 an empty template value is not a planted default"
rm -rf "$W"

echo "== E: the allowlist =="
mkworld; tpl inception "voi_score: 0.5"; gate voi_score; tasks 20 "voi_score: 0.5"
run > /dev/null 2>&1; chk $? 1 "E1 fires before it is ledgered"
printf 'voi_score  # test reason\n' > "$W/allow"
ALLOW="$W/allow" run > "$W/out" 2>&1; chk $? 0 "E2 ledgering clears it"
ALLOW="$W/allow" run > "$W/out" 2>&1
grep -q "acknowledged: voi_score" "$W/out" && ok "E3 acknowledged entries stay VISIBLE, not silent" \
  || bad "E3 acknowledged entries stay visible"
ALLOW="$W/allow" run --json > "$W/j" 2>&1
python3 -c "import json,sys; d=json.load(open('$W/j')); sys.exit(0 if d['acknowledged_count']==1 and d['ok'] else 1)" \
  && ok "E4 JSON carries acknowledged_count" || bad "E4 JSON carries acknowledged_count"
rm -rf "$W"

echo "== F: under-populated is spared  [SHAPE ABSENT FROM OUR CORPUS] =="
mkworld; tpl inception "voi_score: 0.5"; gate voi_score; tasks 4 "voi_score: 0.5"
run > /dev/null 2>&1; chk $? 0 "F1 100% domination over 4 tasks does not fire (min-pop)"
run --min-pop 3 > /dev/null 2>&1; chk $? 1 "F2 ...and DOES fire once min-pop is lowered"
rm -rf "$W"

echo "== G: fail-closed tooling errors =="
mkworld; tpl inception "voi_score: 0.5"; gate voi_score; tasks 20 "voi_score: 0.5"
run --templates "$W/nonexistent" > /dev/null 2>&1; chk $? 2 "G1 missing templates dir is rc=2, not clean"
run --gates "$W/nonexistent" > /dev/null 2>&1;     chk $? 2 "G2 missing gates dir is rc=2, not clean"
run --tasks-dir "$W/nonexistent" > /dev/null 2>&1; chk $? 2 "G3 empty task corpus is rc=2, not clean"
run --bogus-flag > /dev/null 2>&1;                 chk $? 2 "G4 unknown flag is rc=2"
rm -rf "$W"

echo "== H: non-defaults are not defaults  [SHAPES ABSENT FROM OUR CORPUS] =="
for v in "[]" "null" "T-XXX" "{PROJECT_NAME}"; do
  mkworld; tpl inception "voi_score: $v"; gate voi_score; tasks 20 "voi_score: $v"
  run > /dev/null 2>&1; chk $? 0 "H:'$v' is not a planted default"
  rm -rf "$W"
done

echo "== I: scope disclaimer on BOTH paths (T-2680) =="
mkworld; tpl inception "voi_score: 0.5"; gate voi_score; tasks 20 "voi_score: 0.5"
run > "$W/out" 2>&1; grep -q "SCOPE:" "$W/out" && ok "I1 firing path carries scope" || bad "I1 firing scope"
printf 'voi_score  # r\n' > "$W/allow"
ALLOW="$W/allow" run > "$W/out" 2>&1; grep -q "SCOPE:" "$W/out" && ok "I2 clean path carries scope" || bad "I2 clean scope"
ALLOW="$W/allow" run --json > "$W/j" 2>&1
python3 -c "import json,sys; d=json.load(open('$W/j')); sys.exit(0 if d.get('scope') else 1)" \
  && ok "I3 clean JSON carries scope" || bad "I3 clean JSON scope"
rm -rf "$W"

echo "== J: the REAL tree (load-bearing) =="
PLANTED_ALLOWLIST=/dev/null bash "$CHECK" > /root/.claude/jobs/5f599680/tmp/_real.out 2>&1
chk $? 1 "J1 real tree fires with an EMPTY ledger"
grep -q "voi_score" /root/.claude/jobs/5f599680/tmp/_real.out && ok "J2 names voi_score" || bad "J2 names voi_score"
bash "$CHECK" > /dev/null 2>&1
chk $? 0 "J3 real tree is clean with the tracked ledger"

echo ""
echo "planted-default-gate-fixtures: $pass passed, $fail failed"
[ "$fail" = "0" ] || exit 1
