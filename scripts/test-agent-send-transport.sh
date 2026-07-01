#!/usr/bin/env bash
# T-2299 (arc-003 reliable-comms, V6 slice S2) — tests for the transport-select
# seam + reachability probe in scripts/agent-send.sh.
#
# No second host required. Two seams do the work:
#   - LISTENERS_VERB canned-fleet fixture (the T-2273/T6 pattern) places the peer
#     on a chosen `hub` so the --dry-run RESOLVED line surfaces direct_addr + a
#     real reachability probe.
#   - Loopback probe: 127.0.0.1:9100 (the live local hub) is reachable=yes; a
#     closed port (127.0.0.1:1) is reachable=no — both branches of
#     `termlink remote ping` exercised for real, no mock.
#
# Covers the five S2 ACs: flag validation, hub/direct/auto dry-run RESOLVED
# lines, probe reachable-vs-unreachable, default-preserved, and the live-path
# stderr plan line (emitted only for a non-default transport, so `hub` stays
# byte-for-byte).
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT="${SCRIPT:-$HERE/agent-send.sh}"
HEARTBEAT="${HEARTBEAT:-$HERE/listener-heartbeat.sh}"
TERMLINK="${TERMLINK_BIN:-termlink}"

PASS=0; FAIL=0; SKIP=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

if "$TERMLINK" hub status >/dev/null 2>&1; then hub_up=1; else hub_up=0; fi
run_tag="t2299-$$-$(date +%s)"

# Emit a canned single-listener fleet fixture placing the peer on $1 (a hub addr).
# Echoed regardless of args (matches the T-2273 T6 fake-fleet pattern).
make_fixture() {
    local hub_addr="$1" dir; dir="$(mktemp -d)"
    cat > "$dir/fake-fleet.sh" <<EOF
#!/usr/bin/env bash
cat <<'JSON'
{"total_listeners":1,"listeners":[{"agent_id":"$run_tag","status":"LIVE","age_secs":4,"pty_session":"pty-$run_tag","identity_fingerprint":"deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef","hub":"$hub_addr","listen_topics":""}]}
JSON
EOF
    chmod +x "$dir/fake-fleet.sh"
    echo "$dir"
}

# -------- T1: invalid --transport value → exit 2 (no hub needed) --------
echo "T1: --transport bogus → exit 2 with clear message"
out="$(bash "$SCRIPT" --transport bogus --message m --to "$run_tag-x" --dry-run 2>&1)"; rc=$?
if [ "$rc" -eq 2 ] && printf '%s' "$out" | grep -qF "must be auto|direct|hub"; then
    pass "T1: bad value rejected (exit=$rc)"
else
    fail "T1: rc=$rc out=$out"
fi

# -------- T2: --transport hub (explicit) → transport=hub reachable=skip --------
# Even with a remote peer, hub transport NEVER probes (reachable=skip) and
# direct_addr still surfaces the peer hub for observability.
echo "T2: --transport hub, remote peer → transport=hub direct_addr=<hub> reachable=skip"
if [ "$hub_up" -ne 1 ]; then skip "T2: local hub not up (self_fp needs it)"; else
    dir="$(make_fixture "127.0.0.1:9100")"
    out="$(LISTENERS_VERB="$dir/fake-fleet.sh" bash "$SCRIPT" --transport hub --to "$run_tag" --message m --dry-run 2>&1)"; rc=$?
    if [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -qF "transport=hub" \
        && printf '%s' "$out" | grep -qF "direct_addr=127.0.0.1:9100" \
        && printf '%s' "$out" | grep -qF "reachable=skip"; then
        pass "T2: hub transport never probes"
    else
        fail "T2: rc=$rc out=$out"
    fi
    rm -rf "$dir"
fi

# -------- T3: --transport direct, reachable peer hub → reachable=yes --------
echo "T3: --transport direct, hub 127.0.0.1:9100 (up) → reachable=yes"
if [ "$hub_up" -ne 1 ]; then skip "T3: local hub not up"; else
    dir="$(make_fixture "127.0.0.1:9100")"
    out="$(LISTENERS_VERB="$dir/fake-fleet.sh" bash "$SCRIPT" --transport direct --to "$run_tag" --message m --dry-run 2>&1)"; rc=$?
    if [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -qF "transport=direct" \
        && printf '%s' "$out" | grep -qF "direct_addr=127.0.0.1:9100" \
        && printf '%s' "$out" | grep -qF "reachable=yes"; then
        pass "T3: direct + live loopback hub probes reachable=yes"
    else
        fail "T3: rc=$rc out=$out"
    fi
    rm -rf "$dir"
fi

# -------- T4: --transport auto, unreachable peer hub → reachable=no --------
echo "T4: --transport auto, hub 127.0.0.1:1 (closed) → reachable=no"
if [ "$hub_up" -ne 1 ]; then skip "T4: local hub not up"; else
    dir="$(make_fixture "127.0.0.1:1")"
    out="$(LISTENERS_VERB="$dir/fake-fleet.sh" bash "$SCRIPT" --transport auto --to "$run_tag" --message m --dry-run 2>&1)"; rc=$?
    if [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -qF "transport=auto" \
        && printf '%s' "$out" | grep -qF "direct_addr=127.0.0.1:1" \
        && printf '%s' "$out" | grep -qF "reachable=no"; then
        pass "T4: auto + closed port probes reachable=no"
    else
        fail "T4: rc=$rc out=$out"
    fi
    rm -rf "$dir"
fi

# -------- T5: default (no --transport) → transport=hub reachable=skip --------
# A LIVE LOCAL peer keeps peer_hub empty (routing=local), so direct_addr=local
# and the default hub transport never probes. Proves the default is preserved.
echo "T5: no --transport, local peer → transport=hub direct_addr=local reachable=skip routing=local"
if [ "$hub_up" -ne 1 ]; then skip "T5: local hub not up"; else
    aid="t5-local-$run_tag"
    bash "$HEARTBEAT" --agent-id "$aid" --pty-session "pty-$aid" --listen-topic "agent-chat-arc" --once >/dev/null 2>&1
    sleep 1
    out="$(bash "$SCRIPT" --to "$aid" --message m --dry-run 2>&1)"; rc=$?
    if [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -qF "transport=hub" \
        && printf '%s' "$out" | grep -qF "direct_addr=local" \
        && printf '%s' "$out" | grep -qF "reachable=skip" \
        && printf '%s' "$out" | grep -qF "routing=local"; then
        pass "T5: default transport=hub, local degenerate probes nothing"
    else
        fail "T5: rc=$rc out=$out"
    fi
fi

# -------- T6: live send, --transport direct → stderr plan line, stdout unchanged --------
# Explicit --topic (local, peer_hub empty). --no-await-ack keeps it fire-and-forget
# (no doorbell/receipt wait). The stderr plan line must appear; stdout still POSTED.
echo "T6: live --transport direct → stderr transport-plan line + POSTED on stdout"
if [ "$hub_up" -ne 1 ]; then skip "T6: local hub not up"; else
    ltopic="agent-send-transport-test-$$"
    out="$(bash "$SCRIPT" --to-session "no-such-$$" --topic "$ltopic" --message "direct live" \
              --transport direct --no-await-ack 2>/tmp/.t6err.$$)"; rc=$?
    err="$(cat /tmp/.t6err.$$ 2>/dev/null)"; rm -f /tmp/.t6err.$$
    if [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -qF "POSTED" \
        && printf '%s' "$err" | grep -qF "transport-plan: transport=direct"; then
        pass "T6: live direct records plan to stderr, POSTED unchanged on stdout"
    else
        fail "T6: rc=$rc out=$out err=$err"
    fi
fi

# -------- T7: live send, default hub → NO plan line (byte-for-byte preserved) --------
echo "T7: live default (hub) → NO transport-plan line (byte-for-byte)"
if [ "$hub_up" -ne 1 ]; then skip "T7: local hub not up"; else
    ltopic="agent-send-transport-test-$$"
    out="$(bash "$SCRIPT" --to-session "no-such-$$" --topic "$ltopic" --message "hub live" \
              --no-await-ack 2>/tmp/.t7err.$$)"; rc=$?
    err="$(cat /tmp/.t7err.$$ 2>/dev/null)"; rm -f /tmp/.t7err.$$
    if [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -qF "POSTED" \
        && ! printf '%s' "$err" | grep -qF "transport-plan"; then
        pass "T7: default hub emits no plan line (unchanged)"
    else
        fail "T7: rc=$rc out=$out err=$err"
    fi
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
