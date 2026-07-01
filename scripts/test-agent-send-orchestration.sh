#!/usr/bin/env bash
# T-2301 (arc-003 reliable-comms, V6 slice S4) — tests for the try-direct /
# fall-back-to-hub ORCHESTRATION in scripts/agent-send.sh.
#
# No second host. Two seams do the work (same as the S2 suite):
#   - LISTENERS_VERB canned-fleet fixture places the peer on a chosen `hub`
#     address, so agent-send resolves a REMOTE peer and takes the direct/fallback
#     branch.
#   - Loopback reachability: 127.0.0.1:9100 (the live local hub) probes
#     reachable=yes → DIRECT; a closed port (127.0.0.1:1) probes reachable=no →
#     FALLBACK. Both against a real hub, no mock.
#   - The direct post targets --hub 127.0.0.1:9100, which IS the local hub, so the
#     turn + the self-posted receipt land on the same place the poll reads.
#
# Covers the five S4 ACs: default flips to auto (O5); DIRECT branch delivers via
# mechanism A (O1); AUTO fallback on an unreachable host emits the LOUD line +
# delivers via the hub leg (O2); --transport direct on a down host FAILS loud with
# no post (O3); --transport hub does NOT fall back — byte-for-byte escape hatch (O4).
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT="${SCRIPT:-$HERE/agent-send.sh}"
TERMLINK="${TERMLINK_BIN:-termlink}"

command -v "$TERMLINK" >/dev/null 2>&1 || { echo "SKIP: termlink not on PATH"; exit 0; }
"$TERMLINK" hub status >/dev/null 2>&1  || { echo "SKIP: no local hub running"; exit 0; }
command -v jq >/dev/null 2>&1           || { echo "SKIP: jq not available"; exit 0; }

PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
export TERMLINK_PROBE_TIMEOUT=3      # bound the down-host probes

run_tag="t2301-$$-$(date +%s)"
# Fresh 64-hex peer fingerprint per run → a unique dm topic, no cross-run bleed.
peer_fp="$(printf 'ab%062d' "$$" | cut -c1-64)"

# Canned single-listener fleet fixture placing the peer on $1 (a hub addr).
make_fixture() {
    local hub_addr="$1" dir; dir="$(mktemp -d)"
    cat > "$dir/fake-fleet.sh" <<EOF
#!/usr/bin/env bash
cat <<'JSON'
{"total_listeners":1,"listeners":[{"agent_id":"$run_tag","status":"LIVE","age_secs":4,"pty_session":"pty-$run_tag","identity_fingerprint":"$peer_fp","hub":"$hub_addr","listen_topics":""}]}
JSON
EOF
    chmod +x "$dir/fake-fleet.sh"; echo "$dir"
}

# Parse the "posted turn to 'TOPIC' (cid=CID, offset=OFF)" line agent-send emits.
topic_of()  { sed -n "s/.*posted turn to '\([^']*\)'.*/\1/p" "$1" | head -1; }
offset_of() { sed -n "s/.*offset=\([0-9][0-9]*\).*/\1/p" "$1" | head -1; }

# -------- O1: DIRECT branch (reachable peer) → DELIVERED via mechanism A --------
# default transport (auto) + reachable loopback hub → direct; self-ack a receipt
# on the same hub → the send reaches DELIVERED.
echo "O1: auto + reachable peer (127.0.0.1:9100) → DIRECT, DELIVERED via mechanism A"
dir="$(make_fixture "127.0.0.1:9100")"
cidO1="cid-o1-$run_tag"
(
    set +e
    LISTENERS_VERB="$dir/fake-fleet.sh" "$SCRIPT" --to "$run_tag" --message "direct O1" \
        --conversation-id "$cidO1" --timeout 8 --max-rings 2 >"$tmp/o1.out" 2>&1
    echo $? >"$tmp/o1.rc"
) &
bg=$!; sleep 2
t1="$(topic_of "$tmp/o1.out")"; o1off="$(offset_of "$tmp/o1.out")"
if [ -n "$t1" ]; then
    "$TERMLINK" channel post "$t1" --hub 127.0.0.1:9100 --msg-type receipt \
        --metadata conversation_id="$cidO1" --metadata up_to="${o1off:-0}" \
        --metadata stage=delivered --ensure-topic --json >/dev/null 2>&1
fi
wait "$bg" || true
rc1="$(cat "$tmp/o1.rc" 2>/dev/null || echo X)"
if [ "$rc1" = "0" ] && grep -q "DELIVERED" "$tmp/o1.out" && ! grep -q "FALLBACK" "$tmp/o1.out"; then
    pass "O1: direct path delivered (rc=0, no fallback), topic=$t1"
else
    fail "O1: rc=$rc1 topic=$t1 out=$(tr '\n' '|' <"$tmp/o1.out")"
fi
rm -rf "$dir"

# -------- O2: AUTO fallback (unreachable host) → LOUD line + DELIVERED via hub leg --------
# closed-port peer hub → probe no → fallback posts to the LOCAL hub; self-ack there
# → DELIVERED. The LOUD FALLBACK line must appear.
echo "O2: auto + unreachable peer (127.0.0.1:1) → FALLBACK line + DELIVERED via hub leg"
dir="$(make_fixture "127.0.0.1:1")"
cidO2="cid-o2-$run_tag"
(
    set +e
    LISTENERS_VERB="$dir/fake-fleet.sh" "$SCRIPT" --to "$run_tag" --message "fallback O2" \
        --conversation-id "$cidO2" --timeout 8 --max-rings 2 >"$tmp/o2.out" 2>&1
    echo $? >"$tmp/o2.rc"
) &
bg=$!; sleep 2
t2="$(topic_of "$tmp/o2.out")"; o2off="$(offset_of "$tmp/o2.out")"
if [ -n "$t2" ]; then
    # fallback posted to the LOCAL hub → ack locally (no --hub).
    "$TERMLINK" channel post "$t2" --msg-type receipt \
        --metadata conversation_id="$cidO2" --metadata up_to="${o2off:-0}" \
        --metadata stage=delivered --ensure-topic --json >/dev/null 2>&1
fi
wait "$bg" || true
rc2="$(cat "$tmp/o2.rc" 2>/dev/null || echo X)"
if [ "$rc2" = "0" ] \
    && grep -qF "FALLBACK host 127.0.0.1:1 unreachable" "$tmp/o2.out" \
    && grep -q "DELIVERED" "$tmp/o2.out"; then
    pass "O2: loud fallback + delivered via hub leg (rc=0), topic=$t2"
else
    fail "O2: rc=$rc2 topic=$t2 out=$(tr '\n' '|' <"$tmp/o2.out")"
fi
rm -rf "$dir"

# -------- O3: --transport direct + unreachable host → loud FAIL, no post --------
echo "O3: --transport direct + down host (127.0.0.1:1) → loud FAIL exit 3, no post"
dir="$(make_fixture "127.0.0.1:1")"
out="$(LISTENERS_VERB="$dir/fake-fleet.sh" "$SCRIPT" --transport direct --to "$run_tag" \
        --message "direct-fail O3" --timeout 2 --max-rings 1 2>&1)"; rc=$?
if [ "$rc" -eq 3 ] \
    && printf '%s' "$out" | grep -qF "unreachable" \
    && printf '%s' "$out" | grep -qF "no fallback under direct" \
    && ! printf '%s' "$out" | grep -qF "posted turn to"; then
    pass "O3: direct+down fails loud (rc=3), never posted, never fell back"
else
    fail "O3: rc=$rc out=$(printf '%s' "$out" | tr '\n' '|')"
fi
rm -rf "$dir"

# -------- O4: --transport hub + down remote → NO fallback (byte-for-byte escape hatch) --------
# hub transport never probes and never falls back; the TCP post to the down hub
# hard-fails exactly as pre-V6 (die exit 2). Proves hub is unchanged.
echo "O4: --transport hub + down remote (127.0.0.1:1) → no fallback, hard-fails at post"
dir="$(make_fixture "127.0.0.1:1")"
out="$(LISTENERS_VERB="$dir/fake-fleet.sh" "$SCRIPT" --transport hub --to "$run_tag" \
        --message "hub O4" --timeout 2 --max-rings 1 2>&1)"; rc=$?
if [ "$rc" -eq 2 ] \
    && printf '%s' "$out" | grep -qF "channel post failed" \
    && ! printf '%s' "$out" | grep -qF "FALLBACK" \
    && ! printf '%s' "$out" | grep -qF "transport-plan"; then
    pass "O4: hub escape hatch never falls back, no plan line (rc=2, unchanged)"
else
    fail "O4: rc=$rc out=$(printf '%s' "$out" | tr '\n' '|')"
fi
rm -rf "$dir"

# -------- O5: default transport IS auto (no --transport flag) → fallback fires --------
# Same down fixture, NO --transport flag. If the default is auto, the unreachable
# host triggers the LOUD FALLBACK line. No self-ack → it then FAILs (store-and-
# forward), which is the honest deferred-delivery outcome. Proves AC1's default flip.
echo "O5: no --transport flag + down host → default auto → LOUD FALLBACK (AC1 default flip)"
dir="$(make_fixture "127.0.0.1:1")"
out="$(LISTENERS_VERB="$dir/fake-fleet.sh" "$SCRIPT" --to "$run_tag" \
        --message "default O5" --timeout 2 --max-rings 1 2>&1)"; rc=$?
if [ "$rc" -eq 3 ] \
    && printf '%s' "$out" | grep -qF "FALLBACK host 127.0.0.1:1 unreachable" \
    && printf '%s' "$out" | grep -qF "store-and-forward"; then
    pass "O5: default is auto — unreachable host fell back loud (rc=3, deferred)"
else
    fail "O5: rc=$rc out=$(printf '%s' "$out" | tr '\n' '|')"
fi
rm -rf "$dir"

echo ""
echo "Results: $PASS pass / $FAIL fail"
[ "$FAIL" -eq 0 ]
