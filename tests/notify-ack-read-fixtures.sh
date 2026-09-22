#!/usr/bin/env bash
# guard-layer: source
# T-3067 — fixtures for scripts/notify-ack-read.sh (L3 stage=read) and for the
# LADDER stage of scripts/notify-rail-e2e.sh.
#
# Hermetic: no hub, no peer. The post path is exercised through a stub `termlink`
# placed first on PATH, so the guard/idempotency logic is testable without writing
# to a real topic.
#
# THE LOAD-BEARING CASES ARE THE REFUSALS. L3 asserts a message reached a PROMPT.
# The failure that matters is not "it did not post" — it is "it posted when it
# should not have", because that tells the sender something false and the sender
# then stops asking. T-2396 proved live that a bare `termlink inject` returns
# BEFORE submission: on a busy or manual-accept session the text lands unsubmitted
# and is discarded. So the evidence gate is the integrity of the whole rung, and
# it is pinned hardest here.
set -u

ACK="${ACK:-scripts/notify-ack-read.sh}"
E2E="${E2E:-scripts/notify-rail-e2e.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
ND="$WORK/notify"; mkdir -p "$ND"

# A stub `termlink` that records its argv and succeeds.
mkdir -p "$WORK/bin"
cat > "$WORK/bin/termlink" <<EOS
#!/usr/bin/env bash
printf '%s\n' "\$*" >> "$WORK/posts.log"
exit 0
EOS
chmod +x "$WORK/bin/termlink"

# A stub that REJECTS every post, for the hub-rejection path.
cat > "$WORK/bin/termlink-reject" <<'EOS'
#!/usr/bin/env bash
exit 1
EOS
chmod +x "$WORK/bin/termlink-reject"

ackrun() { NOTIFY_DIR="$ND" TERMLINK="$WORK/bin/termlink" bash "$ACK" "$@"; }

echo "T1: --help exits 0"
bash "$ACK" --help >/dev/null 2>&1 && pass "T1" || fail "T1 expected 0"

# ---- the evidence gate — the integrity of the rung -------------------------
echo "T2 [GATE]: NO evidence must refuse (exit 2), posting nothing"
: > "$WORK/posts.log"
ackrun --topic dm:x --up-to 5 >/dev/null 2>&1; rc=$?
if [ "$rc" -eq 2 ] && [ ! -s "$WORK/posts.log" ]; then
    pass "T2 refused and posted nothing (rc=$rc)"
else
    fail "T2 THE EVIDENCE GATE IS OPEN — rc=$rc, posts=$(wc -l < "$WORK/posts.log")"
fi

echo "T3 [GATE]: 'blind-inject' is NOT a valid evidence kind"
: > "$WORK/posts.log"
ackrun --topic dm:x --up-to 5 --evidence blind-inject >/dev/null 2>&1; rc=$?
if [ "$rc" -eq 2 ] && [ ! -s "$WORK/posts.log" ]; then
    pass "T3 refused (rc=$rc) — there is no evidence kind for 'I sent it and hoped'"
else
    fail "T3 blind-inject was ACCEPTED — rc=$rc"
fi

echo "T4 [GATE]: the refusal explains WHY, naming the unsubmitted-inject trap"
OUT="$(ackrun --topic dm:x --up-to 5 2>&1)"
printf '%s' "$OUT" | grep -q 'unsubmitted' \
    && pass "T4 refusal names the T-2396 failure" \
    || fail "T4 refusal does not explain the trap: $OUT"

echo "T5: each valid evidence kind is accepted"
for ev in idle-gated-inject observed-turn operator; do
    : > "$WORK/posts.log"; rm -f "$ND"/.*.l3read 2>/dev/null
    ackrun --topic "dm:ev-$ev" --up-to 7 --evidence "$ev" --agent-id a --quiet >/dev/null 2>&1
    rc=$?
    [ "$rc" -eq 0 ] || { fail "T5 $ev rejected (rc=$rc)"; continue; }
done
pass "T5 all three evidence kinds accepted"

# ---- what gets posted ------------------------------------------------------
echo "T5c [REGRESSION]: wake-consumer is NO LONGER valid evidence"
: > "$WORK/posts.log"; rm -f "$ND"/.*.l3read 2>/dev/null
ackrun --topic dm:wc --up-to 5 --evidence wake-consumer --agent-id a >/dev/null 2>&1; rc=$?
if [ "$rc" -eq 2 ] && [ ! -s "$WORK/posts.log" ]; then
    pass "T5c refused (rc=$rc) — noticing a flag is not evidence of reaching a prompt"
else
    fail "T5c wake-consumer STILL ACCEPTED — L3 can assert a falsehood again (rc=$rc)"
fi

echo "T6: the posted envelope carries stage=read, up_to and evidence"
: > "$WORK/posts.log"; rm -f "$ND"/.*.l3read 2>/dev/null
ackrun --topic dm:shape --up-to 11 --evidence operator --agent-id a --quiet >/dev/null 2>&1
LOG="$(cat "$WORK/posts.log")"
if printf '%s' "$LOG" | grep -q 'stage=read' \
   && printf '%s' "$LOG" | grep -q 'up_to=11' \
   && printf '%s' "$LOG" | grep -q 'evidence=operator' \
   && printf '%s' "$LOG" | grep -q 'msg-type receipt'; then
    pass "T6 envelope shape correct"
else
    fail "T6 wrong envelope: $LOG"
fi

echo "T7 [INVARIANT]: it must NEVER post stage=delivered (L2 is the sidecar's)"
if printf '%s' "$LOG" | grep -q 'stage=delivered'; then
    fail "T7 IT POSTED L2 — a second writer to that cursor races the sidecar"
else
    pass "T7 no L2 emitted"
fi

echo "T8 [IDEMPOTENCY]: a second run at the same offset posts nothing"
: > "$WORK/posts.log"
ackrun --topic dm:shape --up-to 11 --evidence operator --agent-id a --quiet >/dev/null 2>&1; rc=$?
if [ "$rc" -eq 0 ] && [ ! -s "$WORK/posts.log" ]; then
    pass "T8 no ack-spam (rc=$rc, 0 posts)"
else
    fail "T8 re-posted — ack spam: rc=$rc posts=$(cat "$WORK/posts.log")"
fi

echo "T9: a HIGHER offset does post again"
: > "$WORK/posts.log"
ackrun --topic dm:shape --up-to 12 --evidence operator --agent-id a --quiet >/dev/null 2>&1
[ -s "$WORK/posts.log" ] && pass "T9 advancing watermark posts" || fail "T9 did not post for a newer offset"

echo "T10 [INVARIANT]: the guard file is in its own .l3read namespace"
ls "$ND"/.a.*.l3read >/dev/null 2>&1 \
    && pass "T10 guard namespaced, cannot be confused with the L2 guard" \
    || fail "T10 guard file missing or misnamed: $(ls -a "$ND" | tr '\n' ' ')"

echo "T11: --up-to is required (never invent an offset)"
ackrun --topic dm:x --evidence operator >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T11 rc=$rc" || fail "T11 expected 2 got $rc"

echo "T12: a hub rejection exits 3 and is NOT swallowed"
rm -f "$ND"/.*.l3read 2>/dev/null
NOTIFY_DIR="$ND" TERMLINK="$WORK/bin/termlink-reject" \
    bash "$ACK" --topic dm:rej --up-to 3 --evidence operator --agent-id a >/dev/null 2>&1; rc=$?
[ "$rc" -eq 3 ] && pass "T12 rc=$rc (a quiet failure would look like 'never read')" \
                || fail "T12 expected 3 got $rc"

echo "T13: a rejected post does NOT write the guard (so a retry can work)"
if ls "$ND"/.a.dm_rej.l3read >/dev/null 2>&1; then
    fail "T13 guard written despite rejection — the retry would be suppressed"
else
    pass "T13 no guard on failure"
fi

# ---- the LADDER stage ------------------------------------------------------
echo "T14 [LADDER]: L2 receipts only -> L2-ONLY, and that is NOT green"
cat > "$WORK/l2.ndjson" <<'EOS'
{"msg_type":"receipt","offset":1,"metadata":{"stage":"delivered","up_to":"0"}}
{"msg_type":"receipt","offset":2,"metadata":{"stage":"delivered","up_to":"1"}}
EOS
OUT="$(NOTIFY_E2E_TEST_PRECOND=1 NOTIFY_E2E_TEST_LADDER="$WORK/l2.ndjson" \
       bash "$E2E" --stages ladder 2>&1)"; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$OUT" | grep -q 'L2-ONLY'; then
    pass "T14 a ladder stopping at the mailbox is not reported green (rc=$rc)"
else
    fail "T14 L2-ONLY WAS TREATED AS A PASS — rc=$rc: $OUT"
fi

echo "T15 [LADDER]: an L3 receipt present -> PASS"
cat > "$WORK/l3.ndjson" <<'EOS'
{"msg_type":"receipt","offset":1,"metadata":{"stage":"delivered","up_to":"0"}}
{"msg_type":"receipt","offset":2,"metadata":{"stage":"read","up_to":"1","evidence":"idle-gated-inject"}}
EOS
OUT="$(NOTIFY_E2E_TEST_PRECOND=1 NOTIFY_E2E_TEST_LADDER="$WORK/l3.ndjson" \
       bash "$E2E" --stages ladder 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$OUT" | grep -q 'reached L3'; then
    pass "T15 rc=$rc reached L3"
else
    fail "T15 expected 0/reached L3 got $rc: $OUT"
fi

echo "T15b [LADDER]: a DISAVOWED wake-consumer receipt must NOT hold the stage green"
cat > "$WORK/l3wc.ndjson" <<'EOS'
{"msg_type":"receipt","offset":1,"metadata":{"stage":"delivered","up_to":"0"}}
{"msg_type":"receipt","offset":2,"metadata":{"stage":"read","up_to":"1","evidence":"wake-consumer"}}
EOS
OUT="$(NOTIFY_E2E_TEST_PRECOND=1 NOTIFY_E2E_TEST_LADDER="$WORK/l3wc.ndjson" \
       bash "$E2E" --stages ladder 2>&1)"; rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$OUT" | grep -q 'L2-ONLY'; then
    pass "T15b a receipt from a non-injecting consumer does not count as L3"
else
    fail "T15b A DISAVOWED RECEIPT HELD THE LADDER GREEN — rc=$rc: $OUT"
fi

echo "T16 [LADDER]: no staged receipts at all -> FAIL, not a vacuous pass"
printf '{"msg_type":"note","offset":1}\n' > "$WORK/none.ndjson"
OUT="$(NOTIFY_E2E_TEST_PRECOND=1 NOTIFY_E2E_TEST_LADDER="$WORK/none.ndjson" \
       bash "$E2E" --stages ladder 2>&1)"; rc=$?
[ "$rc" -eq 1 ] && pass "T16 rc=$rc" || fail "T16 expected 1 got $rc: $OUT"

echo
echo "notify-ack-read + LADDER fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
