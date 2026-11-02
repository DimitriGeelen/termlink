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
    rm -f "$TMP/topics/"* "$TMP/facts/"*; rm -rf "$TMP/hubs"
    for v in inbox-stale held-unsurfaced orphan-inboxes; do echo '[]' > "$TMP/facts/$v"; done
    : > "$TMP/ignore"; : > "$TMP/handled"
}

cat > "$TMP/termlink" <<STUB
#!/usr/bin/env bash
T="$TMP/topics"; R="$TMP/hubs"
STUB
cat >> "$TMP/termlink" <<'STUB'
# T-3653: --hub <addr> reads that hub's own topic dir; FAKE_DOWN_HUB cannot be reached.
hub=""; prev=""; for a in "$@"; do [ "$prev" = "--hub" ] && hub="$a"; prev="$a"; done
if [ -n "$hub" ]; then
    [ "$hub" = "${FAKE_DOWN_HUB:-}" ] && { echo "Hub rpc_call failed: No route to host" >&2; exit 1; }
    T="$R/$hub"; mkdir -p "$T"
fi
case "$1 $2" in
  "channel list")
      [ -n "${FAKE_LIST_FAIL:-}" ] && { echo "hub down" >&2; exit 1; }
      printf '{"topics":['; first=1
      for f in "$T"/*; do [ -e "$f" ] || continue; [ $first = 1 ] || printf ','; first=0
          printf '{"name":"%s"}' "$(basename "$f" | tr '+%' ':/')"; done
      printf ']}\n'; exit 0 ;;
  "channel subscribe")
      [ -n "${FAKE_READ_FAIL:-}" ] && exit 1
      cat "$T/$(printf '%s' "$3" | tr ':/' '+%')"; exit 0 ;;
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
: > "$TMP/hubs-none.toml"   # T-3653: hermetic by default, no remote hub is ever contacted
run() { UNACTED_TL="$TMP/termlink" UNACTED_FW="$TMP/fw" UNACTED_IGNORE="$TMP/ignore" \
        UNACTED_HANDLED="$TMP/handled" REMOTE_INBOX_HUBS_TOML="${HUBS_TOML:-$TMP/hubs-none.toml}" \
        REMOTE_INBOX_LOCAL_HOSTS="127.0.0.1" REMOTE_INBOX_TIMEOUT=5 bash "$SCRIPT" "$@"; }

# 1. the 055 shape: last real message 197 h old, only receipts after it
reset
post "dm+3bba+d199" 136 197 055-cockpit "Finding: claude-fw guard bug"
receipt "dm+3bba+d199" 137 197 136; receipt "dm+3bba+d199" 138 196 136
out="$(run)"; rc=$?
[ "$rc" = 1 ] && grep -q '^UNACTED  dm  dm:3bba:d199  offset 136  from=055-cockpit  age=197h  Finding: claude-fw guard bug' <<< "$out" \
    && ok "dm: an unanswered message (receipts after it do not count as an answer) is listed, exit 1" || bad "dm unanswered: rc=$rc $out"

# 2. answered: a later real message
post "dm+3bba+d199" 139 190 999-AEF "thanks, fixed in T-4900"
out="$(run)"; rc=$?
[ "$rc" = 0 ] && [ -z "$out" ] && ok "dm: answered topic is not listed, exit 0" || bad "dm answered: rc=$rc $out"
reset; post "dm+e+f" 1 60 050-pen "ask"; post "dm+e+f" 2 30 050-pen "ask again, still nothing"
out="$(run)"
grep -q 'dm:e:f  offset 2  from=050-pen' <<< "$out" && ok "dm: a sender's own follow-up with no answer is still listed" || bad "own follow-up: $out"
reset; post "dm+e+f" 1 400 010-tl "old exchange"; post "dm+e+f" 2 30 050-pen "new ask, two weeks later"
out="$(run)"
grep -q 'dm:e:f  offset 2  from=050-pen' <<< "$out" \
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
grep -q 'offset 16  from=0235-voice  to=999-AEF' <<< "$out" && grep -q 'offset 20  from=1023-vault  to=999-AEF' <<< "$out" \
    && ! grep -q 'offset 19' <<< "$out" \
    && ok "dm: shared mailbox — each message waits for its own to_project; one answered by its addressee is not listed" || bad "shared mailbox: $out"

# 3. threshold and window
reset; post "dm+a+b" 1 2 x "too new"; post "dm+c+d" 1 400 x "older than the window"
out="$(run)"; rc=$?
[ "$rc" = 0 ] && [ -z "$out" ] && ok "dm: younger than 24 h and older than 14 days are not listed" || bad "dm window: rc=$rc $out"
out="$(run --threshold-hours 1)"
grep -q 'dm:a:b  offset 1' <<< "$out" && ok "dm: --threshold-hours moves the lower bound" || bad "threshold flag: $out"

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
grep -q 'dm:c:d  offset 7' <<< "$out" && ok "dm: a later message re-flags a handled topic" || bad "handled re-flag: $out"

# 6. other rails, each listed by name (guards against a rail silently dropped)
reset
echo '[{"topic":"inbox:h/010","unread":2,"age_hours":30.5,"oldest_from":"055"}]' > "$TMP/facts/inbox-stale"
echo '[{"msg_id":"m9","age_hours":48,"from":"aef"}]' > "$TMP/facts/held-unsurfaced"
echo '[{"topic":"inbox:h/ring20-manager","unread":14}]' > "$TMP/facts/orphan-inboxes"
post "dm+a+b" 1 30 x "ask"
out="$(run)"; rc=$?
n="$(printf '%s\n' "$out" | grep -c '^UNACTED')"
[ "$rc" = 1 ] && [ "$n" = 4 ] \
    && grep -q '^UNACTED  inbox  inbox:h/010  unread=2  age_hours=30.5  oldest_from=055' <<< "$out" \
    && grep -q '^UNACTED  held  m9  age_hours=48  from=aef' <<< "$out" \
    && grep -q '^UNACTED  orphan  inbox:h/ring20-manager  unread=14' <<< "$out" \
    && ok "all four rails report (dm, inbox, held, orphan); orphan-inboxes' own exit 1 is a finding, not a failure" \
    || bad "four rails: rc=$rc n=$n $out"

# 7. fail-closed: could not look is exit 2, never clean
reset
out="$(FAKE_LIST_FAIL=1 run)"; rc=$?
[ "$rc" = 2 ] && grep -q '^NOT CHECKED dm: termlink channel list --prefix dm: failed' <<< "$out" \
    && ok "dm: the hub cannot be listed => NOT CHECKED, exit 2" || bad "dm list fail: rc=$rc $out"
post "dm+a+b" 1 30 x "ask"
out="$(FAKE_READ_FAIL=1 run)"; rc=$?
[ "$rc" = 2 ] && grep -q '^NOT CHECKED dm: dm:a:b could not be read' <<< "$out" \
    && ok "dm: a topic that cannot be read => NOT CHECKED, exit 2" || bad "dm read fail: rc=$rc $out"
out="$(FAKE_FW_FAIL=held-unsurfaced run)"; rc=$?
[ "$rc" = 1 ] && grep -q '^NOT CHECKED held: fw sidecar held-unsurfaced failed' <<< "$out" \
    && grep -q '^UNACTED  dm  dm:a:b' <<< "$out" \
    && ok "a finding wins over a read error (exit 1), the NOT CHECKED line is still printed" || bad "finding+blind: rc=$rc $out"
echo 'not json' > "$TMP/facts/inbox-stale"; rm -f "$TMP/topics/"*
out="$(run)"; rc=$?
[ "$rc" = 2 ] && grep -q '^NOT CHECKED inbox:' <<< "$out" \
    && ok "a fact verb answering non-JSON => NOT CHECKED, exit 2" || bad "non-json: rc=$rc $out"
out="$(run --bogus 2>&1)"; rc=$?
[ "$rc" = 2 ] && ok "unknown argument => exit 2" || bad "bogus arg rc=$rc"

# ---- T-3653: our inboxes on REMOTE hubs ------------------------------------------
cat > "$TMP/hubs.toml" <<'T'
[hubs.local-test]
address = "127.0.0.1:9100"
[hubs.ring20-management]
address = "10.9.9.2:9100"
[hubs.laptop]
address = "10.9.9.3:9100"
T
rpost() {  # <hub> <topic file> <offset> <hours ago> <msg_type> <from_project> <body> [extra metadata]
    mkdir -p "$TMP/hubs/$1"
    printf '{"offset":%s,"msg_type":"%s","sender_id":"9219671e28054458","ts":%s,"metadata":{"from_project":"%s"%s},"payload_b64":"%s"}\n' \
        "$3" "$5" "$(h "$4")" "$6" "${8:+,$8}" "$(b64 "$7")" >> "$TMP/hubs/$1/$2"
}
remote_world() {
    reset
    rpost 10.9.9.2:9100 "inbox+22c1%010-termlink" 0 40 sidecar.receipt ring20 "[sidecar receipt] RECEIVED" '"kind":"receipt","receipt_for":"x"'
    rpost 10.9.9.2:9100 "inbox+22c1%010-termlink" 1 400 turn ring20 "too old, past the window"
    rpost 10.9.9.2:9100 "inbox+22c1%010-termlink" 2 30 turn ring20 "a likely CLI bug for you"
    rpost 10.9.9.2:9100 "inbox+22c1%010-termlink" 3 29 turn 010-termlink "our own post"
    rpost 10.9.9.2:9100 "inbox+22c1%010-termlink" 4 2 turn ring20 "too young for the threshold"
    rpost 10.9.9.2:9100 "inbox+22c1%ring20%010-termlink" 7 30 sidecar.consult ring20 "sender-keyed sub-inbox"
    rpost 10.9.9.2:9100 "inbox+22c1%other-project" 9 30 turn ring20 "not addressed to us"
    rpost 127.0.0.1:9100 "inbox+cacc%010-termlink" 5 30 turn ring20 "local hub: never read as remote"
}
remote_world
out="$(HUBS_TOML="$TMP/hubs.toml" run)"; rc=$?
[ "$rc" = 1 ] && grep -q '^UNACTED  remote  inbox:22c1/010-termlink@10.9.9.2:9100  offset 2  from=ring20  age=30h  a likely CLI bug for you' <<< "$out" \
    && grep -q '^UNACTED  remote  inbox:22c1/ring20/010-termlink@10.9.9.2:9100  offset 7  from=ring20' <<< "$out" \
    && [ "$(grep -c '^UNACTED  remote' <<< "$out")" = 2 ] \
    && ok "remote: peer mail on our inbox (and a sender-keyed sub-inbox) on another hub is listed; receipts, our own, too young, too old and other projects' inboxes are not" \
    || bad "remote listing: rc=$rc $out"
grep -q '127.0.0.1' <<< "$out" && bad "remote: the local hub was read as a remote one ($out)" \
    || ok "remote: a hubs.toml profile pointing at this host is not read as remote"
printf '%s\n' "inbox:22c1/010-termlink@10.9.9.2:9100 2 2026-11-02 answered in T-3652" \
    "inbox:22c1/ring20/010-termlink@10.9.9.2:9100 7 2026-11-02 answered" > "$TMP/handled"
out="$(HUBS_TOML="$TMP/hubs.toml" run)"; rc=$?
[ "$rc" = 0 ] && ! grep -q 'remote' <<< "$out" \
    && ok "remote: offsets recorded as <topic>@<hub> in the handled file are dismissed, exit 0" || bad "remote handled: rc=$rc $out"
out="$(FAKE_DOWN_HUB=10.9.9.3:9100 HUBS_TOML="$TMP/hubs.toml" run)"; rc=$?
[ "$rc" = 2 ] && grep -q '^NOT CHECKED remote: 10.9.9.3:9100 (laptop) channel list failed' <<< "$out" \
    && ok "remote: an unreachable hub is NOT CHECKED (exit 2), never clean" || bad "remote down: rc=$rc $out"
: > "$TMP/handled"
out="$(FAKE_DOWN_HUB=10.9.9.3:9100 HUBS_TOML="$TMP/hubs.toml" run)"; rc=$?
[ "$rc" = 1 ] && grep -q '^NOT CHECKED remote: 10.9.9.3:9100' <<< "$out" && grep -q '^UNACTED  remote' <<< "$out" \
    && ok "remote: a finding on one hub wins over another hub being down (exit 1, both printed)" || bad "remote find+down: rc=$rc $out"
out="$(HUBS_TOML="$TMP/no-such-hubs.toml" run)"; rc=$?
[ "$rc" = 2 ] && grep -q '^NOT CHECKED remote: could not read hubs file' <<< "$out" \
    && ok "remote: an unreadable hubs file is NOT CHECKED, exit 2" || bad "remote no toml: rc=$rc $out"
printf '#!/usr/bin/env bash\nsleep 30\n' > "$TMP/slow-tl"
printf '#!/usr/bin/env bash\nexec python3 %s "$@" --tl %s\n' "$PROJECT_ROOT/scripts/lib/remote-inbox-mail.py" "$TMP/slow-tl" > "$TMP/remote-slow"
chmod +x "$TMP/slow-tl" "$TMP/remote-slow"
start=$(date +%s)
out="$(HUBS_TOML="$TMP/hubs.toml" REMOTE_INBOX_TIMEOUT=1 UNACTED_REMOTE="$TMP/remote-slow" run)"; rc=$?
el=$(( $(date +%s) - start ))
[ "$el" -lt 20 ] && grep -q '^NOT CHECKED remote: 10.9.9.2:9100' <<< "$out" \
    && ok "remote: a hanging hub is cut off by the timeout and NOT CHECKED (${el}s)" || bad "remote hang: ${el}s rc=$rc $out"

echo; echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
