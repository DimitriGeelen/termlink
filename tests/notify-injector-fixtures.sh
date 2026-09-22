#!/usr/bin/env bash
# guard-layer: source
# T-3069 — fixtures for scripts/notify-injector.sh.
#
# Hermetic: fixture flag, fixture journal, stub `termlink`, and test seams for the
# PTY/session state. No live session, no hub, nothing injected anywhere real.
#
# THE LOAD-BEARING CASES ARE THE ONES THAT MUST NOT COLLAPSE INTO EACH OTHER.
# The injector has five outcomes and each pair that could be merged has a known
# failure behind it:
#   3 not-running  vs  4 busy      merging them retries forever at a dead session
#   4 deferred     vs  5 unverified merging them hides a message stuck unsubmitted
#   0 verified     vs  5 unverified merging them tells the sender a lie (T-2396)
#   1 nothing-to-do vs 2 tooling   merging them turns a broken journal into silence
# Every one of those is pinned below.
set -u

INJ="${INJ:-scripts/notify-injector.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
ND="$WORK/notify"; mkdir -p "$ND" "$WORK/bin"
JOURNAL="$WORK/journal.sqlite"
TOPIC="dm:aaaa:bbbb"

command -v sqlite3 >/dev/null 2>&1 || { echo "SKIP: sqlite3 not available"; exit 0; }

# stub termlink: records argv, succeeds
cat > "$WORK/bin/termlink" <<EOS
#!/usr/bin/env bash
printf '%s\n' "\$*" >> "$WORK/calls.log"
exit 0
EOS
chmod +x "$WORK/bin/termlink"

mkflag() { # mkflag <mail_ts>
    printf 'pending=0\nlatest_topic=\nts=%s\nlast_mail_ts=%s\nlast_mail_topic=%s\n' \
        "$1" "$1" "$TOPIC" > "$ND/ag.flag"
}
mkqueue() {
    rm -f "$JOURNAL"
    sqlite3 "$JOURNAL" "CREATE TABLE messages (topic TEXT, offset INTEGER, conversation_id TEXT DEFAULT '', sender_id TEXT DEFAULT '', msg_type TEXT DEFAULT '', ts INTEGER DEFAULT 0, payload TEXT DEFAULT '', observed_addr TEXT DEFAULT '');"
    sqlite3 "$JOURNAL" "INSERT INTO messages VALUES ('$TOPIC', 7, '', 'cafebabecafebabe', 'note', 100, 'hello from a peer', '');"
}
run() { # run [extra args...]
    PATH="$WORK/bin:$PATH" TERMLINK="$WORK/bin/termlink" \
    bash "$INJ" --agent-id ag --session tl-fix --notify-dir "$ND" --journal "$JOURNAL" "$@"
}

echo "T1: --help exits 0"
bash "$INJ" --help >/dev/null 2>&1 && pass "T1" || fail "T1"

echo "T2: missing --session exits 2 (a PTY is not optional)"
bash "$INJ" --agent-id x >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T2 rc=$rc" || fail "T2 expected 2 got $rc"

echo "T3 [outcome 1]: no new arrival is 'nothing to do', NOT an error"
mkflag 1000; mkqueue
printf '2000\n' > "$ND/.ag.injected-seen"     # marker AHEAD of the arrival
run >/dev/null 2>&1; rc=$?
[ "$rc" -eq 1 ] && pass "T3 rc=$rc" || fail "T3 expected 1 got $rc"

echo "T4 [outcome 3]: AGENT NOT RUNNING is its own code, not 'busy'"
mkflag 5000; mkqueue; rm -f "$ND/.ag.injected-seen"
OUT="$(INJECTOR_TEST_SESSION_STATE=absent run 2>&1)"; rc=$?
if [ "$rc" -eq 3 ] && printf '%s' "$OUT" | grep -q 'AGENT NOT RUNNING'; then
    pass "T4 rc=$rc — a dead session is not a busy one"
else
    fail "T4 expected 3/AGENT NOT RUNNING got $rc: $OUT"
fi

echo "T5 [outcome 4]: BUSY defers"
OUT="$(INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=BUSY run 2>&1)"; rc=$?
[ "$rc" -eq 4 ] && pass "T5 rc=$rc" || fail "T5 expected 4 got $rc: $OUT"

echo "T6 [FAIL-SAFE]: UNKNOWN defers and NEVER injects"
: > "$WORK/calls.log"
OUT="$(INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=UNKNOWN run 2>&1)"; rc=$?
INJECTS="$(grep -c '^inject ' "$WORK/calls.log" 2>/dev/null)"; INJECTS="${INJECTS:-0}"
if [ "$rc" -eq 4 ] && [ "$INJECTS" -eq 0 ]; then
    pass "T6 rc=$rc, 0 injections — ambiguity never resolves to READY"
else
    fail "T6 UNKNOWN INJECTED or wrong rc — rc=$rc injects=$INJECTS"
fi

echo "T7 [outcome 5, THE IMPORTANT ONE]: injected but NOT verified posts NO L3"
: > "$WORK/calls.log"
OUT="$(INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY \
       INJECTOR_TEST_INJECT_RC=0 INJECTOR_TEST_VERIFY_STATE=READY run 2>&1)"; rc=$?
L3="$(grep -c 'stage=read' "$WORK/calls.log" 2>/dev/null)"; L3="${L3:-0}"
if [ "$rc" -eq 5 ] && [ "$L3" -eq 0 ]; then
    pass "T7 rc=$rc and NO L3 — the sender is not told a message was read when it may be unsubmitted"
else
    fail "T7 UNVERIFIED INJECTION CLAIMED L3 — rc=$rc l3posts=$L3"
fi

echo "T8 [outcome 5]: the unverified message names the T-2396 risk"
printf '%s' "$OUT" | grep -q 'UNSUBMITTED' \
    && pass "T8 the operator is told what may have happened" \
    || fail "T8 unverified path does not explain the risk: $OUT"

echo "T9 [outcome 0]: a VERIFIED injection posts exactly one L3, evidence=idle-gated-inject"
: > "$WORK/calls.log"; rm -f "$ND"/.ag.*.l3read
OUT="$(INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY \
       INJECTOR_TEST_INJECT_RC=0 INJECTOR_TEST_VERIFY_STATE=BUSY run 2>&1)"; rc=$?
L3="$(grep -c 'stage=read' "$WORK/calls.log" 2>/dev/null)"; L3="${L3:-0}"
EV="$(grep -c 'evidence=idle-gated-inject' "$WORK/calls.log" 2>/dev/null)"; EV="${EV:-0}"
if [ "$rc" -eq 0 ] && [ "$L3" -eq 1 ] && [ "$EV" -eq 1 ]; then
    pass "T9 rc=$rc, exactly 1 L3 with the earned evidence kind"
else
    fail "T9 expected 0/1/1 got rc=$rc l3=$L3 ev=$EV: $OUT"
fi

echo "T10: a verified injection advances the marker (no replay)"
: > "$WORK/calls.log"
INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY \
  INJECTOR_TEST_INJECT_RC=0 INJECTOR_TEST_VERIFY_STATE=BUSY run >/dev/null 2>&1; rc=$?
[ "$rc" -eq 1 ] && pass "T10 second run is 'nothing to do' (rc=$rc)" \
                || fail "T10 REPLAYED the same arrival — rc=$rc"

echo "T11 [outcome 2]: flag says mail arrived but journal is empty -> TOOLING, not silence"
mkflag 9000; rm -f "$ND/.ag.injected-seen"
rm -f "$JOURNAL"
sqlite3 "$JOURNAL" "CREATE TABLE messages (topic TEXT, offset INTEGER, conversation_id TEXT DEFAULT '', sender_id TEXT DEFAULT '', msg_type TEXT DEFAULT '', ts INTEGER DEFAULT 0, payload TEXT DEFAULT '', observed_addr TEXT DEFAULT '');"
OUT="$(INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY run 2>&1)"; rc=$?
if [ "$rc" -eq 2 ] && printf '%s' "$OUT" | grep -q 'QUEUE EMPTY'; then
    pass "T11 rc=$rc — an inconsistent journal is reported, not swallowed"
else
    fail "T11 expected 2/QUEUE EMPTY got $rc: $OUT"
fi

echo "T12: meta envelopes are NOT work (a receipt must never be injected)"
mkflag 11000; rm -f "$ND/.ag.injected-seen"
sqlite3 "$JOURNAL" "INSERT INTO messages VALUES ('$TOPIC', 8, '', 'deadbeefdeadbeef', 'receipt', 200, 'ack', '');"
OUT="$(INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY run 2>&1)"; rc=$?
if [ "$rc" -eq 2 ]; then
    pass "T12 a topic holding only receipts has no work"
else
    fail "T12 a receipt was treated as injectable work — rc=$rc: $OUT"
fi

echo "T13: FIFO order — the OLDEST content message is taken first"
mkflag 13000; rm -f "$ND/.ag.injected-seen"; : > "$WORK/calls.log"
sqlite3 "$JOURNAL" "INSERT INTO messages VALUES ('$TOPIC', 20, '', 'f1', 'note', 900, 'NEWER', '');"
sqlite3 "$JOURNAL" "INSERT INTO messages VALUES ('$TOPIC', 5,  '', 'f2', 'note', 50,  'OLDEST', '');"
# NB: no INJECTOR_TEST_INJECT_RC here — that seam SKIPS the real call, so the stub
# would record nothing and the assertion would pass vacuously. The stub returns 0.
INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY \
  INJECTOR_TEST_VERIFY_STATE=BUSY run >/dev/null 2>&1
if grep -q 'OLDEST' "$WORK/calls.log" && ! grep -q 'NEWER' "$WORK/calls.log"; then
    pass "T13 flat FIFO by (ts, offset)"
else
    fail "T13 wrong message taken: $(grep '^inject' "$WORK/calls.log" | head -1)"
fi

echo "T14: --dry-run decides and injects nothing"
mkflag 15000; rm -f "$ND/.ag.injected-seen"; : > "$WORK/calls.log"
OUT="$(INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY run --dry-run 2>&1)"; rc=$?
INJECTS="$(grep -c '^inject ' "$WORK/calls.log" 2>/dev/null)"; INJECTS="${INJECTS:-0}"
if [ "$rc" -eq 0 ] && [ "$INJECTS" -eq 0 ] && printf '%s' "$OUT" | grep -q 'DRY-RUN'; then
    pass "T14 dry-run is side-effect free"
else
    fail "T14 dry-run injected or misreported — rc=$rc injects=$INJECTS"
fi

echo "T15: the injected text names its provenance (peer, topic, offset)"
mkflag 17000; rm -f "$ND/.ag.injected-seen"; : > "$WORK/calls.log"
INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY \
  INJECTOR_TEST_VERIFY_STATE=BUSY run >/dev/null 2>&1
if grep -q 'peer-message' "$WORK/calls.log"; then
    pass "T15 an injected peer message is distinguishable from operator input"
else
    fail "T15 no provenance marker: $(grep '^inject' "$WORK/calls.log" | head -1)"
fi

echo "T16: a failed inject posts no L3"
mkflag 19000; rm -f "$ND/.ag.injected-seen"; : > "$WORK/calls.log"
OUT="$(INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY \
       INJECTOR_TEST_INJECT_RC=1 run 2>&1)"; rc=$?
L3="$(grep -c 'stage=read' "$WORK/calls.log" 2>/dev/null)"; L3="${L3:-0}"
if [ "$rc" -eq 5 ] && [ "$L3" -eq 0 ]; then
    pass "T16 rc=$rc, no L3 on a failed inject"
else
    fail "T16 expected 5 and no L3 — rc=$rc l3=$L3"
fi

echo "T17 [SHARED CLASSIFIER]: the injector sources the lib, it does not copy it"
if grep -q 'lib/pty-state.sh' "$INJ" && ! grep -q 'esctointerrupt' "$INJ"; then
    pass "T17 one copy of the heuristic, shared with the push-waker"
else
    fail "T17 the classifier has been duplicated into the injector"
fi

echo
echo "notify-injector fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
