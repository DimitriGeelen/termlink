#!/usr/bin/env bash
# T-3256 — scope fixture for scripts/check-receiver-ack-lag.sh.
#
# The guard's rows are SENDERS. On a broadcast rail the never-ackers are its own
# posting bots, so scanning agent-chat-arc / framework:pickup made the check
# permanently red (T-3007, R7 F26). These cases pin the ack-contract scope:
#
#   B1  broadcast rail full of never-acked bots, no dm topics → NOT scanned, rc 0,
#       and the run says "nothing to measure", never "all senders acked"
#   D1  dm:* topic with a sender behind threshold → scanned, fires rc 1
#   D2  dm:* topic all caught up + broadcast bots present → rc 0, exclusion counted
#   I1  include-listed non-dm topic IS scanned
#   E1  unreadable channel list → NO VERDICT rc 2
#   M1  mutant: restore the old broadcast defaults → B1 goes red
#   C1  CI + binary present + hub unreachable → SKIP rc 0 (T-3262, release-job shape)
#   C2  CI + reachable hub → measured normally
#   M2  mutant: binary-only CI skip → C1 goes red
#
# Mock termlink via TERMLINK_BIN. No hub, no network. Exit 0 pass / 1 fail.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/../scripts/check-receiver-ack-lag.sh"
[ -f "$SCRIPT" ] || { echo "FAIL: $SCRIPT not found"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required"; exit 1; }
W="$(mktemp -d -t ack-lag-scope.XXXXXX)" || { echo "FAIL: mktemp"; exit 1; }
trap 'rm -rf "$W"' EXIT

# mock <topics-csv> — `channel list` reports those topics; ack-status replies from
# $W/ack/<topic-with-colons-as-_>.json, or fails if absent.
mock() {
    mkdir -p "$W/ack"
    cat > "$W/termlink" <<MOCK
#!/usr/bin/env bash
case "\$1 \$2" in
  "channel list")
    [ -f "$W/list-broken" ] && exit 1
    printf '{"topics":['
    first=1
    for t in \$(printf '%s' "$1" | tr ',' ' '); do
      [ \$first = 1 ] || printf ','; first=0; printf '{"name":"%s","count":10}' "\$t"
    done
    printf ']}\n' ;;
  "channel ack-status")
    f="$W/ack/\$(printf '%s' "\$3" | tr ':' '_').json"
    [ -f "\$f" ] && cat "\$f" || exit 1 ;;
esac
MOCK
    chmod +x "$W/termlink"
}
ack() { printf '%s\n' "$2" > "$W/ack/$(printf '%s' "$1" | tr ':' '_').json"; }
NEVER='[{"sender_id":"botbotbotbotbotbot","lag":1896,"up_to":null}]'
BEHIND='[{"sender_id":"peerpeerpeerpeer01","lag":99,"up_to":3}]'
FINE='[{"sender_id":"peerpeerpeerpeer01","lag":0,"up_to":40}]'

run() { # <script> [args...] — output to $W/out, echo rc
    local s="$1"; shift
    (unset CI; TERMLINK_BIN="$W/termlink" bash "$s" "$@") >"$W/out" 2>&1
    echo $?
}
fail=0
pass() { echo "PASS: $1"; }
bad()  { echo "FAIL: $1"; sed 's/^/      /' "$W/out" | head -12; fail=1; }

case_b1() { # <script> — returns 0 when the property holds
    rm -rf "$W/ack" "$W/list-broken"; mock "agent-chat-arc,framework:pickup"
    ack agent-chat-arc "$NEVER"; ack framework:pickup "$NEVER"
    [ "$(run "$1")" = 0 ] && grep -q 'nothing to measure' "$W/out" && ! grep -q 'NEVER-ACKED' "$W/out"
}
case_b1 "$SCRIPT" && pass "B1 broadcast rails of never-acked bots are not scanned (rc 0, 'nothing to measure')" \
                  || bad "B1 broadcast rails still scanned or silent"

rm -rf "$W/ack"; mock "agent-chat-arc,dm:aaa:bbb"
ack agent-chat-arc "$NEVER"; ack dm:aaa:bbb "$BEHIND"
rc="$(run "$SCRIPT")"
[ "$rc" = 1 ] && grep -q 'BEHIND' "$W/out" && grep -q 'dm:aaa:bbb' "$W/out" && ! grep -q 'botbotbot' "$W/out" \
  && pass "D1 dm topic behind threshold fires rc 1; broadcast bot row absent" || bad "D1 rc=$rc"

ack dm:aaa:bbb "$FINE"
rc="$(run "$SCRIPT")"
[ "$rc" = 0 ] && grep -q '1 other topic(s) excluded' "$W/out" && grep -q 'agent-chat-arc framework:pickup' "$W/out" \
  && grep -q 'FINGERPRINT' "$W/out" \
  && pass "D2 caught-up dm → rc 0; exclusion counted + rails named; sender-keyed caveat kept" || bad "D2 rc=$rc"

rm -rf "$W/ack"; mock "agent-chat-arc,work-queue"
ack work-queue "$BEHIND"
rc="$( (ACK_LAG_INCLUDE_TOPICS="work-queue"; export ACK_LAG_INCLUDE_TOPICS; run "$SCRIPT") )"
[ "$rc" = 1 ] && grep -q 'work-queue' "$W/out" && pass "I1 include-listed non-dm topic is scanned (fires rc 1)" || bad "I1 rc=$rc"

touch "$W/list-broken"
rc="$(run "$SCRIPT")"
[ "$rc" = 2 ] && grep -q 'NO VERDICT' "$W/out" && pass "E1 unreadable channel list → NO VERDICT rc 2" || bad "E1 rc=$rc"
rm -f "$W/list-broken"

# T-3262 — the release job: CI set, a termlink binary PRESENT (cargo test built it),
# no reachable hub. Must SKIP rc 0 and say why; off CI the same state is E1 (rc 2).
run_ci() { local s="$1"; shift; (CI=true TERMLINK_BIN="$W/termlink" bash "$s" "$@") >"$W/out" 2>&1; echo $?; }
case_c1() { touch "$W/list-broken"; local r; r="$(run_ci "$1")"; rm -f "$W/list-broken"
            [ "$r" = 0 ] && grep -q 'no reachable hub' "$W/out"; }
case_c1 "$SCRIPT" && pass "C1 CI + binary present + hub unreachable → SKIP rc 0 naming the hub" || bad "C1 (CI, binary, no hub)"
rm -rf "$W/ack"; mock "dm:a:b"; ack "dm:a:b" "$BEHIND"
rc="$(run_ci "$SCRIPT")"
[ "$rc" = 1 ] && pass "C2 CI + reachable hub → still measures (fires rc 1), no skip" || bad "C2 rc=$rc"
# M2: the pre-T-3262 skip (keyed on the binary only) — C1 must go red.
M2="$W/mutant2.sh"
python3 - "$SCRIPT" "$M2" <<'PY'
import sys; s=open(sys.argv[1]).read()
a=s.index("  if ! timeout 15 \"$TL\" channel list --json"); b=s.index("  fi\n", a)+5
open(sys.argv[2],"w").write(s[:a]+s[b:])
PY
if cmp -s "$SCRIPT" "$M2"; then bad "M2 mutant did not apply"
elif case_c1 "$M2"; then bad "M2 mutant (binary-only CI skip) survived C1"
else pass "M2 mutant (binary-only CI skip) killed by C1"; fi

rm -rf "$W/ack"; mock "agent-chat-arc"; ack agent-chat-arc "$NEVER"
rc="$(run "$SCRIPT" --topics "agent-chat-arc")"
[ "$rc" = 1 ] && pass "X1 explicit --topics still overrides the scope (broadcast scanned on request)" || bad "X1 rc=$rc"

# M1: restore the pre-T-3256 broadcast defaults — B1 must go red.
M="$W/mutant.sh"
sed -e 's/^TOPICS=""$/TOPICS="agent-chat-arc framework:pickup"/' -e 's/^TOPICS_EXPLICIT=0$/TOPICS_EXPLICIT=1/' "$SCRIPT" > "$M"
if cmp -s "$SCRIPT" "$M"; then bad "M1 mutant did not apply"
elif case_b1 "$M"; then bad "M1 mutant (old broadcast defaults) survived B1"
else pass "M1 mutant (old broadcast defaults) killed by B1"; fi

echo
[ "$fail" -eq 0 ] && { echo "receiver-ack-lag-scope-fixtures: ALL PASS"; exit 0; }
echo "receiver-ack-lag-scope-fixtures: FAILURES"; exit 1
