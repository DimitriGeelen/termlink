#!/usr/bin/env bash
# T-2592 — load-bearing fixture for scripts/agent-chat-arc-recent.sh msg_type
# default. Drives the whole engine through a mock `termlink` (TERMLINK_BIN) on a
# single --hub (bypasses the hubs.toml walk) and asserts the three-state filter:
#
#   default (no flags)   → content set {post,chat,note}, meta excluded
#   --filter-msg-type X   → single-type narrowing
#   --all-msg-types       → everything, incl meta (reaction)
#
# The default-view test FAILS against the old `FILTER_MSG_TYPE="chat"` +
# `.msg_type == $mtype` default (temp-revert proven) and PASSES against the fix —
# making "a note post appears in the default 'what's been said?' view"
# load-bearing. No live hub, no network.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
# CHAT_ARC_SCRIPT lets the suite be pointed at another copy — used to prove the
# T-2731 fixtures are load-bearing by running them against the pre-fix script.
SCRIPT="${CHAT_ARC_SCRIPT:-$HERE/../scripts/agent-chat-arc-recent.sh}"
[ -f "$SCRIPT" ] || { echo "FAIL: $SCRIPT not found"; exit 1; }

WORK="$(mktemp -d -t chat-arc-recent-fixtures.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

# --- mock termlink ------------------------------------------------------------
# agent-chat-arc window has THREE content posters across three content types
# plus one meta envelope:
#   note by agent-alpha      (termlink_agent_post / agent_reply path — the type
#                             the OLD chat-only default silently DROPPED)
#   chat by agent-beta       (/broadcast-chat path)
#   post by agent-gamma      (legacy content type — still in the content set)
#   reaction by agent-delta  (META — must NOT appear except under --all-msg-types)
# ts is stamped at run time so every envelope is inside the 24h window.
MOCK="$WORK/termlink"
cat > "$MOCK" <<'MOCKEOF'
#!/usr/bin/env bash
args="$*"
now_ms="$(date +%s)000"
case "$args" in
  *"channel info"*"agent-chat-arc"*)
    printf '%s\n' '{"count":4}'
    ;;
  *"channel subscribe"*"agent-chat-arc"*)
    printf '{"offset":0,"ts":%s,"msg_type":"note","sender_id":"fp-a","metadata":{"agent_id":"agent-alpha"},"payload":"hello-note"}\n' "$now_ms"
    printf '{"offset":1,"ts":%s,"msg_type":"chat","sender_id":"fp-b","metadata":{"agent_id":"agent-beta"},"payload":"hello-chat"}\n' "$now_ms"
    printf '{"offset":2,"ts":%s,"msg_type":"post","sender_id":"fp-c","metadata":{"agent_id":"agent-gamma"},"payload":"hello-post"}\n' "$now_ms"
    printf '{"offset":3,"ts":%s,"msg_type":"reaction","sender_id":"fp-d","metadata":{"agent_id":"agent-delta"},"payload":"hello-reaction"}\n' "$now_ms"
    ;;
  *)
    : ;;
esac
MOCKEOF
chmod +x "$MOCK"

HUB="127.0.0.1:9999"
run() { TERMLINK_BIN="$MOCK" bash "$SCRIPT" --hub "$HUB" --json "$@" 2>/dev/null; }
types() { jq -r '.posts // [] | map(.msg_type) | sort | join(",")' 2>/dev/null; }

fail=0
pass() { echo "PASS: $1"; }
bad()  { echo "FAIL: $1"; fail=1; }

# ---- Test 1: default view = content set {post,chat,note}, meta excluded -------
out="$(run)"
got="$(printf '%s' "$out" | types)"
if [ "$got" = "chat,note,post" ]; then
  pass "default → content set {chat,note,post}, reaction excluded (note is VISIBLE)"
else
  bad "default → expected 'chat,note,post', got '$got' (old chat-only default gives just 'chat')"
  printf '  raw: %s\n' "$out"
fi

# ---- Test 2: the load-bearing assertion — a note post appears by default ------
if printf '%s' "$out" | jq -e '.posts // [] | any(.msg_type == "note")' >/dev/null 2>&1; then
  pass "default → a note post is present (the T-2592 silent-drop is closed)"
else
  bad "default → NO note post present (regression: note content hidden by default)"
fi

# ---- Test 3: --filter-msg-type note narrows to a single type ------------------
got="$(run --filter-msg-type note | types)"
if [ "$got" = "note" ]; then
  pass "--filter-msg-type note → only note (single-type narrowing preserved)"
else
  bad "--filter-msg-type note → expected 'note', got '$got'"
fi

# ---- Test 4: --all-msg-types includes meta (reaction) ------------------------
got="$(run --all-msg-types | types)"
if [ "$got" = "chat,note,post,reaction" ]; then
  pass "--all-msg-types → everything incl reaction (meta) still works"
else
  bad "--all-msg-types → expected 'chat,note,post,reaction', got '$got'"
fi

# ---- T-2731: read completeness ----------------------------------------------
# `ok` says the command ran; `read_complete` says the ANSWER is trustworthy.
# Live evidence for why this matters: a real fleet read returned
# {ok:true, total_posts:0} while one hub was unreachable and two served a
# partial head-read — a zero that cannot distinguish "quiet" from "did not
# look". Absence of evidence from a degraded read is not evidence of absence.

# ---- Test 5: a complete read declares itself complete ------------------------
if printf '%s' "$out" | jq -e '.summary.read_complete == true and (.summary.degraded_reasons | length) == 0' >/dev/null 2>&1; then
  pass "complete read → read_complete:true, no degraded_reasons"
else
  bad "complete read → expected read_complete:true with empty degraded_reasons"
  printf '  summary: %s\n' "$(printf '%s' "$out" | jq -c '.summary' 2>/dev/null)"
fi

# ---- Test 6: fallback hub (channel info fails, subscribe works) --------------
# This is the T-1872 partial head-read: the hub answers, but only from the head,
# so recent posts can be missing entirely. The old envelope reported this as an
# ordinary success.
MOCK_FB="$WORK/termlink-fallback"
sed 's/\*"channel info"\*"agent-chat-arc"\*)/*"channel info"*"agent-chat-arc"*)\n    exit 1;;\n  *"__never_matches__"*)/' "$MOCK" > "$MOCK_FB"
chmod +x "$MOCK_FB"
out_fb="$(TERMLINK_BIN="$MOCK_FB" bash "$SCRIPT" --hub "$HUB" --json 2>/dev/null)"
if printf '%s' "$out_fb" | jq -e '.summary.read_complete == false and (.summary.degraded_reasons | join(" ") | test("partial head-read"))' >/dev/null 2>&1; then
  pass "fallback hub → read_complete:false, reason names the partial head-read"
else
  bad "fallback hub → expected read_complete:false naming 'partial head-read'"
  printf '  summary: %s\n' "$(printf '%s' "$out_fb" | jq -c '.summary' 2>/dev/null)"
fi

# ---- Test 7: failed hub (both info and subscribe fail) -----------------------
MOCK_FAIL="$WORK/termlink-fail"
cat > "$MOCK_FAIL" <<'FAILEOF'
#!/usr/bin/env bash
exit 1
FAILEOF
chmod +x "$MOCK_FAIL"
out_fail="$(TERMLINK_BIN="$MOCK_FAIL" bash "$SCRIPT" --hub "$HUB" --json 2>/dev/null)"
if printf '%s' "$out_fail" | jq -e '.summary.read_complete == false and (.summary.degraded_reasons | join(" ") | test("unreachable"))' >/dev/null 2>&1; then
  pass "failed hub → read_complete:false, reason names it unreachable"
else
  bad "failed hub → expected read_complete:false naming 'unreachable'"
  printf '  summary: %s\n' "$(printf '%s' "$out_fail" | jq -c '.summary' 2>/dev/null)"
fi

# ---- Test 8: human mode refuses to assert absence on a degraded read ---------
# The failure this prevents is a reader skimming past the `failed:` line and
# taking "(no posts matched filters)" as the verdict.
human="$(TERMLINK_BIN="$MOCK_FAIL" bash "$SCRIPT" --hub "$HUB" 2>/dev/null)"
if printf '%s' "$human" | grep -q "absence is NOT established" && \
   ! printf '%s' "$human" | grep -q "no posts matched filters"; then
  pass "degraded + zero results → human mode states absence is NOT established"
else
  bad "degraded + zero results → expected the DEGRADED wording, not 'no posts matched filters'"
  printf '  output: %s\n' "$human"
fi

# ---- Test 9: healthy path keeps its original wording (no alert fatigue) ------
human_ok="$(TERMLINK_BIN="$MOCK" bash "$SCRIPT" --hub "$HUB" --filter-msg-type nonesuch 2>/dev/null)"
if printf '%s' "$human_ok" | grep -q "no posts matched filters"; then
  pass "complete read + zero results → original wording preserved (PL-219)"
else
  bad "complete read + zero results → expected 'no posts matched filters' unchanged"
  printf '  output: %s\n' "$human_ok"
fi

# ---- T-2758: retention-trimmed topic — count is NOT an offset ----------------
# Reproduces the measured production defect. agent-chat-arc had count=2003 under
# `messages/2000` retention while live offsets ran to 11973, so the legacy
# `cursor = count - SCAN_LIMIT` seek landed at 1503 — ~8500 offsets BELOW the
# retained range. The scan read the OLDEST retained envelopes, the ts window
# discarded every one, and the verb reported "0 posts" on a topic that had a
# post that same morning: a silent under-report, not an error (Directive #2).
#
# The mock records every cursor it is asked for, so the assertions pin BOTH
# halves: that recent posts surface at all, and that the cursor was derived
# from an OFFSET. The cursor assertion is what makes this load-bearing — it
# fails against the pre-fix arithmetic (which asks for 1503) even if some
# future change made recent posts appear for an unrelated reason.
TRIM_WORK="$(mktemp -d -t chat-arc-trim.XXXXXX)"
trap 'rm -rf "$WORK" "$TRIM_WORK"' EXIT
CURSOR_LOG="$TRIM_WORK/cursors"
: > "$CURSOR_LOG"

TRIM_MOCK="$TRIM_WORK/termlink"
cat > "$TRIM_MOCK" <<'TRIMEOF'
#!/usr/bin/env bash
args="$*"
now_ms="$(date +%s)000"
old_ms=$(( $(date +%s) - 30*24*3600 ))000
case "$args" in
  *"channel info"*"agent-chat-arc"*)
    # count is capped by retention; the true tail is far beyond it. The only
    # offset-shaped signal in this payload is the receipt frontier.
    printf '%s\n' '{"count":2003,"retention":{"kind":"messages","value":2000},"receipts":[{"sender_id":"fp-x","up_to":11952}]}'
    ;;
  *"channel subscribe"*"agent-chat-arc"*)
    cursor=0; limit=100
    prev=""
    for a in $args; do
      case "$prev" in --cursor) cursor="$a" ;; --limit) limit="$a" ;; esac
      prev="$a"
    done
    echo "$cursor" >> "$TL_CURSOR_LOG"
    # Retained range is 9974..11973. Everything below 11953 is 30 days old;
    # 11953..11973 are the recent envelopes the operator is looking for.
    emitted=0
    off="$cursor"
    [ "$off" -lt 9974 ] && off=9974
    while [ "$off" -le 11973 ] && [ "$emitted" -lt "$limit" ]; do
      if [ "$off" -ge 11953 ]; then ts="$now_ms"; tag="recent"; else ts="$old_ms"; tag="ancient"; fi
      printf '{"offset":%s,"ts":%s,"msg_type":"chat","sender_id":"fp-x","metadata":{"agent_id":"agent-trim"},"payload":"%s-%s"}\n' \
        "$off" "$ts" "$tag" "$off"
      off=$((off + 1)); emitted=$((emitted + 1))
    done
    ;;
  *)
    : ;;
esac
TRIMEOF
chmod +x "$TRIM_MOCK"

trim_out="$(TERMLINK_BIN="$TRIM_MOCK" TL_CURSOR_LOG="$CURSOR_LOG" \
  bash "$SCRIPT" --hub "$HUB" --json --since 24 2>/dev/null)"

recent_n="$(printf '%s' "$trim_out" | jq -r '.posts // [] | map(select(.payload_preview | startswith("recent"))) | length' 2>/dev/null || echo 0)"
if [ "${recent_n:-0}" -gt 0 ]; then
  pass "trimmed topic → recent posts surface ($recent_n) where the count-derived seek saw none"
else
  bad "trimmed topic → NO recent posts (the T-2758 silent under-report is back)"
  printf '  summary: %s\n' "$(printf '%s' "$trim_out" | jq -c '.summary' 2>/dev/null)"
fi

first_cursor="$(head -n 1 "$CURSOR_LOG" 2>/dev/null || echo "")"
if [ -n "$first_cursor" ] && [ "$first_cursor" -ge 11000 ] 2>/dev/null; then
  pass "trimmed topic → first cursor $first_cursor is offset-derived (pre-fix arithmetic asks for 1503)"
else
  bad "trimmed topic → first cursor '$first_cursor' is not offset-derived (expected >= 11000)"
fi

# The drain must actually reach the tail: a single 500-envelope read starting at
# 11452 stops at 11951 and still misses every recent envelope. Two cursors means
# the loop continued past a full batch.
cursor_rounds="$(wc -l < "$CURSOR_LOG" | tr -d ' ')"
if [ "${cursor_rounds:-0}" -ge 2 ]; then
  pass "trimmed topic → drain continued past a full batch ($cursor_rounds reads) to reach the live tail"
else
  bad "trimmed topic → only $cursor_rounds read(s); a lower-bound tail needs the forward drain"
fi

# ---- T-2758: the other three tail-signal shapes -------------------------------
# The trimmed case above exercises the receipt-derived path. derive_tail_offset
# has three more branches that must be pinned, because each one silently
# produces a plausible cursor rather than an error when it goes wrong:
#   authoritative  — a T-2533+ hub serves latest_offset; it must WIN over both
#                    count and receipts (here count/receipts would say 1503/…)
#   untrimmed      — no latest_offset, no receipt beyond count-1: the legacy
#                    count path, which is correct while the topic is untrimmed
#   neither        — no offset signal at all on a topic that LOOKS trimmed:
#                    must not fabricate a tail, and must refuse to certify the
#                    read complete (this is the Directive #2 half of the fix)
tail_case() {
  # $1 = case name, $2 = channel info JSON
  local name="$1" info="$2" log="$TRIM_WORK/cursors-$1"
  : > "$log"
  cat > "$TRIM_WORK/termlink-$name" <<CASEEOF
#!/usr/bin/env bash
args="\$*"
now_ms="\$(date +%s)000"
case "\$args" in
  *"channel info"*"agent-chat-arc"*) printf '%s\n' '$info' ;;
  *"channel subscribe"*"agent-chat-arc"*)
    cursor=0; prev=""
    for a in \$args; do
      case "\$prev" in --cursor) cursor="\$a" ;; esac
      prev="\$a"
    done
    echo "\$cursor" >> "$log"
    printf '{"offset":%s,"ts":%s,"msg_type":"chat","sender_id":"fp-x","metadata":{"agent_id":"agent-t"},"payload":"x"}\n' "\$cursor" "\$now_ms"
    ;;
  *) : ;;
esac
CASEEOF
  chmod +x "$TRIM_WORK/termlink-$name"
  TERMLINK_BIN="$TRIM_WORK/termlink-$name" bash "$SCRIPT" --hub "$HUB" --json --since 24 2>/dev/null
  TAIL_CASE_CURSOR="$(head -n 1 "$log" 2>/dev/null || echo "")"
}

# authoritative latest_offset must win over count AND receipts
tail_case authoritative '{"count":2003,"retention":{"kind":"messages","value":2000},"receipts":[{"sender_id":"fp-x","up_to":11952}],"latest_offset":13000}' >/dev/null
if [ "$TAIL_CASE_CURSOR" = "12500" ]; then
  pass "tail signal: latest_offset wins over count and receipts (cursor 12500)"
else
  bad "tail signal: expected cursor 12500 from latest_offset, got '$TAIL_CASE_CURSOR'"
fi

# untrimmed topic: no latest_offset, receipts do not exceed count-1 → count path
tail_case untrimmed '{"count":900,"retention":{"kind":"messages","value":2000},"receipts":[{"sender_id":"fp-x","up_to":100}]}' >/dev/null
if [ "$TAIL_CASE_CURSOR" = "399" ]; then
  pass "tail signal: untrimmed topic keeps the count path (cursor 399 = 900-1-500)"
else
  bad "tail signal: expected cursor 399 on an untrimmed topic, got '$TAIL_CASE_CURSOR'"
fi

# no offset signal at all on a topic that looks trimmed → must not certify
neither_out="$(tail_case neither '{"count":2000,"retention":{"kind":"messages","value":2000}}')"
if printf '%s' "$neither_out" | jq -e '.summary.read_complete == false and (.summary.tail_unknown_hubs | length) > 0' >/dev/null 2>&1; then
  pass "tail signal: no offset signal on a trimmed topic → read_complete:false, hub named"
else
  bad "tail signal: no offset signal must clear read_complete and name the hub"
  printf '  summary: %s\n' "$(printf '%s' "$neither_out" | jq -c '.summary' 2>/dev/null)"
fi

echo
if [ "$fail" -eq 0 ]; then
  echo "chat-arc-recent-fixtures: ALL PASS"; exit 0
else
  echo "chat-arc-recent-fixtures: FAILURES"; exit 1
fi
