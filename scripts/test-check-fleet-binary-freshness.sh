#!/usr/bin/env bash
# T-2359 — hub-independent tests for check-fleet-binary-freshness.sh.
# Uses the PL-213 test hook (TERMLINK_FLEET_FRESHNESS_TEST_JSON) with canned
# `fleet doctor --json` fixtures — no hub, no network, no termlink binary.
set -u

SCRIPT="scripts/check-fleet-binary-freshness.sh"
TMPD=$(mktemp -d)
trap 'rm -rf "$TMPD"' EXIT

PASS=0
FAIL=0

check() { # desc expected_rc actual_rc [output grep-pattern] [output]
    local desc="$1" want="$2" got="$3" pat="${4:-}" out="${5:-}"
    if [ "$got" != "$want" ]; then
        echo "FAIL: $desc — exit $got, wanted $want"
        FAIL=$((FAIL + 1)); return
    fi
    if [ -n "$pat" ] && ! echo "$out" | grep -q "$pat"; then
        echo "FAIL: $desc — output missing pattern: $pat"
        echo "$out" | sed 's/^/    | /'
        FAIL=$((FAIL + 1)); return
    fi
    echo "pass: $desc"
    PASS=$((PASS + 1))
}

run() { # fixture floors → sets OUT/RC
    OUT=$(TERMLINK_FLEET_FRESHNESS_TEST_JSON="$1" HEARTBEAT_FILE="$TMPD/hb" \
        bash "$SCRIPT" --floors "$2" 2>&1)
    RC=$?
}

# ── fixtures ────────────────────────────────────────────────────────────────
cat > "$TMPD/fleet.json" <<'EOF'
{"ok": false, "hubs": [
  {"hub": "alpha", "status": "ok", "hub_version": "0.11.296"},
  {"hub": "beta",  "status": "ok", "hub_version": "0.11.324"},
  {"hub": "gamma", "status": "error", "error": "no route"},
  {"hub": "delta", "status": "ok", "hub_version": "0.11.806"},
  {"hub": "old",   "status": "ok"}
]}
EOF

# 1. below-floor firing: alpha 0.11.296 < floor 0.11.324
cat > "$TMPD/floors1" <<'EOF'
alpha 0.11.324
EOF
run "$TMPD/fleet.json" "$TMPD/floors1"
check "below-floor hub fires (exit 1, named)" 1 "$RC" "alpha: served=0.11.296 < floor=0.11.324" "$OUT"

# 2. at/above floor healthy
cat > "$TMPD/floors2" <<'EOF'
beta 0.11.324
EOF
run "$TMPD/fleet.json" "$TMPD/floors2"
check "at-floor hub healthy (exit 0)" 0 "$RC" "healthy" "$OUT"

# 3. unreachable hub with a floor: informational, never firing (PL-219)
cat > "$TMPD/floors3" <<'EOF'
gamma 0.11.324
EOF
run "$TMPD/fleet.json" "$TMPD/floors3"
check "unreachable floored hub does not fire" 0 "$RC" "gamma: unreachable" "$OUT"

# 4. exempt (-) hub below any floor: informational
cat > "$TMPD/floors4" <<'EOF'
alpha -
EOF
run "$TMPD/fleet.json" "$TMPD/floors4"
check "exempt hub does not fire" 0 "$RC" "exempt" "$OUT"

# 5. unknown version on a reachable floored hub FIRES
cat > "$TMPD/floors5" <<'EOF'
old 0.11.1
EOF
run "$TMPD/fleet.json" "$TMPD/floors5"
check "version-unknown floored hub fires" 1 "$RC" "old: version UNKNOWN" "$OUT"

# 6. numeric (not lexicographic) compare: 0.9.1591 < 0.11.2
cat > "$TMPD/fleet6.json" <<'EOF'
{"hubs": [{"hub": "lex", "status": "ok", "hub_version": "0.9.1591"}]}
EOF
cat > "$TMPD/floors6" <<'EOF'
lex 0.11.2
EOF
run "$TMPD/fleet6.json" "$TMPD/floors6"
check "numeric segment compare (0.9.1591 < 0.11.2 fires)" 1 "$RC" "lex: served=0.9.1591 < floor=0.11.2" "$OUT"

# 6b. and the reverse must NOT fire (0.11.2 >= floor 0.9.1591)
cat > "$TMPD/fleet6b.json" <<'EOF'
{"hubs": [{"hub": "lex", "status": "ok", "hub_version": "0.11.2"}]}
EOF
cat > "$TMPD/floors6b" <<'EOF'
lex 0.9.1591
EOF
run "$TMPD/fleet6b.json" "$TMPD/floors6b"
check "numeric compare reverse (0.11.2 >= 0.9.1591 healthy)" 0 "$RC" "healthy" "$OUT"

# 7. star default row applies to undeclared hubs
cat > "$TMPD/floors7" <<'EOF'
delta -
gamma -
old -
beta -
* 0.11.324
EOF
run "$TMPD/fleet.json" "$TMPD/floors7"
check "star default floor fires on undeclared below-floor hub" 1 "$RC" "alpha: served=0.11.296 < floor=0.11.324" "$OUT"

# 8. --json envelope shape
OUT=$(TERMLINK_FLEET_FRESHNESS_TEST_JSON="$TMPD/fleet.json" HEARTBEAT_FILE="$TMPD/hb" \
    bash "$SCRIPT" --floors "$TMPD/floors1" --json 2>&1)
RC=$?
check "--json exits 1 on firing" 1 "$RC"
echo "$OUT" | jq -e '.ok == false and (.firing | length == 1) and .firing[0].hub == "alpha" and (.hubs | length == 5)' >/dev/null 2>&1 \
    && { echo "pass: --json envelope shape"; PASS=$((PASS + 1)); } \
    || { echo "FAIL: --json envelope shape"; echo "$OUT" | sed 's/^/    | /'; FAIL=$((FAIL + 1)); }

# 9. heartbeat touched even on setup-fail (missing floors file → exit 2)
rm -f "$TMPD/hb"
OUT=$(TERMLINK_FLEET_FRESHNESS_TEST_JSON="$TMPD/fleet.json" HEARTBEAT_FILE="$TMPD/hb" \
    bash "$SCRIPT" --floors "$TMPD/nonexistent" 2>&1)
RC=$?
check "missing floors file is setup-fail (exit 2)" 2 "$RC"
[ -f "$TMPD/hb" ] && { echo "pass: heartbeat touched before setup-fail"; PASS=$((PASS + 1)); } \
    || { echo "FAIL: heartbeat not touched on setup-fail path"; FAIL=$((FAIL + 1)); }

# 10. --quiet healthy prints nothing
OUT=$(TERMLINK_FLEET_FRESHNESS_TEST_JSON="$TMPD/fleet.json" HEARTBEAT_FILE="$TMPD/hb" \
    bash "$SCRIPT" --floors "$TMPD/floors2" --quiet 2>&1)
RC=$?
check "--quiet healthy is silent (exit 0)" 0 "$RC"
[ -z "$OUT" ] && { echo "pass: --quiet healthy prints nothing"; PASS=$((PASS + 1)); } \
    || { echo "FAIL: --quiet healthy printed: $OUT"; FAIL=$((FAIL + 1)); }

echo
echo "results: $PASS passed, $FAIL failed"
if [ "$FAIL" -eq 0 ]; then
    echo "ALL PASS"
    exit 0
fi
exit 1
