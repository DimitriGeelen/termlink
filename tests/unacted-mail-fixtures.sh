#!/usr/bin/env bash
# guard-layer: source
#
# T-3618 — fixtures for scripts/check-unacted-mail.sh: mail delivered but never acted
# on, every rail. Runs the REAL script with a stub termlink and a stub fw; no hub.
set -uo pipefail
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/check-unacted-mail.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/topics" "$TMP/facts"

now_ms=$(( $(date +%s) * 1000 ))
h() { echo $(( now_ms - $1 * 3600000 )); }   # <hours ago> -> ts in ms
b64() { printf '%s' "$1" | base64 -w0; }
post() {  # <topic file> <offset> <hours ago> <from_project> <body> [extra metadata json]
    printf '{"offset":%s,"msg_type":"chat","sender_id":"d1993c2c3ec44c94","ts":%s,"metadata":{"from_project":"%s"%s},"payload_b64":"%s"}\n' \
        "$2" "$(h "$3")" "$4" "${6:+,$6}" "$(b64 "$5")" >> "$TMP/topics/$1"
}
receipt() { printf '{"offset":%s,"msg_type":"chat","sender_id":"3bba15e681b3a078","ts":%s,"metadata":{"stage":"delivered","up_to":"%s"},"payload_b64":""}\n' \
        "$2" "$(h "$3")" "$4" >> "$TMP/topics/$1"; }

reset() {
    rm -f "$TMP/topics/"* "$TMP/facts/"*
    for v in inbox-stale held-unsurfaced orphan-inboxes; do echo '[]' > "$TMP/facts/$v"; done
    : > "$TMP/ignore"; : > "$TMP/handled"
}

cat > "$TMP/termlink" <<STUB
#!/usr/bin/env bash
T="$TMP/topics"
STUB
cat >> "$TMP/termlink" <<'STUB'
case "$1 $2" in
  "channel list")
      [ -n "${FAKE_LIST_FAIL:-}" ] && { echo "hub down" >&2; exit 1; }
      printf '{"topics":['; first=1
      for f in "$T"/*; do [ -e "$f" ] || continue; [ $first = 1 ] || printf ','; first=0
          printf '{"name":"%s"}' "$(basename "$f" | tr '+' ':')"; done
      printf ']}\n'; exit 0 ;;
  "channel subscribe")
      [ -n "${FAKE_READ_FAIL:-}" ] && exit 1
      cat "$T/$(printf '%s' "$3" | tr ':' '+')"; exit 0 ;;
esac
exit 0
STUB
cat > "$TMP/fw" <<STUB
#!/usr/bin/env bash
F="$TMP/facts"
STUB
cat >> "$TMP/fw" <<'STUB'
[ "$1" = sidecar ] || exit 2
v="$2"
[ "${FAKE_FW_FAIL:-}" = "$v" ] && { echo "boom" >&2; exit 3; }
cat "$F/$v"
[ "$v" = orphan-inboxes ] && [ "$(cat "$F/$v")" != "[]" ] && exit 1
exit 0
STUB
chmod +x "$TMP/termlink" "$TMP/fw"
run() { UNACTED_TL="$TMP/termlink" UNACTED_FW="$TMP/fw" UNACTED_IGNORE="$TMP/ignore" \
        UNACTED_HANDLED="$TMP/handled" bash "$SCRIPT" "$@"; }

# 1. the 055 shape: last real message 197 h old, only receipts after it
reset
post "dm+3bba+d199" 136 197 055-cockpit "Finding: claude-fw guard bug"
receipt "dm+3bba+d199" 137 197 136; receipt "dm+3bba+d199" 138 196 136
out="$(run)"; rc=$?
[ "$rc" = 1 ] && printf '%s' "$out" | grep -q '^UNACTED  dm  dm:3bba:d199  offset 136  from=055-cockpit  age=197h  Finding: claude-fw guard bug' \
    && ok "dm: an unanswered message (receipts after it do not count as an answer) is listed, exit 1" || bad "dm unanswered: rc=$rc $out"

# 2. answered: a later real message
post "dm+3bba+d199" 139 190 999-AEF "thanks, fixed in T-4900"
out="$(run)"; rc=$?
[ "$rc" = 0 ] && [ -z "$out" ] && ok "dm: answered topic is not listed, exit 0" || bad "dm answered: rc=$rc $out"
reset; post "dm+e+f" 1 60 050-pen "ask"; post "dm+e+f" 2 30 050-pen "ask again, still nothing"
out="$(run)"
printf '%s' "$out" | grep -q 'dm:e:f  offset 2  from=050-pen' && ok "dm: a sender's own follow-up with no answer is still listed" || bad "own follow-up: $out"
reset; post "dm+e+f" 1 400 010-tl "old exchange"; post "dm+e+f" 2 30 050-pen "new ask, two weeks later"
out="$(run)"
printf '%s' "$out" | grep -q 'dm:e:f  offset 2  from=050-pen' \
    && ok "dm: a message long after the other side's last one is a new ask, listed (the Pen case)" || bad "Pen case: $out"
reset; post "dm+e+f" 1 60 050-pen "ask"; post "dm+e+f" 2 50 010-tl "answer"; post "dm+e+f" 3 30 050-pen "follow-up"
out="$(run)"
[ -z "$out" ] && ok "dm: documented blind spot — a follow-up within 72 h of an answer reads as part of the exchange, not listed" || bad "blind spot changed: $out"

# shared mailbox: to_project decides who must answer, per message
reset
post "dm+r+r" 16 60 0235-voice "for AEF" '"to_project":"999-AEF"'
post "dm+r+r" 19 50 050-pen "for 010" '"to_project":"010-termlink"'
post "dm+r+r" 20 40 1023-vault "for AEF too" '"to_project":"999-AEF"'
post "dm+r+r" 21 30 010-termlink "re 050: done"
out="$(run)"
printf '%s' "$out" | grep -q 'offset 16  from=0235-voice  to=999-AEF' && printf '%s' "$out" | grep -q 'offset 20  from=1023-vault  to=999-AEF' \
    && ! printf '%s' "$out" | grep -q 'offset 19' \
    && ok "dm: shared mailbox — each message waits for its own to_project; one answered by its addressee is not listed" || bad "shared mailbox: $out"

# 3. threshold and window
reset; post "dm+a+b" 1 2 x "too new"; post "dm+c+d" 1 400 x "older than the window"
out="$(run)"; rc=$?
[ "$rc" = 0 ] && [ -z "$out" ] && ok "dm: younger than 24 h and older than 14 days are not listed" || bad "dm window: rc=$rc $out"
out="$(run --threshold-hours 1)"
printf '%s' "$out" | grep -q 'dm:a:b  offset 1' && ok "dm: --threshold-hours moves the lower bound" || bad "threshold flag: $out"

# 4. a reply or a notice as the last word is not a question left open
reset; post "dm+a+b" 3 30 x "here is the answer" '"in_reply_to":"m1"'; post "dm+c+d" 4 30 y "fyi" '"notice":true'
out="$(run)"; [ -z "$out" ] && ok "dm: a reply or a notice as the last word is not listed" || bad "dm reply/notice: $out"

# 5. ignore file and handled file
reset; post "dm+a+b" 5 30 x "smoke hi"; post "dm+c+d" 6 30 y "real ask"
printf '# test rails\ndm:a:b  fake peer\n' > "$TMP/ignore"
printf 'dm:c:d 6 2026-10-28 forwarded\n' > "$TMP/handled"
out="$(run)"; rc=$?
[ "$rc" = 0 ] && [ -z "$out" ] && ok "dm: an ignored topic and a handled offset are skipped" || bad "ignore/handled: rc=$rc $out"
post "dm+c+d" 7 25 y "second ask"
out="$(run)"
printf '%s' "$out" | grep -q 'dm:c:d  offset 7' && ok "dm: a later message re-flags a handled topic" || bad "handled re-flag: $out"

# 6. other rails, each listed by name (guards against a rail silently dropped)
reset
echo '[{"topic":"inbox:h/010","unread":2,"age_hours":30.5,"oldest_from":"055"}]' > "$TMP/facts/inbox-stale"
echo '[{"msg_id":"m9","age_hours":48,"from":"aef"}]' > "$TMP/facts/held-unsurfaced"
echo '[{"topic":"inbox:h/ring20-manager","unread":14}]' > "$TMP/facts/orphan-inboxes"
post "dm+a+b" 1 30 x "ask"
out="$(run)"; rc=$?
n="$(printf '%s\n' "$out" | grep -c '^UNACTED')"
[ "$rc" = 1 ] && [ "$n" = 4 ] \
    && printf '%s' "$out" | grep -q '^UNACTED  inbox  inbox:h/010  unread=2  age_hours=30.5  oldest_from=055' \
    && printf '%s' "$out" | grep -q '^UNACTED  held  m9  age_hours=48  from=aef' \
    && printf '%s' "$out" | grep -q '^UNACTED  orphan  inbox:h/ring20-manager  unread=14' \
    && ok "all four rails report (dm, inbox, held, orphan); orphan-inboxes' own exit 1 is a finding, not a failure" \
    || bad "four rails: rc=$rc n=$n $out"

# 7. fail-closed: could not look is exit 2, never clean
reset
out="$(FAKE_LIST_FAIL=1 run)"; rc=$?
[ "$rc" = 2 ] && printf '%s' "$out" | grep -q '^NOT CHECKED dm: termlink channel list --prefix dm: failed' \
    && ok "dm: the hub cannot be listed => NOT CHECKED, exit 2" || bad "dm list fail: rc=$rc $out"
post "dm+a+b" 1 30 x "ask"
out="$(FAKE_READ_FAIL=1 run)"; rc=$?
[ "$rc" = 2 ] && printf '%s' "$out" | grep -q '^NOT CHECKED dm: dm:a:b could not be read' \
    && ok "dm: a topic that cannot be read => NOT CHECKED, exit 2" || bad "dm read fail: rc=$rc $out"
out="$(FAKE_FW_FAIL=held-unsurfaced run)"; rc=$?
[ "$rc" = 1 ] && printf '%s' "$out" | grep -q '^NOT CHECKED held: fw sidecar held-unsurfaced failed' \
    && printf '%s' "$out" | grep -q '^UNACTED  dm  dm:a:b' \
    && ok "a finding wins over a read error (exit 1), the NOT CHECKED line is still printed" || bad "finding+blind: rc=$rc $out"
echo 'not json' > "$TMP/facts/inbox-stale"; rm -f "$TMP/topics/"*
out="$(run)"; rc=$?
[ "$rc" = 2 ] && printf '%s' "$out" | grep -q '^NOT CHECKED inbox:' \
    && ok "a fact verb answering non-JSON => NOT CHECKED, exit 2" || bad "non-json: rc=$rc $out"
out="$(run --bogus 2>&1)"; rc=$?
[ "$rc" = 2 ] && ok "unknown argument => exit 2" || bad "bogus arg rc=$rc"

echo; echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
