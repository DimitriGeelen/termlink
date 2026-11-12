#!/usr/bin/env bash
# Fixtures for scripts/session-message-selftest.sh (T-2876; re-based on the TermLink
# rail by T-3711, arc-011 P-27 / B-26 / E-74).
#
# Weighted toward the FIRING cases. A prover that only ever goes green is not a
# prover, so most of what is pinned here is the script correctly reporting a
# BROKEN rail, and the fail-closed paths where it must refuse to answer at all.
#
# All cases drive `assert`, which is the pure-read subcommand. `prepare` and
# `cleanup` spawn/stop a real session, so only their argument-validation paths
# are exercised here.
#
# Seams (PL-213): SESSION_MSG_TEST_TRANSCRIPT_DIR + SESSION_MSG_TEST_AGENTS_JSON +
# SESSION_MSG_TEST_INBOX_JSON (the receiver's TermLink inbox) + SESSION_MSG_TEST_SEND_CMD
# + SESSION_MSG_TEST_WAKE_CMD, so the whole suite runs with no live session, no
# `claude`, no hub and no network. The default rail is termlink (T-3711); the T-2876
# SendMessage rail is kept as `--rail sendmessage` (rollback) and has its own cases.
#
# Mutants (FP 3.1.b): the last section copies the script, breaks one rule per copy and
# re-runs this whole suite against it (SMS_FIXTURE_TARGET); each must turn it red.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="${SMS_FIXTURE_TARGET:-$ROOT/scripts/session-message-selftest.sh}"
PASS=0; FAIL=0
ok(){ if [ "$1" = "$2" ]; then PASS=$((PASS+1)); else FAIL=$((FAIL+1)); echo "  FAIL: $3 (want=$1 got=$2)"; fi; }
has(){ if grep -q "$1" "$2"; then PASS=$((PASS+1)); else FAIL=$((FAIL+1)); echo "  FAIL: $3"; fi; }
hasnt(){ if grep -q "$1" "$2"; then FAIL=$((FAIL+1)); echo "  FAIL: $3"; else PASS=$((PASS+1)); fi; }

TMP="$(mktemp -d "${TMPDIR:-/tmp}/sms-fixtures.XXXXXX")"; trap 'rm -rf "$TMP"' EXIT
SID="aaaaaaaa-1111-2222-3333-444444444444"
SENT="SMS-PROBE-FIXTURE-1"
TO="sms-probe-fixture"
# The receiver's inbox, empty unless a case says otherwise. Exported so no case can
# fall through to a live `fw sidecar inbox`.
printf '{"consults": [], "dm_rails": []}\n' > "$TMP/inbox-empty.json"
export SESSION_MSG_TEST_INBOX_JSON="$TMP/inbox-empty.json"

# Build a transcript dir: $1 = case name, remaining stdin = jsonl body.
mk_transcript(){
  local case="$1"
  mkdir -p "$TMP/$case/-opt-termlink"
  cat > "$TMP/$case/-opt-termlink/$SID.jsonl"
}

# Canned `claude agents --json`. Schema is the MEASURED one (T-2876): the fields
# that actually exist are id/cwd/kind/startedAt/sessionId/name/state/status.
# `waitingFor` is deliberately absent everywhere — it does not exist in the real
# output (0 of 56 agents carried it), and a fixture that invented it would let a
# regression to the phantom field pass.
agents(){ # $1 = file, $2 = state, $3 = status
  cat > "$1" <<AJ
[{"id":"deadbeef","cwd":"/opt/termlink","kind":"background","startedAt":1781305212348,
  "sessionId":"$SID","name":"probe target","state":"$2","status":"$3"}]
AJ
}
AG_REST="$TMP/agents-rest.json";    agents "$AG_REST" blocked idle
AG_BUSY="$TMP/agents-busy.json";    agents "$AG_BUSY" working busy
: > "$TMP/agents-empty.json"; printf '[]\n' > "$TMP/agents-empty.json"

run(){ # run assert against a case dir; echoes nothing, sets $RC / writes $TMP/out
  local case="$1"; shift
  SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/$case" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --to "$TO" "$@" \
    > "$TMP/out" 2>&1
  RC=$?
}

# ── 1. DELIVERED — the sentinel is a `user` turn. The only green verdict. ────────
mk_transcript delivered <<J
{"type":"queue-operation","operation":"enqueue","content":"$SENT","sessionId":"$SID"}
{"type":"user","message":{"role":"user","content":"$SENT"},"sessionId":"$SID"}
J
run delivered; ok 0 "$RC" "DELIVERED exits 0"
has "DELIVER=PASS" "$TMP/out" "DELIVERED names the DELIVER stage as passing"

# 2. A user turn WITHOUT any queue-operation is still DELIVERED (direct delivery).
mk_transcript delivered_direct <<J
{"type":"user","message":{"role":"user","content":"$SENT"},"sessionId":"$SID"}
J
run delivered_direct; ok 0 "$RC" "user turn alone is DELIVERED"

# 3. user turn wins over queue-operation — a drained message must never be
#    reported as merely enqueued.
run delivered --id deadbeef; ok 0 "$RC" "user turn beats queue-operation"

# ── 4. ENQUEUED / BLOCKED — accepted by the queue, never drained. ───────────────
# This is the T-2875 shape: the send succeeded, the target could not act. It is a
# DIFFERENT fault from not-delivered and must be reported as such.
mk_transcript enqueued <<J
{"type":"queue-operation","operation":"enqueue","content":"$SENT","sessionId":"$SID"}
J
run enqueued; ok 1 "$RC" "ENQUEUED is a failure (exit 1)"
hasnt "UNDELIVERED" "$TMP/out" "ENQUEUED is never reported as UNDELIVERED"

# 5. A target AT REST with the message still queued is BLOCKED: delivery succeeded
#    and the target never drained it. This must be its own verdict, distinguishable
#    from a target that is merely still busy — otherwise the operator cannot tell
#    "go clear the prompt" from "wait a bit longer". Asserting only on the shared
#    "DELIVERY IS FINE" sentence would let the two collapse silently.
SESSION_MSG_TEST_AGENTS_JSON="$AG_REST" SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/enqueued" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --id deadbeef \
  > "$TMP/out" 2>&1; RC=$?
ok 1 "$RC" "a resting target with a queued message exits 1"
has "DELIVERY IS FINE" "$TMP/out" "a stuck target is reported as a delivery success"
has "BLOCKED" "$TMP/out" "a resting target is named BLOCKED, not ENQUEUED"
has "TARGET=FAIL" "$TMP/out" "BLOCKED is attributed to the TARGET stage"
hasnt "may yet drain" "$TMP/out" "BLOCKED does not tell the operator to just wait"

SESSION_MSG_TEST_AGENTS_JSON="$AG_REST" SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/enqueued" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --id deadbeef --json \
  > "$TMP/out" 2>&1
has '"verdict": *"BLOCKED"' "$TMP/out" "json carries the BLOCKED verdict"
has '"stage": *"TARGET"' "$TMP/out" "json attributes BLOCKED to the TARGET stage"

# 6. A BUSY target is a DIFFERENT verdict — it may still drain, so the operator is
#    told to retry rather than to go clear a prompt.
SESSION_MSG_TEST_AGENTS_JSON="$AG_BUSY" SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/enqueued" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --id deadbeef \
  > "$TMP/out" 2>&1; RC=$?
ok 1 "$RC" "a busy target with a queued message exits 1"
has "busy" "$TMP/out" "a busy target is annotated as busy"
has "DRAIN=FAIL" "$TMP/out" "a busy target is attributed to the DRAIN stage"
hasnt "BLOCKED" "$TMP/out" "a busy target is never named BLOCKED"

# 6b. LOAD-BEARING: at-rest and busy must not produce the same output. If the
#     BLOCKED classification is removed, these two become identical and this fails.
SESSION_MSG_TEST_AGENTS_JSON="$AG_REST" SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/enqueued" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --id deadbeef \
  > "$TMP/rest.out" 2>&1
SESSION_MSG_TEST_AGENTS_JSON="$AG_BUSY" SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/enqueued" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --id deadbeef \
  > "$TMP/busy.out" 2>&1
if cmp -s "$TMP/rest.out" "$TMP/busy.out"; then
  FAIL=$((FAIL+1)); echo "  FAIL: BLOCKED collapsed into ENQUEUED (at-rest and busy report identically)"
else PASS=$((PASS+1)); fi

# ── 7. UNDELIVERED — the sentinel never arrived at all. The broken rail. ────────
mk_transcript undelivered <<J
{"type":"user","message":{"role":"user","content":"some other message"},"sessionId":"$SID"}
{"type":"assistant","message":{"role":"assistant","content":"hello"},"sessionId":"$SID"}
J
run undelivered; ok 1 "$RC" "UNDELIVERED exits 1"
has "UNDELIVERED" "$TMP/out" "absent sentinel is named UNDELIVERED"

# 8. LOAD-BEARING: ENQUEUED and UNDELIVERED must not collapse into one verdict.
#    If a future edit makes these identical, this pair fails — which is the whole
#    point of the classification.
run enqueued;     E_OUT="$(cat "$TMP/out")"
run undelivered;  U_OUT="$(cat "$TMP/out")"
if [ "$E_OUT" = "$U_OUT" ]; then FAIL=$((FAIL+1)); echo "  FAIL: ENQUEUED and UNDELIVERED collapsed to one verdict"; else PASS=$((PASS+1)); fi

# 9. LOAD-BEARING: the target-state lookup must not be able to turn an ABSENT
#    sentinel into BLOCKED. `state:blocked` is the ordinary resting state of a
#    background session (43 of 56 measured), so keying BLOCKED off it would make
#    almost every broken rail look like a stuck target — the inverse of the bug
#    this prover exists to prevent.
SESSION_MSG_TEST_AGENTS_JSON="$AG_REST" SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/undelivered" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --id deadbeef --to "$TO" \
  > "$TMP/out" 2>&1; RC=$?
ok 1 "$RC" "absent sentinel with a resting target still exits 1"
has "UNDELIVERED" "$TMP/out" "a resting target never converts UNDELIVERED into BLOCKED"

# ── 10. The sentinel must be a USER turn, not any mention. ──────────────────────
# An assistant turn that happens to echo the sentinel is not delivery. Without
# the type check the prover would pass on its own output.
mk_transcript assistant_echo <<J
{"type":"assistant","message":{"role":"assistant","content":"I will look for $SENT"},"sessionId":"$SID"}
J
run assistant_echo; ok 1 "$RC" "an assistant echo of the sentinel is not delivery"

# 10b. The sentinel must appear in the message CONTENT, not merely somewhere in
#      the record. A `user` record also carries tool results, and a tool that read
#      or echoed the sentinel would otherwise be counted as the message arriving —
#      the prover passing on its own footprint. This is what the decoded-text check
#      buys over a raw substring match on the line.
mk_transcript tool_result_echo <<J
{"type":"user","message":{"role":"user","content":"unrelated prompt"},"toolUseResult":{"stdout":"grep found $SENT here"},"sessionId":"$SID"}
J
run tool_result_echo; ok 1 "$RC" "a sentinel in a tool result is not a delivered message"
has "UNDELIVERED" "$TMP/out" "a tool-result echo is named UNDELIVERED"

# 11. A sentinel in a DIFFERENT session's transcript must not count.
mkdir -p "$TMP/wrong_session/-opt-termlink"
printf '{"type":"user","message":{"content":"%s"},"sessionId":"other"}\n' "$SENT" \
  > "$TMP/wrong_session/-opt-termlink/bbbbbbbb-9999-9999-9999-999999999999.jsonl"
printf '{"type":"user","message":{"content":"unrelated"},"sessionId":"%s"}\n' "$SID" \
  > "$TMP/wrong_session/-opt-termlink/$SID.jsonl"
run wrong_session; ok 1 "$RC" "sentinel in another session's transcript does not count"

# 12. Unparseable JSONL lines are skipped, never fatal — a transcript is written
#     concurrently and a torn last line is normal.
mk_transcript torn <<J
{"type":"user","message":{"content":"$SENT"},"sessionId":"$SID"}
{"type":"user","message":{"content":"$SENT
J
run torn; ok 0 "$RC" "a torn trailing line does not break the scan"

# ── 13. Fail-closed paths. A prover that cannot observe must never say "proven". ─
bash "$CHECK" assert --sentinel "$SENT" >/dev/null 2>&1; ok 2 "$?" "missing --session-id exits 2"
bash "$CHECK" assert --session-id "$SID" >/dev/null 2>&1; ok 2 "$?" "missing --sentinel exits 2"

SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/delivered" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs abc >/dev/null 2>&1
ok 2 "$?" "non-integer --timeout-secs exits 2"

SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/no-such-dir" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 >/dev/null 2>&1
ok 2 "$?" "missing transcript root exits 2"

# 14. Root exists but the session's transcript does not: we cannot see the
#     receiver at all, so this is TOOLING (2), never a delivery verdict (1).
#     Reporting "not delivered" here would be a claim the script cannot support.
mkdir -p "$TMP/no_transcript/-opt-termlink"
SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/no_transcript" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 >/dev/null 2>&1
ok 2 "$?" "an unobservable receiver exits 2, not 1"

bash "$CHECK" bogus-subcommand >/dev/null 2>&1;            ok 2 "$?" "unknown subcommand exits 2"
bash "$CHECK" assert --not-a-flag >/dev/null 2>&1;         ok 2 "$?" "unknown argument exits 2"
bash "$CHECK" cleanup >/dev/null 2>&1;                     ok 2 "$?" "cleanup without --id exits 2"

# 15. A bad test-seam path is tooling, not a silent fallback to the live binary.
SESSION_MSG_TEST_AGENTS_JSON="$TMP/nope.json" SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/enqueued" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --id deadbeef \
  >/dev/null 2>&1
ok 2 "$?" "a missing seam file exits 2 rather than falling back to the live binary"

# 15b. Unparseable agents output must be TOOLING, never a verdict. If it were
#      swallowed, `detail` would come back empty, read as "target at rest", and the
#      script would confidently report BLOCKED from data it failed to parse.
printf 'not json at all\n' > "$TMP/agents-garbage.json"
SESSION_MSG_TEST_AGENTS_JSON="$TMP/agents-garbage.json" SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/enqueued" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --id deadbeef \
  >/dev/null 2>&1
ok 2 "$?" "unparseable agents json exits 2, not a BLOCKED verdict"

# 15c. A well-formed but unexpected shape (object, not list) is also tooling.
printf '{"agents":[]}\n' > "$TMP/agents-shape.json"
SESSION_MSG_TEST_AGENTS_JSON="$TMP/agents-shape.json" SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/enqueued" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --id deadbeef \
  >/dev/null 2>&1
ok 2 "$?" "unexpected agents json shape exits 2"

# 15d. A target absent from the listing is reported honestly as unknown, not as a
#      silent at-rest reading.
SESSION_MSG_TEST_AGENTS_JSON="$TMP/agents-empty.json" SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/enqueued" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 --id deadbeef \
  > "$TMP/out" 2>&1
has "unknown" "$TMP/out" "an unlisted target is reported as unknown state"

# ── 16. --json is machine-readable and carries the verdict. ─────────────────────
run delivered --json
has '"ok": *true' "$TMP/out" "json reports ok:true when delivered"
has '"verdict": *"DELIVERED"' "$TMP/out" "json carries the DELIVERED verdict"
python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$TMP/out" >/dev/null 2>&1
ok 0 "$?" "delivered json parses"

run undelivered --json
has '"ok": *false' "$TMP/out" "json reports ok:false when undelivered"
has '"verdict": *"UNDELIVERED"' "$TMP/out" "json carries the UNDELIVERED verdict"
python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$TMP/out" >/dev/null 2>&1
ok 0 "$?" "undelivered json parses"

run enqueued --json
has '"verdict": *"ENQUEUED"' "$TMP/out" "json carries the ENQUEUED verdict"
python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$TMP/out" >/dev/null 2>&1
ok 0 "$?" "enqueued json parses"

# 17. A sentinel containing shell/JSON metacharacters must not corrupt the
#     envelope — sentinels are generated, but an operator may pass --sentinel.
QSENT='probe "quoted" \back'
mkdir -p "$TMP/quoted/-opt-termlink"
python3 -c '
import json,sys
sid,sent=sys.argv[1],sys.argv[2]
print(json.dumps({"type":"user","message":{"content":sent},"sessionId":sid}))
' "$SID" "$QSENT" > "$TMP/quoted/-opt-termlink/$SID.jsonl"
SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/quoted" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$QSENT" --timeout-secs 0 --json \
  > "$TMP/out" 2>&1
ok 0 "$?" "a sentinel with metacharacters is still matched"
python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$TMP/out" >/dev/null 2>&1
ok 0 "$?" "json stays parseable with a metacharacter sentinel"

# ── 18. The script must NOT claim guard-layer membership: prepare/cleanup have
#        side effects (they spawn and stop a real session).
grep -q '^# guard-layer: source' "$CHECK"
if [ $? -eq 0 ]; then FAIL=$((FAIL+1)); echo "  FAIL: script must not be a guard-layer member (it has side effects)"; else PASS=$((PASS+1)); fi

# 19. The phantom field must stay gone. `waitingFor` does not exist in the real
#     `claude agents --json` (measured 0 of 56). If a future edit reintroduces a
#     dependency on it, the blocked-detection is silently dead again.
if grep -q 'waitingFor' "$CHECK"; then
  grep -q 'does not exist\|phantom\|0 of 56' "$CHECK" \
    && { PASS=$((PASS+1)); } \
    || { FAIL=$((FAIL+1)); echo "  FAIL: waitingFor is referenced without the note that it does not exist"; }
else PASS=$((PASS+1)); fi

# ── 20. prepare's reachability gate (T-2876 live finding). ─────────────────────
# A session listed by `claude agents --json` is NOT necessarily addressable. The
# measured predicate is the presence of a `status` field: that set matched the
# SendMessage peer list exactly, and `claude --bg` produces a record WITHOUT it.
# Handing back an unreachable address would make the send fail for a reason that
# has nothing to do with the rail under test.
cat > "$TMP/agents-mixed.json" <<AJ
[{"id":"aaaa1111","cwd":"/x","kind":"background","sessionId":"$SID","name":"reachable probe","state":"blocked","status":"idle"},
 {"id":"bbbb2222","cwd":"/x","kind":"background","sessionId":"cccccccc-0000-0000-0000-000000000000","name":"listed but unreachable","state":"blocked"}]
AJ

# 20a. a reachable target resolves
SESSION_MSG_TEST_AGENTS_JSON="$TMP/agents-mixed.json" \
  bash "$CHECK" prepare --existing aaaa1111 --sentinel "$SENT" > "$TMP/out" 2>&1
ok 0 "$?" "prepare --existing resolves a reachable target"
has "PREPARE=PASS" "$TMP/out" "prepare reports PASS for a reachable target"
has "reachable probe" "$TMP/out" "prepare names the recipient for the send"

# 20b. a LISTED BUT UNREACHABLE target is refused — this is the whole gate.
SESSION_MSG_TEST_AGENTS_JSON="$TMP/agents-mixed.json" \
  bash "$CHECK" prepare --existing bbbb2222 --sentinel "$SENT" > "$TMP/out" 2>&1
ok 2 "$?" "prepare refuses a listed-but-unreachable target"
has "REACHABLE" "$TMP/out" "the refusal names reachability as the reason"

# 20c. name-substring matching works, not just id prefix
SESSION_MSG_TEST_AGENTS_JSON="$TMP/agents-mixed.json" \
  bash "$CHECK" prepare --existing "reachable probe" --sentinel "$SENT" >/dev/null 2>&1
ok 0 "$?" "prepare matches on a name substring"

# 20d. an ambiguous match is refused rather than silently picking one
cat > "$TMP/agents-dupe.json" <<AJ
[{"id":"aaaa1111","cwd":"/x","kind":"background","sessionId":"$SID","name":"probe one","status":"idle"},
 {"id":"aaaa2222","cwd":"/x","kind":"background","sessionId":"dddddddd-0000-0000-0000-000000000000","name":"probe two","status":"idle"}]
AJ
SESSION_MSG_TEST_AGENTS_JSON="$TMP/agents-dupe.json" \
  bash "$CHECK" prepare --existing probe --sentinel "$SENT" > "$TMP/out" 2>&1
ok 2 "$?" "an ambiguous target is refused"
has "ambiguous" "$TMP/out" "the refusal says the match was ambiguous"

# 20e. no match at all is tooling, not a silent pass
SESSION_MSG_TEST_AGENTS_JSON="$TMP/agents-mixed.json" \
  bash "$CHECK" prepare --existing zzzznope --sentinel "$SENT" >/dev/null 2>&1
ok 2 "$?" "an unmatched target exits 2"

# 20f. mode is mandatory and the two modes are exclusive — prepare must never
#      guess whether it is allowed to spawn a process.
# Assert on the REFUSAL MESSAGE, not just the code: without a mode the fall-through
# would spawn a real session and could still exit 2 for an unrelated reason, so an
# exit-code-only check would pass while the guard was gone.
bash "$CHECK" prepare --sentinel "$SENT" > "$TMP/out" 2>&1
ok 2 "$?" "prepare without a mode exits 2"
has "needs --existing" "$TMP/out" "prepare names the missing mode rather than guessing"
bash "$CHECK" prepare --existing x --spawn --sentinel "$SENT" >/dev/null 2>&1
ok 2 "$?" "--existing and --spawn together exit 2"

# 20g. LOAD-BEARING: dropping the status requirement makes the unreachable target
#      resolve. If that happens, 20b passes silently and the gate is gone.
SESSION_MSG_TEST_AGENTS_JSON="$TMP/agents-mixed.json" \
  bash "$CHECK" prepare --existing bbbb2222 --sentinel "$SENT" > "$TMP/out" 2>&1
hasnt "PREPARE=PASS" "$TMP/out" "an unreachable target never reports PREPARE=PASS"

# ── 21. --help must show the whole header, not a truncated slice. ──────────────
# The help range is a hardcoded line count, so adding header lines silently cuts
# the usage block and the exit-code contract off the end — which happened once
# during this task's own implementation.
bash "$CHECK" --help > "$TMP/help.out" 2>&1; ok 0 "$?" "--help exits 0"
has "Usage:" "$TMP/help.out" "help shows the usage block"
has "prepare --existing" "$TMP/help.out" "help documents the --existing mode"
has "Exit: 0 proven" "$TMP/help.out" "help shows the exit-code contract (the last header line)"

# 21b. Bare invocation shows the same help rather than doing anything.
bash "$CHECK" > "$TMP/bare.out" 2>&1; ok 0 "$?" "bare invocation exits 0"
has "Exit: 0 proven" "$TMP/bare.out" "bare invocation shows the full header too"

# ══ T-3711: the TermLink rail (E-74) ═══════════════════════════════════════════

# 22. The TermLink rail's delivery evidence: the harness writes a
#     `hook_additional_context` attachment when the receiver's hook hands the mail
#     to the model. Content is a list (measured live) or a string.
mk_transcript tl_delivered <<J
{"type":"attachment","attachment":{"type":"hook_additional_context","content":["# Pending peer consult(s): 1\n  body: $SENT"],"toolUseID":"x"},"sessionId":"$SID"}
J
run tl_delivered; ok 0 "$RC" "E-74: a hook_additional_context attachment is DELIVERED"
has "DELIVER=PASS" "$TMP/out" "E-74: the attachment delivery names DELIVER as passing"
mk_transcript tl_delivered_str <<J
{"type":"attachment","attachment":{"type":"hook_additional_context","content":"frame: $SENT"},"sessionId":"$SID"}
J
run tl_delivered_str; ok 0 "$RC" "E-74: a string-content attachment is DELIVERED"

# 23. Records that carry the sentinel without the model having been given it are not
#     delivery: the hook_success echo of a hook's stdout (exists even when the harness
#     dropped the output) and the api-request-blob.
mk_transcript tl_hook_echo <<J
{"type":"attachment","attachment":{"type":"hook_success","content":"stdout: $SENT"},"sessionId":"$SID"}
J
run tl_hook_echo; ok 1 "$RC" "E-74: a hook_success echo is not delivery"
has "UNDELIVERED" "$TMP/out" "E-74: a hook_success echo is named UNDELIVERED"
mk_transcript tl_blob <<J
{"type":"api-request-blob","sessionId":"$SID","message":{"role":"system","content":[{"type":"text","text":"ctx $SENT"}]}}
J
run tl_blob; ok 1 "$RC" "E-74: an api-request-blob mention is not delivery"

# 24. Queued at the receiver: the sentinel waits on the receiver's own TermLink inbox
#     and is not in its transcript. That is ENQUEUED (delivery fine, target has not
#     taken it), never UNDELIVERED.
cat > "$TMP/inbox-queued.json" <<J
{"consults": [{"offset": 0, "topic": "inbox:h/p/$TO", "from": "x", "notice": true, "body": "$SENT"}], "dm_rails": []}
J
SESSION_MSG_TEST_INBOX_JSON="$TMP/inbox-queued.json" run undelivered
ok 1 "$RC" "E-74: a sentinel waiting on the receiver's inbox exits 1"
has "ENQUEUED" "$TMP/out" "E-74: a sentinel on the receiver's inbox is ENQUEUED"
hasnt "UNDELIVERED" "$TMP/out" "E-74: an inbox-queued sentinel is never UNDELIVERED"
cp "$TMP/out" "$TMP/tl-enq.out"
run undelivered; cp "$TMP/out" "$TMP/tl-und.out"
if cmp -s "$TMP/tl-enq.out" "$TMP/tl-und.out"; then FAIL=$((FAIL+1)); echo "  FAIL: E-74: inbox ENQUEUED collapsed into UNDELIVERED"; else PASS=$((PASS+1)); fi

# 24b. At rest with the sentinel on its inbox: BLOCKED, and busy is a different verdict.
SESSION_MSG_TEST_INBOX_JSON="$TMP/inbox-queued.json" SESSION_MSG_TEST_AGENTS_JSON="$AG_REST" \
  run undelivered --id deadbeef
ok 1 "$RC" "E-74: a resting target with inbox mail exits 1"
has "TARGET=FAIL" "$TMP/out" "E-74: a resting target with inbox mail is BLOCKED (TARGET)"
SESSION_MSG_TEST_INBOX_JSON="$TMP/inbox-queued.json" SESSION_MSG_TEST_AGENTS_JSON="$AG_BUSY" \
  run undelivered --id deadbeef
has "DRAIN=FAIL" "$TMP/out" "E-74: a busy target with inbox mail is ENQUEUED (DRAIN)"

# 25. Fail-closed on the receiver's inbox: without it ENQUEUED and UNDELIVERED cannot
#     be told apart, so it is TOOLING, never a verdict.
SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/undelivered" \
  bash "$CHECK" assert --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 > "$TMP/out" 2>&1
ok 2 "$?" "E-74: the termlink rail without --to exits 2 on an absent sentinel"
has "needs --to" "$TMP/out" "E-74: the refusal names the missing receiver inbox"
printf 'not json\n' > "$TMP/inbox-garbage.json"
SESSION_MSG_TEST_INBOX_JSON="$TMP/inbox-garbage.json" run undelivered
ok 2 "$RC" "E-74: an unparseable receiver inbox exits 2, not UNDELIVERED"
printf '[]\n' > "$TMP/inbox-shape.json"
SESSION_MSG_TEST_INBOX_JSON="$TMP/inbox-shape.json" run undelivered
ok 2 "$RC" "E-74: an unexpected receiver inbox shape exits 2"
SESSION_MSG_TEST_INBOX_JSON="$TMP/no-inbox.json" run undelivered
ok 2 "$RC" "E-74: a missing inbox seam exits 2 rather than reading the live hub"
# A DELIVERED verdict never needs the inbox (the transcript already decided).
SESSION_MSG_TEST_INBOX_JSON="$TMP/inbox-garbage.json" run tl_delivered
ok 0 "$RC" "E-74: a delivered sentinel does not consult the inbox"
bash "$CHECK" assert --rail bogus --session-id "$SID" --sentinel "$SENT" >/dev/null 2>&1
ok 2 "$?" "an unknown --rail exits 2"

# 26. The SendMessage rail is kept for rollback (T-2876 behaviour): no inbox needed.
SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/undelivered" \
  bash "$CHECK" assert --rail sendmessage --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 > "$TMP/out" 2>&1
ok 1 "$?" "sendmessage rail: an absent sentinel is UNDELIVERED rc 1"
has "UNDELIVERED" "$TMP/out" "sendmessage rail: named UNDELIVERED"
SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/enqueued" \
  bash "$CHECK" assert --rail sendmessage --session-id "$SID" --sentinel "$SENT" --timeout-secs 0 > "$TMP/out" 2>&1
ok 1 "$?" "sendmessage rail: a queue-operation exits 1"
has "ENQUEUED" "$TMP/out" "sendmessage rail: a queue-operation is ENQUEUED"

# ── 27. send: the sender's answer decides only the SEND stage. ─────────────────
SESSION_MSG_TEST_SEND_CMD='echo "{\"state\": \"HUB_ACCEPTED\", \"delivered\": true}"' \
  bash "$CHECK" send --to "$TO" --sentinel "$SENT" > "$TMP/out" 2>&1
ok 0 "$?" "send: an accepted post is SEND=PASS"
has "not trusted for delivery" "$TMP/out" "send: says the sender is not trusted for delivery"
SESSION_MSG_TEST_SEND_CMD='echo "{\"state\": \"REFUSED\", \"delivered\": false}"' \
  bash "$CHECK" send --to "$TO" --sentinel "$SENT" > "$TMP/out" 2>&1
ok 1 "$?" "send: a REFUSED post exits 1"
has "SEND=FAIL" "$TMP/out" "send: a refusal names the SEND stage"
SESSION_MSG_TEST_SEND_CMD='echo boom; exit 3' \
  bash "$CHECK" send --to "$TO" --sentinel "$SENT" > "$TMP/out" 2>&1
ok 1 "$?" "send: a failing sender exits 1"
bash "$CHECK" send --sentinel "$SENT" >/dev/null 2>&1; ok 2 "$?" "send without --to exits 2"
bash "$CHECK" wake --to "$TO" >/dev/null 2>&1;         ok 2 "$?" "wake without --session-id exits 2"

# ── 28. prove: the live pass end to end, hermetic. ─────────────────────────────
# The send seam stashes what it posted; the wake seam plays the receiver's harness
# and writes the probe's transcript. Three receivers: one that got the mail, one
# whose sender said "accepted" but which never got it (the sender lie), and a
# vacuous one that "receives" every sentinel, the control included.
mkdir -p "$TMP/prove/-opt-termlink"
export SMS_FX_DIR="$TMP"
SEND_STASH='printf "%s" "$SMS_SENT" > "$SMS_FX_DIR/sent"; echo "{\"state\": \"HUB_ACCEPTED\", \"delivered\": true}"'
WAKE_GOOD='s=$(cat "$SMS_FX_DIR/sent"); printf "{\"type\":\"attachment\",\"attachment\":{\"type\":\"hook_additional_context\",\"content\":[\"consult: %s\"]},\"sessionId\":\"%s\"}\n" "$s" "$SMS_SID" > "$SMS_FX_DIR/prove/-opt-termlink/$SMS_SID.jsonl"'
WAKE_LOST='printf "{\"type\":\"user\",\"message\":{\"content\":\"READY?\"},\"sessionId\":\"%s\"}\n" "$SMS_SID" > "$SMS_FX_DIR/prove/-opt-termlink/$SMS_SID.jsonl"'
WAKE_VACUOUS='s=$(cat "$SMS_FX_DIR/sent"); c=${s/SMS-PROBE-/SMS-CONTROL-}; printf "{\"type\":\"user\",\"message\":{\"content\":\"%s %s\"},\"sessionId\":\"%s\"}\n" "$s" "$c" "$SMS_SID" > "$SMS_FX_DIR/prove/-opt-termlink/$SMS_SID.jsonl"'
prove(){ # $1 wake cmd, $2 record
  SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/prove" SESSION_MSG_TEST_SEND_CMD="$SEND_STASH" \
  SESSION_MSG_TEST_WAKE_CMD="$1" bash "$CHECK" prove --record "$2" --timeout-secs 0 > "$TMP/out" 2>&1
  RC=$?
}
prove "$WAKE_GOOD" "$TMP/rec-good.jsonl"
ok 0 "$RC" "E-74: prove — delivered probe with an undelivered control exits 0"
has "PROVE=PASS" "$TMP/out" "E-74: prove reports PASS"
python3 - "$TMP/rec-good.jsonl" > "$TMP/rec.chk" 2>&1 <<'PY'
import json,sys
es=[json.loads(l) for l in open(sys.argv[1])]
roles={(e["role"],e["verdict"],e["rc"],e["rail"]) for e in es}
assert ("probe","DELIVERED",0,"termlink") in roles and ("control","UNDELIVERED",1,"termlink") in roles, roles
assert len({e["session_id"] for e in es})==1 and len({e["run_id"] for e in es})==1
assert all(len(e["script_sha256"])==64 for e in es)
print("ok")
PY
has "^ok" "$TMP/rec.chk" "E-74: the record holds the probe DELIVERED rc 0 and the control UNDELIVERED rc 1"
bash "$CHECK" verify-record --record "$TMP/rec-good.jsonl" > "$TMP/out" 2>&1
ok 0 "$?" "E-74: verify-record passes on a proven pair"

prove "$WAKE_LOST" "$TMP/rec-lost.jsonl"
ok 1 "$RC" "E-74: prove — sender said accepted, receiver never got it: exits 1"
has "PROVE=FAIL" "$TMP/out" "E-74: the sender lie is reported as a failed probe"
bash "$CHECK" verify-record --record "$TMP/rec-lost.jsonl" > "$TMP/out" 2>&1
ok 1 "$?" "E-74: verify-record refuses a run whose probe was not delivered"

prove "$WAKE_VACUOUS" "$TMP/rec-vac.jsonl"
ok 2 "$RC" "E-74: prove — a control that reads delivered makes the prover untrusted (2)"
bash "$CHECK" verify-record --record "$TMP/rec-vac.jsonl" > "$TMP/out" 2>&1
ok 1 "$?" "E-74: verify-record refuses a run without an undelivered control"

SESSION_MSG_TEST_TRANSCRIPT_DIR="$TMP/prove" SESSION_MSG_TEST_SEND_CMD='echo "{\"state\": \"REFUSED\"}"' \
  SESSION_MSG_TEST_WAKE_CMD="$WAKE_GOOD" bash "$CHECK" prove --record "$TMP/rec-ref.jsonl" --timeout-secs 0 > "$TMP/out" 2>&1
ok 1 "$?" "prove — a refused send exits 1 at the SEND stage"
test ! -e "$TMP/rec-ref.jsonl"; ok 0 "$?" "prove — a refused send writes no record"

# 29. verify-record is the P-28 gate's reader: evidence must be this script's.
bash "$CHECK" verify-record --record "$TMP/none.jsonl" >/dev/null 2>&1
ok 1 "$?" "verify-record: no record exits 1"
sed 's/"script_sha256": "[0-9a-f]*"/"script_sha256": "0000"/' "$TMP/rec-good.jsonl" > "$TMP/rec-oldsha.jsonl"
bash "$CHECK" verify-record --record "$TMP/rec-oldsha.jsonl" >/dev/null 2>&1
ok 1 "$?" "verify-record: a pass made by another version of the script does not count"
grep '"role": "probe"' "$TMP/rec-good.jsonl" > "$TMP/rec-probeonly.jsonl"
bash "$CHECK" verify-record --record "$TMP/rec-probeonly.jsonl" >/dev/null 2>&1
ok 1 "$?" "verify-record: a probe without its control does not count"
sed 's/"rail": "termlink"/"rail": "sendmessage"/' "$TMP/rec-good.jsonl" > "$TMP/rec-sm.jsonl"
bash "$CHECK" verify-record --record "$TMP/rec-sm.jsonl" >/dev/null 2>&1
ok 1 "$?" "verify-record: a SendMessage-rail pass does not count"
{ cat "$TMP/rec-good.jsonl"; echo 'torn{'; } > "$TMP/rec-torn.jsonl"
bash "$CHECK" verify-record --record "$TMP/rec-torn.jsonl" >/dev/null 2>&1
ok 2 "$?" "verify-record: an unreadable record exits 2"

echo "session-message-selftest fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] && [ -z "${SMS_FIXTURE_TARGET:-}" ] && echo "PASS E-74 the session message prover on the TermLink rail: DELIVERED 0 / BLOCKED 1 / ENQUEUED 1 / UNDELIVERED 1 / TOOLING 2, assert on the receiver"

# ══ Mutants (FP 3.1.b): each breaks one rule; the suite must go red on each. ════
if [ -z "${SMS_FIXTURE_TARGET:-}" ] && [ "$FAIL" -eq 0 ]; then
  SRC="$ROOT/scripts/session-message-selftest.sh"
  KILLED=0; TOTAL=0
  mutant(){ # $1 name, $2 python replace old, $3 new
    TOTAL=$((TOTAL+1))
    local m="$TMP/mutant-$1.sh"
    MUT_OLD="$2" MUT_NEW="$3" python3 -c '
import os,sys
s=open(sys.argv[1]).read(); o=os.environ["MUT_OLD"]
if o not in s: sys.exit(1)
open(sys.argv[2],"w").write(s.replace(o,os.environ["MUT_NEW"],1))' "$SRC" "$m" \
      || { FAIL=$((FAIL+1)); echo "  FAIL: mutant $1 did not apply"; return; }
    if SMS_FIXTURE_TARGET="$m" bash "${BASH_SOURCE[0]}" > "$TMP/mutant-$1.out" 2>&1; then
      FAIL=$((FAIL+1)); echo "  FAIL: mutant $1 SURVIVED"
    else KILLED=$((KILLED+1)); echo "  killed: $1"; fi
  }
  mutant remove-blocked-branch '*) verdict="BLOCKED" ;;' '*) : ;;'
  mutant drop-reachability-gate 'hits=[a for a in d if a.get("status")' 'hits=[a for a in d if True'
  mutant collapse-enqueued-into-undelivered 'ENQUEUED)  rc=$RC_BROKEN; stage="DRAIN" ;;' 'ENQUEUED)  rc=$RC_BROKEN; stage="DELIVER"; verdict="UNDELIVERED" ;;'
  mutant any-attachment-is-delivery 'if a.get("type")=="hook_additional_context":' 'if True:'
  mutant skip-receiver-inbox '[ "$q" = "QUEUED" ] && verdict="ENQUEUED"' ':'
  mutant unreadable-inbox-is-empty "|| die_tooling \"the receiver's inbox answer is not the expected JSON (\$TO)\"" '|| q=NONE'
  mutant verify-ignores-script-version 'or d.get("script_sha256")!=os.environ["SMS_SHA"] ' ''
  mutant verify-without-control 'if p and c and any(x["session_id"]==y["session_id"] and x["sentinel"]!=y["sentinel"] for x in p for y in c):' 'if p:'
  mutant prove-ignores-control '[ $prc -eq 0 ] && [ $crc -eq 1 ] && control_undelivered "$RUN_ID"' '[ $prc -eq 0 ]'
  echo "MUTANTS: $KILLED/$TOTAL killed"
  [ "$KILLED" -eq "$TOTAL" ] || exit 1
fi
[ "$FAIL" -eq 0 ]
