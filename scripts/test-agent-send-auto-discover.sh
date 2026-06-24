#!/usr/bin/env bash
# T-1834 — tests for agent-send.sh --to <agent-id> auto-discover.
#
# Uses --dry-run to test the resolution layer without actually posting/
# injecting. Five tests cover the success path + four error paths.
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SCRIPT="${SCRIPT:-scripts/agent-send.sh}"
HEARTBEAT="${HEARTBEAT:-scripts/listener-heartbeat.sh}"

PASS=0; FAIL=0; SKIP=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

if "$TERMLINK" hub status >/dev/null 2>&1; then hub_up=1; else hub_up=0; fi

run_tag="t1834-$$-$(date +%s)"

# -------- T1: --to + --to-session both → exit 2 (mutex) --------
echo "T1: --to + --to-session both given → exit 2"
out="$(bash "$SCRIPT" --to "$run_tag-a" --to-session "X" --message "m" --dry-run 2>&1)"
rc=$?
if [ "$rc" -eq 2 ] && printf '%s' "$out" | grep -qF "mutex"; then
    pass "T1: mutex enforced (exit=$rc)"
else
    fail "T1: rc=$rc out=$out"
fi

# -------- T2: --to <unknown> → exit 2 with not-found --------
echo "T2: --to <unknown> → exit 2 with not-found"
if [ "$hub_up" -ne 1 ]; then
    skip "T2: local hub not up"
else
    out="$(bash "$SCRIPT" --to "definitely-not-listening-$run_tag" --message "m" --dry-run 2>&1)"
    rc=$?
    if [ "$rc" -eq 2 ] && printf '%s' "$out" | grep -qiF "no listener"; then
        pass "T2: not-found surfaced"
    else
        fail "T2: rc=$rc out=$out"
    fi
fi

# -------- T3: --to <agent> with full metadata + --dry-run → RESOLVED line --------
echo "T3: --to <agent> with pty_session + dm:* listen_topic + --dry-run"
if [ "$hub_up" -ne 1 ]; then
    skip "T3: local hub not up"
else
    aid="t3-good-$run_tag"
    bash "$HEARTBEAT" --agent-id "$aid" --pty-session "pty-t3-$run_tag" --listen-topic "dm:alice:bob" --listen-topic "agent-chat-arc" --once >/dev/null 2>&1
    sleep 1
    out="$(bash "$SCRIPT" --to "$aid" --message "m" --dry-run 2>&1)"
    rc=$?
    # T-2273: topic is now computed from the peer's identity_fingerprint (not the
    # dm:* listen_topic), so assert a dm: topic + routing=local rather than the
    # literal dm:alice:bob the old listen_topics scan would have echoed.
    if [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -qF "RESOLVED:" \
        && printf '%s' "$out" | grep -qF "to_session=pty-t3-$run_tag" \
        && printf '%s' "$out" | grep -qF "topic=dm:" \
        && printf '%s' "$out" | grep -qF "routing=local" \
        && printf '%s' "$out" | grep -qF "status=LIVE"; then
        pass "T3: RESOLVED with to_session + identity_fp topic + routing=local"
    else
        fail "T3: rc=$rc out=$out"
    fi
fi

# -------- T4: --to <agent> with no pty_session → exit 2 --------
echo "T4: --to <agent> heartbeat lacks pty_session → exit 2"
if [ "$hub_up" -ne 1 ]; then
    skip "T4: local hub not up"
else
    aid="t4-nopty-$run_tag"
    bash "$HEARTBEAT" --agent-id "$aid" --listen-topic "dm:alice:bob" --once >/dev/null 2>&1
    sleep 1
    out="$(bash "$SCRIPT" --to "$aid" --message "m" --dry-run 2>&1)"
    rc=$?
    if [ "$rc" -eq 2 ] && printf '%s' "$out" | grep -qiF "pty_session"; then
        pass "T4: pty_session-missing surfaced"
    else
        fail "T4: rc=$rc out=$out"
    fi
fi

# -------- T5: --to <agent> with no dm:* listen_topic now RESOLVES (T-2273) --------
# Pre-T-2273 this exited 2 ("no dm:* listen_topic"). The topic is now computed
# from the peer's identity_fingerprint, so a LIVE peer with no prior DM resolves.
echo "T5: --to <agent> no dm:* listen_topic → RESOLVED via identity_fingerprint"
if [ "$hub_up" -ne 1 ]; then
    skip "T5: local hub not up"
else
    aid="t5-nodm-$run_tag"
    bash "$HEARTBEAT" --agent-id "$aid" --pty-session "pty-t5-$run_tag" --listen-topic "agent-chat-arc" --once >/dev/null 2>&1
    sleep 1
    out="$(bash "$SCRIPT" --to "$aid" --message "m" --dry-run 2>&1)"
    rc=$?
    if [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -qF "RESOLVED:" \
        && printf '%s' "$out" | grep -qF "to_session=pty-t5-$run_tag" \
        && printf '%s' "$out" | grep -qF "topic=dm:"; then
        pass "T5: no-dm-topic peer resolves via identity_fingerprint"
    else
        fail "T5: rc=$rc out=$out"
    fi
fi

# -------- T6: cross-hub fixture — fleet row on a remote hub → routing=remote (T-2273) --------
# Canned fleet-presence row (LISTENERS_VERB override) places the peer on a remote
# hub; the real LOCAL verb misses it, so the fleet path is exercised and the
# resolved hub + routing=remote must surface. Proves cross-hub resolution without
# a live second hub (the T-2270 test-seam pattern).
echo "T6: --to <agent> resolved on a remote hub → RESOLVED hub + routing=remote"
if [ "$hub_up" -ne 1 ]; then
    skip "T6: local hub not up (self_fp resolution needs it)"
else
    aid="t6-remote-$run_tag"
    fixture_dir="$(mktemp -d)"
    fake_fleet="$fixture_dir/fake-fleet.sh"
    cat > "$fake_fleet" <<EOF
#!/usr/bin/env bash
cat <<'JSON'
{"total_listeners":1,"listeners":[{"agent_id":"$aid","status":"LIVE","age_secs":4,"pty_session":"pty-t6-remote","identity_fingerprint":"deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef","hub":"192.168.10.199:9100","listen_topics":""}]}
JSON
EOF
    chmod +x "$fake_fleet"
    out="$(LISTENERS_VERB="$fake_fleet" bash "$SCRIPT" --to "$aid" --message "m" --dry-run 2>&1)"
    rc=$?
    if [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -qF "RESOLVED:" \
        && printf '%s' "$out" | grep -qF "to_session=pty-t6-remote" \
        && printf '%s' "$out" | grep -qF "hub=192.168.10.199:9100" \
        && printf '%s' "$out" | grep -qF "routing=remote"; then
        pass "T6: cross-hub fleet row resolved with remote routing"
    else
        fail "T6: rc=$rc out=$out"
    fi
    rm -rf "$fixture_dir"
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
