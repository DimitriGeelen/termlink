#!/usr/bin/env bash
# test-arc-live-probe.sh (T-2480) -- host-independent unit tests for the G-069
# "shipped == live" gate. Feeds canned fleet-doctor + cv-keys fixtures via the
# PL-213 test hooks so the probe runs with no live hub.
#
# Prints one line per case + a final "PASS"/"FAIL" summary line (the P-011
# verification greps for "PASS").
set -uo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT="$SELF_DIR/arc-live-probe.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fails=0
check() { # <label> <expected-exit> <actual-exit>
    if [ "$2" -eq "$3" ]; then echo "  ok   $1 (exit $3)"; else echo "  FAIL $1 (expected $2, got $3)"; fails=$((fails+1)); fi
}

# --- fixtures ---------------------------------------------------------------
# fleet doctor: hub 'primary' (192.168.10.107:9100) serving 0.11.714, cv-index-capable
cat > "$TMP/doctor_fresh.json" <<'JSON'
{"hubs":[
  {"hub":"primary","address":"192.168.10.107:9100","status":"ok","hub_version":"0.11.714"},
  {"hub":"other","address":"192.168.10.122:9100","status":"ok","hub_version":"0.11.500"}
]}
JSON
# fleet doctor: hub 'primary' serving a STALE version below any modern floor
cat > "$TMP/doctor_stale.json" <<'JSON'
{"hubs":[
  {"hub":"primary","address":"192.168.10.107:9100","status":"ok","hub_version":"0.9.0"}
]}
JSON
# fleet doctor: hub 'primary' present but reports no version (too old to answer)
cat > "$TMP/doctor_noversion.json" <<'JSON'
{"hubs":[
  {"hub":"primary","address":"192.168.10.107:9100","status":"error","hub_version":null}
]}
JSON
# fleet doctor: unparseable (no .hubs array)
echo '{"garbage":true}' > "$TMP/doctor_bad.json"
# cv-keys probe outputs
echo '{"topic":"agent-presence","count":5}' > "$TMP/cvkeys_ok.json"
echo 'error -32601: Method not found'       > "$TMP/cvkeys_rejected.json"

run() { # runs the probe with given hooks/args; echoes exit code
    ( "$@" ) >/dev/null 2>&1; echo $?
}

echo "test-arc-live-probe:"

# 1. version at/above floor -> live (0)
check "version >= floor -> live" 0 "$(TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON="$TMP/doctor_fresh.json" \
    run bash "$SCRIPT" --hub primary --min-version 0.11.700)"

# 2. version below floor -> shipped-but-not-live (1)
check "version < floor -> shipped-not-live" 1 "$(TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON="$TMP/doctor_stale.json" \
    run bash "$SCRIPT" --hub primary --min-version 0.11.700)"

# 3. hub reports no version -> shipped-but-not-live (1)
check "no version -> shipped-not-live" 1 "$(TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON="$TMP/doctor_noversion.json" \
    run bash "$SCRIPT" --hub primary --min-version 0.11.700)"

# 4. capability cv-keys served -> live (0)
check "cv-keys served -> live" 0 "$(TERMLINK_ARC_PROBE_TEST_CVKEYS_RC=0 TERMLINK_ARC_PROBE_TEST_CVKEYS_OUT="$TMP/cvkeys_ok.json" \
    run bash "$SCRIPT" --hub primary --capability cv-keys)"

# 5. capability cv-keys rejected -> shipped-but-not-live (1)
check "cv-keys rejected -> shipped-not-live" 1 "$(TERMLINK_ARC_PROBE_TEST_CVKEYS_RC=1 TERMLINK_ARC_PROBE_TEST_CVKEYS_OUT="$TMP/cvkeys_rejected.json" \
    run bash "$SCRIPT" --hub primary --capability cv-keys)"

# 6. field capability present -> live (0)
check "field present -> live" 0 "$(TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON="$TMP/doctor_fresh.json" \
    run bash "$SCRIPT" --hub primary --capability field:hub_version)"

# 7. field capability absent -> shipped-but-not-live (1)
check "field absent -> shipped-not-live" 1 "$(TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON="$TMP/doctor_fresh.json" \
    run bash "$SCRIPT" --hub primary --capability field:nonexistent_field)"

# 8. combined version + capability, both pass -> live (0)
check "version+field both pass -> live" 0 "$(TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON="$TMP/doctor_fresh.json" \
    run bash "$SCRIPT" --hub primary --min-version 0.11.700 --capability field:hub_version)"

# 9. FAIL-CLOSED: unparseable doctor JSON -> tooling error (2), never a false live
check "unparseable doctor -> tooling(2)" 2 "$(TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON="$TMP/doctor_bad.json" \
    run bash "$SCRIPT" --hub primary --min-version 0.11.700)"

# 10. FAIL-CLOSED: hub not in doctor output -> tooling error (2)
check "hub absent -> tooling(2)" 2 "$(TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON="$TMP/doctor_fresh.json" \
    run bash "$SCRIPT" --hub ghost-hub --min-version 0.11.700)"

# 11. FAIL-CLOSED: cv-keys probe inconclusive (timeout rc, no rejection signature) -> tooling(2)
check "cv-keys inconclusive -> tooling(2)" 2 "$(TERMLINK_ARC_PROBE_TEST_CVKEYS_RC=124 TERMLINK_ARC_PROBE_TEST_CVKEYS_OUT=/dev/null \
    run bash "$SCRIPT" --hub primary --capability cv-keys)"

# 12. nothing to gate on -> tooling error (2)
check "no assertions -> tooling(2)" 2 "$(run bash "$SCRIPT" --hub primary)"

# 13. missing --hub -> tooling error (2)
check "missing --hub -> tooling(2)" 2 "$(run bash "$SCRIPT" --min-version 0.11.700)"

# 14. --help documents the shipped==live concept + exit codes
if bash "$SCRIPT" --help 2>&1 | grep -q "shipped"; then
    echo "  ok   --help documents shipped==live"
else
    echo "  FAIL --help missing shipped"; fails=$((fails+1))
fi

echo
if [ "$fails" -eq 0 ]; then echo "test-arc-live-probe: PASS"; exit 0
else echo "test-arc-live-probe: FAIL ($fails failing)"; exit 1; fi
