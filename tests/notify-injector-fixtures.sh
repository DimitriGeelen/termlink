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

# stub termlink: records argv, succeeds.
#
# T-3079: `channel subscribe` must MODEL THE HUB, not return silence. The injector
# now reads its L3 back from the topic instead of trusting notify-ack-read's exit
# code, because that code means "posted OR already acked" — a zero that was being
# reported as "L3 posted" when nothing had been written. A stub that returned
# nothing here would make the read-back fail always, and the tempting fix (a seam
# that skips it) would delete the very assertion T9 exists to make.
#
# So the stub replays what was posted: if a stage=read receipt carrying the earned
# evidence has already gone through this stub, subscribe returns it. Nothing was
# posted -> subscribe returns nothing -> the read-back correctly refuses.
cat > "$WORK/bin/termlink" <<EOS
#!/usr/bin/env bash
printf '%s\n' "\$*" >> "$WORK/calls.log"
case "\$1 \$2" in
  "channel subscribe")
      # A real topic is never empty — it holds at least the message we are
      # injecting. Emitting a content envelope unconditionally keeps "topic
      # unreadable" and "no L3 on the topic" as DISTINCT outcomes; collapsing
      # them let a mutation of the read-back survive the suite (found by
      # mutation testing this very fixture).
      printf '{"msg_type":"note","sender_id":"peer","offset":1,"metadata":{}}\n'
      up_to="\$(grep -o 'up_to=[0-9]*' "$WORK/calls.log" 2>/dev/null | tail -1 | cut -d= -f2)"
      if grep -q 'evidence=idle-gated-inject' "$WORK/calls.log" 2>/dev/null; then
          printf '{"msg_type":"receipt","sender_id":"peer","offset":9001,"metadata":{"stage":"read","up_to":"%s","evidence":"idle-gated-inject"}}\n' "\${up_to:-0}"
      fi
      ;;
esac
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

# ---------------------------------------------------------------------------
# T-3071 (arc-011 S8) — priority band ordering.
#
# Note what T1..T17 above now prove for free: mkqueue() builds a table with NO
# priority column, so every one of them exercises the PRE-MIGRATION fallback path.
# Their continued passing is the back-compat assertion, not a separate case.
#
# The band cases below need a column, so they build their own queue.
# ---------------------------------------------------------------------------
mkqueue_p() { # queue WITH the T-3071 priority column
    rm -f "$JOURNAL"
    sqlite3 "$JOURNAL" "CREATE TABLE messages (topic TEXT, offset INTEGER, conversation_id TEXT DEFAULT '', sender_id TEXT DEFAULT '', msg_type TEXT DEFAULT '', ts INTEGER DEFAULT 0, payload TEXT DEFAULT '', observed_addr TEXT DEFAULT '', priority INTEGER DEFAULT 0);"
}
addp() { # addp <offset> <ts> <priority> <body>
    sqlite3 "$JOURNAL" "INSERT INTO messages VALUES ('$TOPIC', $1, '', 'f$1', 'note', $2, '$4', '', $3);"
}
# no INJECTOR_TEST_INJECT_RC — that seam SKIPS the real call and the stub would
# record nothing, so the grep would pass vacuously (the T13 lesson).
run_pick() {
    rm -f "$ND/.ag.injected-seen"; : > "$WORK/calls.log"
    INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY \
      INJECTOR_TEST_VERIFY_STATE=BUSY run >/dev/null 2>&1
}
picked() { grep -o 'PICK_[A-Z]*' "$WORK/calls.log" | head -1; }

echo "T18: equal priority falls back to FIFO (the band decides nothing here)"
mkqueue_p; mkflag 20000
addp 30 900 2 PICK_WRONG
addp 10  50 2 PICK_RIGHT
run_pick
[ "$(picked)" = "PICK_RIGHT" ] && pass "T18 same band -> oldest first" \
                               || fail "T18 got '$(picked)' want PICK_RIGHT"

echo "T19 [THE POINT OF THE SLICE]: a higher band wins regardless of arrival order"
mkqueue_p; mkflag 21000
addp 10  50 0 PICK_WRONG
addp 30 900 5 PICK_RIGHT
run_pick
[ "$(picked)" = "PICK_RIGHT" ] && pass "T19 newest-but-urgent outranks oldest-but-normal" \
                               || fail "T19 got '$(picked)' want PICK_RIGHT"

echo "T20: a NEGATIVE band sorts BEHIND normal (deprioritise is real, not a no-op)"
mkqueue_p; mkflag 22000
addp 10  50 -5 PICK_WRONG
addp 30 900  0 PICK_RIGHT
run_pick
[ "$(picked)" = "PICK_RIGHT" ] && pass "T20 negative band yields to normal" \
                               || fail "T20 got '$(picked)' want PICK_RIGHT"

echo "T21 [FAIL-SAFE]: a NULL priority sorts as normal, never ahead of a real band"
# A row written by a pre-T-3071 mirror into a migrated table can be NULL. If COALESCE
# were dropped, NULL sorts LAST under DESC in SQLite — which would look 'safe' here but
# silently demote every legacy message below anything a peer marks. Pin the behaviour.
mkqueue_p; mkflag 23000
sqlite3 "$JOURNAL" "INSERT INTO messages VALUES ('$TOPIC', 10, '', 'f10', 'note', 50, 'PICK_RIGHT', '', NULL);"
addp 30 900 0 PICK_WRONG
run_pick
[ "$(picked)" = "PICK_RIGHT" ] && pass "T21 NULL == band 0, ties break by FIFO" \
                               || fail "T21 got '$(picked)' want PICK_RIGHT"

echo "T22: a meta envelope is still not work, whatever band it claims"
mkqueue_p; mkflag 24000
sqlite3 "$JOURNAL" "INSERT INTO messages VALUES ('$TOPIC', 10, '', 'f10', 'receipt', 50, 'PICK_WRONG', '', 9);"
addp 30 900 0 PICK_RIGHT
run_pick
[ "$(picked)" = "PICK_RIGHT" ] && pass "T22 priority cannot promote a receipt into the queue" \
                               || fail "T22 got '$(picked)' want PICK_RIGHT"

echo "T23 [outcome 2]: an empty priority-aware queue still exits 2, not a vacuous pass"
mkqueue_p; mkflag 25000; rm -f "$ND/.ag.injected-seen"
INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY run >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T23 rc=$rc" || fail "T23 expected 2 got $rc"

echo "T24 [THE FALSE CLAIM]: ack returns 0 WITHOUT posting -> rc=5, never 'L3 posted'"
# Reproduces a defect found live on 2026-09-22, not an imagined one. The L3 guard
# stood at 145 while the queue served offset 0, so notify-ack-read took its
# "already acked ... nothing to do" branch and exited 0 — its contract says
# "0 posted (OR already acked)" — and the injector printed
# "L3 posted (evidence=idle-gated-inject)" for a receipt that did not exist.
# Here the guard is pre-seeded high to force that same branch. Nothing is posted,
# so the stub's subscribe returns nothing, and the read-back must refuse.
mkqueue; mkflag 30000; rm -f "$ND/.ag.injected-seen"; : > "$WORK/calls.log"
printf '999\n' > "$ND/.ag.dm_aaaa_bbbb.l3read"
OUT="$(INJECTOR_TEST_SESSION_STATE=present INJECTOR_TEST_PTY_STATE=READY \
       INJECTOR_TEST_INJECT_RC=0 INJECTOR_TEST_VERIFY_STATE=BUSY run 2>&1)"; rc=$?
if [ "$rc" -eq 5 ] && ! printf '%s' "$OUT" | grep -q 'L3 confirmed'; then
    pass "T24 rc=5 and no success claim — a no-op ack is not a posted receipt"
else
    fail "T24 expected rc=5 with no success claim, got rc=$rc: $OUT"
fi
rm -f "$ND/.ag.dm_aaaa_bbbb.l3read"

echo
echo "notify-injector fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
