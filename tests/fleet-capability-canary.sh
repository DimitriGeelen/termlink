#!/usr/bin/env bash
# tests/fleet-capability-canary.sh (T-2415) — hermetic test for the fleet
# capability-freshness canary (G-084 prevention). Two layers:
#   (1) classify_probe pure-function unit tests (capable/incapable/inconclusive)
#   (2) end-to-end via the PL-213 seams (canned fleet doctor JSON + per-hub probe
#       fixture dir), asserting the load-bearing behaviours:
#         - a version-floor-EXEMPT hub still FIRES when doorbell-incapable (.121)
#         - a capable hub does NOT fire
#         - unreachable hubs stay informational (never firing)
#         - an inconclusive (network-ish) probe does NOT fire
#         - FLEET_CAP_EXEMPT opts a hub out of firing
set -u
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SELF_DIR/.."
SCRIPT="$ROOT/scripts/check-fleet-capability-freshness.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
fails=0
pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1"; fails=$((fails+1)); }

# ── (1) classify_probe pure unit tests ───────────────────────────────────────
FLEET_CAP_LIB=1 . "$SCRIPT"   # source in lib mode → helpers only, no walk

cl() { # expected rc out label
    local want="$1" rc="$2" out="$3" label="$4" got
    got=$(classify_probe "$rc" "$out")
    [ "$got" = "$want" ] && pass "classify: $label -> $got" \
                         || fail "classify: $label expected $want got $got"
}
cl capable      0 '{"count":2,"entries":[]}'  "exit0 + count=2 (healthy peer)"
cl capable      0 '{"count":0,"entries":[]}'  "exit0 + count=0 (empty cv_index is healthy, T-2106)"
cl incapable    1 'Error: Hub returned error for channel.cv_keys: JSON-RPC error -32001: Missing '\''target'\'' in params' \
    "-32001 Missing target (the real .121 signature)"
cl incapable    1 'Error: JSON-RPC error -32601: Method not found' \
    "-32601 method not found (hub too old for the RPC)"
cl inconclusive 124 '' \
    "empty/timeout (network — fleet doctor owns reachability)"
cl inconclusive 1 'Error: TCP connect to 1.2.3.4:9100 failed: No route to host' \
    "connection failure (not an RPC rejection)"
# an exit-0 body that is NOT the expected shape must not read as capable
cl inconclusive 0 'garbage-not-json' "exit0 but unparseable body"

# is_exempt honours FLEET_CAP_EXEMPT
( FLEET_CAP_EXEMPT="a,ring20-dashboard,c"; is_exempt ring20-dashboard ) \
    && pass "is_exempt matches a listed hub" || fail "is_exempt missed a listed hub"
( FLEET_CAP_EXEMPT="a,b"; is_exempt ring20-dashboard ) \
    && fail "is_exempt matched an unlisted hub" || pass "is_exempt rejects an unlisted hub"

# ── (2) end-to-end via seams ─────────────────────────────────────────────────
# Canned fleet doctor: one capable, one EXEMPT-from-version-floor-but-incapable,
# one unreachable, one inconclusive.
cat > "$TMP/doctor.json" <<'JSON'
{"hubs":[
  {"hub":"good-hub","status":"ok","address":"10.0.0.1:9100","hub_version":"0.11.500"},
  {"hub":"ring20-dashboard","status":"ok","address":"10.0.0.121:9100","hub_version":"0.11.806"},
  {"hub":"down-hub","status":"error","address":"10.0.0.9:9100"},
  {"hub":"flaky-hub","status":"ok","address":"10.0.0.5:9100","hub_version":"0.11.500"}
]}
JSON

# per-hub probe fixtures (slug = address with :/. → _)
mkdir -p "$TMP/probes"
slug() { printf '%s' "$1" | tr ':./' '___'; }
mk() { printf '%s' "$2" > "$TMP/probes/$(slug "$1").rc"; printf '%s' "$3" > "$TMP/probes/$(slug "$1").out"; }
mk 10.0.0.1:9100   0   '{"count":3,"entries":[]}'                                          # capable
mk 10.0.0.121:9100 1   'Error: JSON-RPC error -32001: Missing '\''target'\'' in params'    # incapable
mk 10.0.0.5:9100   124 ''                                                                   # inconclusive
# down-hub: no probe fixture — must never be probed (unreachable path)

run() { # extra-env... ; sets RUN_OUT RUN_RC
    RUN_OUT=$(env TERMLINK_FLEET_CAP_DOCTOR_JSON="$TMP/doctor.json" \
              TERMLINK_FLEET_CAP_PROBE_DIR="$TMP/probes" \
              "$@" bash "$SCRIPT" --no-heartbeat --json 2>/dev/null)
    RUN_RC=$?
}

run
[ "$RUN_RC" -eq 1 ] && pass "e2e: fires (exit 1) with an incapable hub present" \
                    || fail "e2e: expected exit 1, got $RUN_RC"
echo "$RUN_OUT" | jq -e '.ok == false' >/dev/null 2>&1 \
    && pass "e2e: ok:false when firing" || fail "e2e: expected ok:false"
echo "$RUN_OUT" | jq -e '[.firing[].hub] == ["ring20-dashboard"]' >/dev/null 2>&1 \
    && pass "e2e: ONLY the exempt-but-incapable hub fires" \
    || fail "e2e: firing set wrong: $(echo "$RUN_OUT" | jq -c '.firing')"
echo "$RUN_OUT" | jq -e '.firing[0].capability == "channel.cv_keys"' >/dev/null 2>&1 \
    && pass "e2e: firing row names the failed capability" \
    || fail "e2e: firing row missing capability"
echo "$RUN_OUT" | jq -e '[.hubs[]|select(.hub=="good-hub")][0].state == "ok"' >/dev/null 2>&1 \
    && pass "e2e: capable hub state=ok" || fail "e2e: capable hub not ok"
echo "$RUN_OUT" | jq -e '[.hubs[]|select(.hub=="down-hub")][0].state == "unreachable"' >/dev/null 2>&1 \
    && pass "e2e: unreachable hub informational" || fail "e2e: unreachable hub misclassified"
echo "$RUN_OUT" | jq -e '[.hubs[]|select(.hub=="flaky-hub")][0].state == "inconclusive"' >/dev/null 2>&1 \
    && pass "e2e: inconclusive probe does not fire" || fail "e2e: inconclusive misclassified"

# FLEET_CAP_EXEMPT silences the incapable hub → healthy
run FLEET_CAP_EXEMPT=ring20-dashboard
[ "$RUN_RC" -eq 0 ] && pass "e2e: FLEET_CAP_EXEMPT silences the incapable hub (exit 0)" \
                    || fail "e2e: expected exit 0 under exemption, got $RUN_RC"
echo "$RUN_OUT" | jq -e '[.hubs[]|select(.hub=="ring20-dashboard")][0].state == "exempt"' >/dev/null 2>&1 \
    && pass "e2e: exempted hub state=exempt" || fail "e2e: exemption not applied"

# all-capable fleet → healthy exit 0
cat > "$TMP/doctor2.json" <<'JSON'
{"hubs":[{"hub":"good-hub","status":"ok","address":"10.0.0.1:9100","hub_version":"0.11.500"}]}
JSON
RUN_OUT=$(TERMLINK_FLEET_CAP_DOCTOR_JSON="$TMP/doctor2.json" TERMLINK_FLEET_CAP_PROBE_DIR="$TMP/probes" \
          bash "$SCRIPT" --no-heartbeat --json 2>/dev/null); RUN_RC=$?
[ "$RUN_RC" -eq 0 ] && pass "e2e: all-capable fleet is healthy (exit 0)" \
                    || fail "e2e: all-capable expected exit 0, got $RUN_RC"

# ── syntax ───────────────────────────────────────────────────────────────────
bash -n "$SCRIPT" 2>/dev/null && pass "bash -n scripts/check-fleet-capability-freshness.sh clean" \
                              || fail "bash -n FAILED"
# the version-floor canary must be UNTOUCHED (no regression — separate concern)
bash -n "$ROOT/scripts/check-fleet-binary-freshness.sh" 2>/dev/null \
    && pass "bash -n scripts/check-fleet-binary-freshness.sh clean (untouched)" \
    || fail "version-floor canary broken"

echo ""
if [ "$fails" -eq 0 ]; then echo "fleet-capability-canary: ALL PASS"; exit 0
else echo "fleet-capability-canary: $fails FAIL"; exit 1; fi
