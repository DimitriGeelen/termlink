#!/usr/bin/env bash
# guard-layer: source
# T-3061 — fixtures for scripts/notify-rail-e2e.sh (the two-party notify-rail prover).
#
# Hermetic by construction (PL-213): every case runs with NO live hub and NO live
# peer, driven by the prover's own test seams. A prover you can only exercise
# against production is a prover nobody runs.
#
# WEIGHTED TOWARD THE FIRING CASES. A prover is trivially green when everything
# works; what matters is that it goes RED for the right reasons. Three mutants are
# pinned explicitly, each corresponding to a mistake that was actually made while
# building this thing:
#
#   M1  trusting the sender's exit code       (the T-2876 rule)
#   M2  collapsing DEAF into CLEAR            (a dead rail looking healthy)
#   M3  reporting WAKE NOT-WIRED as a pass    (a mailbox nobody reads, called green)
#
set -u

E2E="${E2E:-scripts/notify-rail-e2e.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# ---- stub builders --------------------------------------------------------
mk_send_ok()   { printf '#!/usr/bin/env bash\nexit 0\n' > "$1"; chmod +x "$1"; }
mk_send_fail() { printf '#!/usr/bin/env bash\nexit 2\n' > "$1"; chmod +x "$1"; }

# A peer that NEVER observes anything: flag frozen, ts in the distant past.
mk_peer_blind() {
    cat > "$1" <<'EOS'
#!/usr/bin/env bash
case "$2" in
  *.heartbeat) date +%s%3N ;;
  *.flag)      printf 'pending=0\nlatest_topic=\nts=1\n' ;;
esac
EOS
    chmod +x "$1"
}

# A peer that DOES observe: flag ts is always now, pending high, topic ours.
mk_peer_sees() {
    cat > "$1" <<EOS
#!/usr/bin/env bash
case "\$2" in
  *.heartbeat) date +%s%3N ;;
  *.flag)      printf 'pending=9\nlatest_topic=%s\nts=%s\n' "$2" "\$(date +%s%3N)" ;;
esac
EOS
    chmod +x "$1"
}

mk_self_sees() { mk_peer_sees "$1" "$2"; }

TOPIC="dm:3bba15e681b3a078:d1993c2c3ec44c94"

# ===========================================================================
echo "T1: --help exits 0"
bash "$E2E" --help >/dev/null 2>&1 && pass "T1" || fail "T1 expected 0"

echo "T2: unknown argument exits 2"
bash "$E2E" --bogus >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T2 rc=$rc" || fail "T2 expected 2 got $rc"

echo "T3: non-numeric --iterations exits 2"
bash "$E2E" --iterations abc >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T3 rc=$rc" || fail "T3 expected 2 got $rc"

# ---- M1: the sender's exit code is NOT evidence ---------------------------
echo "T4 [M1]: send SUCCEEDS but receiver never observes -> BROKEN, not proven"
mk_send_ok "$WORK/send-ok.sh"; mk_peer_blind "$WORK/peer-blind.sh"
out="$(NOTIFY_E2E_TEST_PRECOND=1 \
       NOTIFY_E2E_TEST_SEND="$WORK/send-ok.sh" \
       NOTIFY_E2E_TEST_PEER_READ="$WORK/peer-blind.sh" \
       bash "$E2E" --stages deliver --deliver-timeout 4 2>&1)"; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q 'DELIVER.*FAIL'; then
    pass "T4 a green send with a blind receiver is BROKEN (rc=$rc)"
else
    fail "T4 THE T-2876 RULE IS NOT ENFORCED — rc=$rc; prover trusted the sender"
fi

echo "T5 [M1]: send FAILS but receiver DOES observe -> still proven"
mk_send_fail "$WORK/send-bad.sh"; mk_peer_sees "$WORK/peer-sees.sh" "$TOPIC"
out="$(NOTIFY_E2E_TEST_PRECOND=1 \
       NOTIFY_E2E_TEST_SEND="$WORK/send-bad.sh" \
       NOTIFY_E2E_TEST_PEER_READ="$WORK/peer-sees.sh" \
       bash "$E2E" --stages deliver --deliver-timeout 10 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'DELIVER.*PASS'; then
    pass "T5 receiver observation outranks a red send (rc=$rc)"
else
    fail "T5 expected PASS on receiver evidence, rc=$rc"
fi

# ---- M2: DEAF must never read as CLEAR ------------------------------------
echo "T6 [M2]: E1 proves a dead sidecar reports DEAF(3), never CLEAR(0)"
out="$(bash "$E2E" --stages "" --experiment e1 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'E1.*PASS'; then
    pass "T6 dead sidecar stays distinguishable from a quiet one"
else
    fail "T6 E1 did not pass — rc=$rc: $(printf '%s' "$out" | head -3)"
fi

echo "T7 [M2]: a stale peer heartbeat is TOOLING(2), never a delivery verdict"
cat > "$WORK/peer-stale.sh" <<'EOS'
#!/usr/bin/env bash
case "$2" in
  *.heartbeat) echo 1 ;;
  *.flag)      printf 'pending=0\nlatest_topic=\nts=1\n' ;;
esac
EOS
chmod +x "$WORK/peer-stale.sh"
mk_send_ok "$WORK/send-ok2.sh"
out="$(NOTIFY_E2E_TEST_SEND="$WORK/send-ok2.sh" \
       NOTIFY_E2E_TEST_PEER_READ="$WORK/peer-stale.sh" \
       NOTIFY_E2E_TEST_SELF_READ="$WORK/peer-stale.sh" \
       NOTIFY_E2E_PEER_SESSION=tl-fixture \
       bash "$E2E" --stages precond,deliver --deliver-timeout 4 2>&1)"; rc=$?
if [ "$rc" -eq 2 ]; then
    pass "T7 stale producer is fail-closed TOOLING (rc=2)"
else
    fail "T7 expected 2 (tooling) got $rc — 'cannot look' must not share an exit code with 'looked and it is broken'"
fi

# ---- M3: NOT-WIRED is not a pass ------------------------------------------
echo "T8 [M3]: zero flag consumers -> WAKE NOT-WIRED and overall BROKEN"
out="$(NOTIFY_E2E_TEST_PRECOND=1 NOTIFY_E2E_TEST_WAKE_CONSUMERS=0 \
       bash "$E2E" --stages wake 2>&1)"; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q 'NOT-WIRED'; then
    pass "T8 an unread mailbox is not reported green (rc=$rc)"
else
    fail "T8 NOT-WIRED WAS TREATED AS A PASS — rc=$rc"
fi

echo "T9 [M3]: a wired consumer -> WAKE PASS"
out="$(NOTIFY_E2E_TEST_PRECOND=1 NOTIFY_E2E_TEST_WAKE_CONSUMERS=2 \
       bash "$E2E" --stages wake 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'WAKE.*PASS'; then
    pass "T9 wired consumers pass (rc=$rc)"
else
    fail "T9 expected PASS with 2 consumers, rc=$rc"
fi

# ---- freshness guard (the vacuous-pass regression) ------------------------
echo "T10: a STICKY latest_topic with an OLD ts must NOT count as an observation"
cat > "$WORK/peer-sticky.sh" <<EOS
#!/usr/bin/env bash
case "\$2" in
  *.heartbeat) date +%s%3N ;;
  # topic already ours from a PREVIOUS send, but ts is ancient.
  *.flag)      printf 'pending=3\nlatest_topic=%s\nts=1\n' "$TOPIC" ;;
esac
EOS
chmod +x "$WORK/peer-sticky.sh"
mk_send_ok "$WORK/send-ok3.sh"
out="$(NOTIFY_E2E_TEST_PRECOND=1 \
       NOTIFY_E2E_TEST_SEND="$WORK/send-ok3.sh" \
       NOTIFY_E2E_TEST_PEER_READ="$WORK/peer-sticky.sh" \
       bash "$E2E" --stages deliver --deliver-timeout 4 2>&1)"; rc=$?
if [ "$rc" -eq 1 ]; then
    pass "T10 stale flag with matching topic is NOT an observation (rc=$rc)"
else
    fail "T10 VACUOUS PASS — prover matched pre-existing state, rc=$rc"
fi

# ---- receipt stage --------------------------------------------------------
echo "T11: a receipt from the peer fp passes RECEIPT"
printf '{"receipts":[{"sender_id":"3bba15e681b3a078","ts_unix_ms":1,"up_to":22}]}' > "$WORK/r-ok.json"
out="$(NOTIFY_E2E_TEST_PRECOND=1 NOTIFY_E2E_TEST_RECEIPTS="$WORK/r-ok.json" \
       bash "$E2E" --stages receipt 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'RECEIPT.*PASS'; then
    pass "T11 peer receipt recognised (rc=$rc)"
else
    fail "T11 expected PASS, rc=$rc"
fi

echo "T12: receipts from OTHER fps only -> RECEIPT FAIL"
printf '{"receipts":[{"sender_id":"deadbeefdeadbeef","ts_unix_ms":1,"up_to":9}]}' > "$WORK/r-other.json"
out="$(NOTIFY_E2E_TEST_PRECOND=1 NOTIFY_E2E_TEST_RECEIPTS="$WORK/r-other.json" \
       bash "$E2E" --stages receipt 2>&1)"; rc=$?
if [ "$rc" -eq 1 ]; then
    pass "T12 a stranger's receipt is not the peer's (rc=$rc)"
else
    fail "T12 expected FAIL, rc=$rc — receipt attribution is not checked"
fi

echo "T13: empty receipts -> RECEIPT FAIL (the await-ack dead-letter case)"
printf '{"receipts":[]}' > "$WORK/r-empty.json"
NOTIFY_E2E_TEST_PRECOND=1 NOTIFY_E2E_TEST_RECEIPTS="$WORK/r-empty.json" \
    bash "$E2E" --stages receipt >/dev/null 2>&1; rc=$?
[ "$rc" -eq 1 ] && pass "T13 rc=$rc" || fail "T13 expected 1 got $rc"

# ---- topic derivation -----------------------------------------------------
echo "T14: the dm topic is the SORTED pair, regardless of argument order"
a="$(bash "$E2E" --self-fp aaaa --peer-fp bbbb --help >/dev/null 2>&1; \
     NOTIFY_E2E_TEST_PRECOND=1 bash "$E2E" --self-fp bbbb --peer-fp aaaa --stages "" 2>&1 | grep -o 'dm:[a-z:]*' | head -1)"
if [ "$a" = "dm:aaaa:bbbb" ]; then
    pass "T14 topic=$a (sorted, order-independent)"
else
    fail "T14 expected dm:aaaa:bbbb got '$a'"
fi

# ---- json envelope --------------------------------------------------------
echo "T15: --json emits parseable output carrying the verdict"
out="$(NOTIFY_E2E_TEST_PRECOND=1 NOTIFY_E2E_TEST_WAKE_CONSUMERS=0 \
       bash "$E2E" --stages wake --json 2>/dev/null)"
if printf '%s' "$out" | jq -e '.verdict' >/dev/null 2>&1; then
    v="$(printf '%s' "$out" | jq -r '.verdict')"
    [ "$v" = "BROKEN" ] && pass "T15 json verdict=$v" || fail "T15 verdict=$v expected BROKEN"
else
    fail "T15 --json did not emit parseable JSON"
fi

echo
echo "notify-rail-e2e fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
