#!/usr/bin/env bash
# T-3374: recurrence guard. A guard-layer fixture must never append to the REAL
# .context/checks/voi-waiver-ledger (fixture waivers T-8/T-9/T-10 leaked into it on
# every full run, again after T-3458's fix was overwritten by T-3189).
# Runs the gate fixtures and compares the real ledger byte-for-byte.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
REAL=.context/checks/voi-waiver-ledger
PASS=0; FAIL=0
ok(){ PASS=$((PASS+1)); echo "  ok   $1"; }
bad(){ FAIL=$((FAIL+1)); echo "  FAIL $1"; }
[ -f "$REAL" ] || { echo "ledger missing: $REAL" >&2; exit 2; }

echo "== real ledger holds no fixture waiver rows =="
if grep -qE $'\t(T-8|T-9|T-10|T-11|T-12)(\t|$)' "$REAL"; then bad "fixture rows (T-8..T-12) present in real ledger"; else ok "no fixture rows in real ledger"; fi

echo "== running the gate fixtures leaves the real ledger byte-identical =="
before=$(sha256sum "$REAL" | cut -d' ' -f1)
bash tests/inception-voi-gate-fixtures.sh >/dev/null 2>&1
after=$(sha256sum "$REAL" | cut -d' ' -f1)
if [ "$before" = "$after" ]; then ok "inception-voi-gate-fixtures.sh did not touch the real ledger"; else bad "real ledger changed by the gate fixtures"; fi

echo "== mutant: without the temp-ledger export the real ledger WOULD change =="
tmp=$(mktemp -d); trap 'rm -rf -- "$tmp"' EXIT
sed '/^export FW_VOI_GATE_LEDGER=/d' tests/inception-voi-gate-fixtures.sh > "$tmp/mutant.sh"
cp "$REAL" "$tmp/ledger.bak"
GATE="$PWD/scripts/gate-inception-voi.sh" bash "$tmp/mutant.sh" >/dev/null 2>&1
if [ "$(sha256sum "$REAL" | cut -d' ' -f1)" != "$before" ]; then ok "mutant leaks (guard is load-bearing)"; else bad "mutant did not leak - guard proves nothing"; fi
cp "$tmp/ledger.bak" "$REAL"
[ "$(sha256sum "$REAL" | cut -d' ' -f1)" = "$before" ] && ok "ledger restored after mutant" || bad "ledger not restored"

echo "passed=$PASS failed=$FAIL"; [ "$FAIL" -eq 0 ]
