#!/usr/bin/env bash
# arc011-inventory-check-fixtures.sh (T-3397)
# Hermetic proof for scripts/check-arc011-inventory.sh: scratch doc + inventory, no
# repo state. Includes the real-tree control and a mutant (cap test removed).
set -uo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-arc011-inventory.sh"
REAL_DOC="$REPO_ROOT/docs/design/interactive-agent-communication-03-floor-and-phasing.md"
REAL_INV="$REPO_ROOT/docs/design/arc-011-inventory.yaml"
S="$(mktemp -d)"; trap 'rm -rf "$S"' EXIT
pass=0; fail=0
rc_is() { # desc expected actual
    if [ "$2" -eq "$3" ]; then pass=$((pass+1)); echo "  ok: $1 (rc=$3)"; else fail=$((fail+1)); echo "  FAIL: $1 expected rc=$2 got $3" >&2; fi; }
has() { if grep -q -- "$3" <<<"$2"; then pass=$((pass+1)); echo "  ok: $1"; else fail=$((fail+1)); echo "  FAIL: $1 lacks '$3'" >&2; fi; }
run() { bash "${CHECK_OVERRIDE:-$CHECK}" --no-heartbeat "$@" 2>&1; }

# 1 real tree control
out=$(run); rc_is "real tree is clean" 0 $?
# 2 missing files are tooling
run --inventory "$S/none.yaml" >/dev/null; rc_is "missing inventory is 2" 2 $?
run --doc "$S/none.md" >/dev/null; rc_is "missing doc is 2" 2 $?
printf 'nothing here\n' > "$S/empty.md"; run --doc "$S/empty.md" >/dev/null; rc_is "doc without anchors is 2" 2 $?

# 3 doc gains a state (14th) with the inventory unchanged: disagreement
sed 's/WAITING, WAITING FOR RECIPIENT, STUCK, UNKNOWN/WAITING, WAITING FOR RECIPIENT, STUCK, UNKNOWN, REJECTED/' "$REAL_DOC" > "$S/d14.md"
out=$(run --doc "$S/d14.md"); rc_is "extra doc state fires" 1 $?; has "names the mismatch" "$out" "sender labels differ"
# 4 doc gains two states (15): cap exceeded
sed 's/WAITING, WAITING FOR RECIPIENT, STUCK, UNKNOWN/WAITING, WAITING FOR RECIPIENT, STUCK, UNKNOWN, REJECTED, GONE/' "$REAL_DOC" > "$S/d15.md"
out=$(run --doc "$S/d15.md"); rc_is "15 doc states fires" 1 $?; has "cap exceeded reported" "$out" "CAP EXCEEDED in doc"
# 5 inventory over cap and consistent count
python3 - "$REAL_INV" "$S/inv15.yaml" <<'PY'
import sys,yaml
d=yaml.safe_load(open(sys.argv[1])); d["states"]["sender_labels"]+=["X1","X2"]; d["counts"]["states"]=15
yaml.safe_dump(d,open(sys.argv[2],"w"))
PY
out=$(run --inventory "$S/inv15.yaml"); rc_is "inventory over cap fires" 1 $?; has "inventory cap reported" "$out" "CAP EXCEEDED: 15 states"
# 6 declared count wrong
sed 's/^  markers: 7/  markers: 6/' "$REAL_INV" > "$S/badcount.yaml"
out=$(run --inventory "$S/badcount.yaml"); rc_is "wrong declared count fires" 1 $?; has "count message" "$out" "inventory counts 6"
# 7 marker renamed in inventory
sed 's/- unreconciled/- reconciled/' "$REAL_INV" > "$S/badmark.yaml"
out=$(run --inventory "$S/badmark.yaml"); rc_is "marker rename fires" 1 $?
# 8 GP-12 number drift
sed 's/query_seconds: 5/query_seconds: 7/' "$REAL_INV" > "$S/badgp.yaml"
out=$(run --inventory "$S/badgp.yaml"); rc_is "gp12 drift fires" 1 $?; has "gp12 named" "$out" "gp12.query_seconds"
# 9 code annotations
mkdir -p "$S/code"; python3 - "$REAL_INV" "$S/code/m.rs" <<'PY'
import sys,yaml
d=yaml.safe_load(open(sys.argv[1])); o=open(sys.argv[2],"w")
for s in d["states"]["chain"]+d["states"]["sender_labels"]: o.write("// arc011-state: %s\n"%s)
for m in d["markers"]: o.write("// arc011-marker: %s\n"%m)
PY
run --code-root "$S/code" >/dev/null; rc_is "matching code annotations clean" 0 $?
echo "// arc011-state: SECRET_NEW" >> "$S/code/m.rs"
run --code-root "$S/code" >/dev/null; rc_is "extra code state fires" 1 $?
run --code-root "$S/nodir" >/dev/null; rc_is "bad code root is 2" 2 $?
# 10 json
out=$(run --json); has "json ok" "$out" '"ok": true'

# 11 mutant: remove the cap rules; the over-cap inventory must stop reporting CAP EXCEEDED
grep -v 'CAP EXCEEDED' "$CHECK" > "$S/mutant.sh"; chmod +x "$S/mutant.sh"
bash "$S/mutant.sh" --no-heartbeat --doc "$REAL_DOC" --inventory "$S/inv15.yaml" >"$S/m.out" 2>&1
if grep -q "CAP EXCEEDED" "$S/m.out"; then fail=$((fail+1)); echo "  FAIL: mutant still reports cap" >&2; else pass=$((pass+1)); echo "  ok: mutant without cap rule is silent on cap (real check reports it, case 5)"; fi
echo "pass=$pass fail=$fail"; [ "$fail" -eq 0 ]
