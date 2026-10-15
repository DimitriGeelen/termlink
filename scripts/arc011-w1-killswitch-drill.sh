#!/usr/bin/env bash
# arc011-w1-killswitch-drill.sh — the W1 kill-switch drill on the real pair (T-3426, arc-011 PH-2 W1).
#
# W1's exit asks that the kill switch is exercised (design 6.2.3.b) and that a SIGKILLed sidecar
# comes back under systemd with its stored messages intact (R-8.e, design 6.5 item 7). The fixtures
# proved both against fakes; this proves them on the two production sidecars.
#
#   1  preflight   both listed sidecars answer and report delivery ON (never drill from an unknown state)
#   2  switch off  the fleet kill switch (`termlink sidecar admin delivery off --all --list`), read back
#   3  probe       the main sidecar sends one probe to the partner; the partner reports it STORED
#   4  refused     claiming the probe at the partner is refused with `delivery-off`  <- the switch works
#   5  switch on   read back ON; the same claim now gets past the switch (any answer but delivery-off)
#                  <- the control: the switch, and nothing else, was what refused in step 4
#   6  restart     SIGKILL the partner's MainPID; systemd starts a new one that answers status,
#                  and the probe is still STORED under the same id (R-8.e, R-15.e)
#
# Delivery is put back ON on every exit path once step 2 ran, and that is reported.
# The probe says it is a drill; the partner session may see it at its next prompt, which is
# real delivery after the switch is on again.
#
# Exit: 0 every step passed · 1 a step failed (named) · 2 tooling (nothing was switched)
# Seams (fixtures only): DRILL_CALL DRILL_TL DRILL_SOCK_DIR DRILL_LIST DRILL_SYSTEMCTL DRILL_KILL
#                        DRILL_WAIT_SECS DRILL_FROM DRILL_TO
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

FROM="${DRILL_FROM:-claude-termlink}"
TO="${DRILL_TO:-claude-termlink-alt}"
SOCK_DIR="${DRILL_SOCK_DIR:-/var/lib/termlink/sidecar}"
LIST="${DRILL_LIST:-$ROOT/.context/working/sidecar-admin-sockets.list}"
TL="${DRILL_TL:-termlink}"
SYSTEMCTL="${DRILL_SYSTEMCTL:-systemctl}"
KILL="${DRILL_KILL:-kill}"
WAIT="${DRILL_WAIT_SECS:-60}"
UNIT="termlink-sidecar@010-termlink--$TO.service"
WHY="W1 kill-switch drill (T-3426)"

# One JSON request line to a sidecar socket, one JSON answer line back.
call() {  # <socket> <json>
    if [ -n "${DRILL_CALL:-}" ]; then $DRILL_CALL "$1" "$2"; return; fi
    python3 -c '
import socket, sys
s = socket.socket(socket.AF_UNIX); s.settimeout(20); s.connect(sys.argv[1])
s.sendall(sys.argv[2].encode() + b"\n")
buf = b""
while not buf.endswith(b"\n"):
    c = s.recv(65536)
    if not c: break
    buf += c
sys.stdout.write(buf.decode())' "$1" "$2"
}
field() {  # <json> <python expr on d> -> value or empty
    python3 -c 'import json,sys
try: d=json.loads(sys.argv[1])
except Exception: sys.exit(0)
try: v=eval(sys.argv[2])
except Exception: sys.exit(0)
print("" if v is None else v)' "$1" "$2"
}

PASSED=0; FAILED=0; SWITCHED=0
pass() { PASSED=$((PASSED+1)); echo "  PASS  $*"; }
fail() { FAILED=$((FAILED+1)); echo "  FAIL  $*"; }

restore() {
    [ "$SWITCHED" = 1 ] || return 0
    if $TL sidecar admin delivery on --why "$WHY: restore" --all --list "$LIST" >/dev/null 2>&1; then
        echo "  restore  delivery back ON on every listed sidecar (read back)"
    else
        echo "  restore  FAILED to put delivery back ON — run: /opt/termlink/runme.sh --sidecar-delivery on --why \"drill restore\""
        FAILED=$((FAILED+1))
    fi
    SWITCHED=0
}
trap 'restore' EXIT

delivery_of() {  # <agent> -> on|off|empty
    field "$(call "$SOCK_DIR/$1.admin.sock" '{"op":"delivery","state":"query"}' 2>/dev/null)" 'd.get("delivery")'
}
stage_of() {  # <agent> <msg_id> -> the stage the agent's status reports for that message
    local out; out="$(call "$SOCK_DIR/$1.status.sock" "{\"op\":\"status\",\"msg_id\":\"$2\"}" 2>/dev/null)"
    python3 -c 'import json,sys
try: d=json.loads(sys.argv[1])
except Exception: sys.exit(0)
for m in d.get("messages",[]):
    if m.get("msg_id")==sys.argv[2]: print(m.get("stage","")); break' "$out" "$2"
}
claim_code() {  # <agent> <msg_id> -> the refusal code, or "claimed"
    local out; out="$(call "$SOCK_DIR/$1.send.sock" "{\"op\":\"claim\",\"msg_id\":\"$2\"}" 2>/dev/null)"
    field "$out" '"claimed" if d.get("claimed") else d.get("code")'
}

echo "W1 kill-switch drill: $FROM -> $TO ($(date -u +%FT%TZ))"

# 1 preflight
[ -s "$LIST" ] || { echo "  TOOLING  no admin-socket list at $LIST (W1 not provisioned?)"; exit 2; }
for a in "$FROM" "$TO"; do
    st="$(delivery_of "$a")"
    [ "$st" = "on" ] || { echo "  TOOLING  $a: delivery reads '${st:-no answer}', not on — not drilling from an unknown state"; exit 2; }
done
pass "1 preflight: both sidecars answer, delivery on"

# 2 switch off
if $TL sidecar admin delivery off --why "$WHY" --all --list "$LIST" >/dev/null 2>&1; then SWITCHED=1; fi
if [ "$SWITCHED" = 1 ] && [ "$(delivery_of "$FROM")" = off ] && [ "$(delivery_of "$TO")" = off ]; then
    pass "2 kill switch off: both read back off"
else
    SWITCHED=1; fail "2 kill switch off did not read back off on both"; exit 1
fi

# 3 probe
resp="$(call "$SOCK_DIR/$FROM.send.sock" "$(python3 -c 'import json,sys; print(json.dumps({"op":"send","to":sys.argv[1],"body":"W1 kill-switch drill probe from the main session (T-3426). No reply needed."}))' "$TO")" 2>/dev/null)"
MSG="$(field "$resp" 'd.get("msg_id")')"
[ -n "$MSG" ] || { fail "3 probe: the send was not accepted ($resp)"; exit 1; }
stage=""; for _ in $(seq 1 "$WAIT"); do stage="$(stage_of "$TO" "$MSG")"; [ -n "$stage" ] && break; sleep 1; done
case "$stage" in
    STORED) pass "3 probe $MSG reached $TO (stage $stage)" ;;
    *) fail "3 probe $MSG did not reach $TO within ${WAIT}s (stage '${stage:-none}')"; exit 1 ;;
esac

# 4 refused while off
code="$(claim_code "$TO" "$MSG")"
if [ "$code" = "delivery-off" ]; then pass "4 claim refused with delivery-off: the switch stops hand-over"
else fail "4 claim while off answered '${code:-nothing}', expected delivery-off"; fi

# 5 switch on + control
restore
if [ "$(delivery_of "$FROM")" = on ] && [ "$(delivery_of "$TO")" = on ]; then pass "5a kill switch on: both read back on"
else fail "5a kill switch on did not read back on"; fi
code="$(claim_code "$TO" "$MSG")"
if [ -n "$code" ] && [ "$code" != "delivery-off" ]; then pass "5b control: the same claim now passes the switch (answer: $code)"
else fail "5b control: the claim still answers '${code:-nothing}' with delivery on"; fi

# 6 SIGKILL + restart (R-8.e)
old="$($SYSTEMCTL show -p MainPID --value "$UNIT" 2>/dev/null)"
case "$old" in ''|0|*[!0-9]*) fail "6 restart: no MainPID for $UNIT"; old="" ;; esac
if [ -n "$old" ]; then
    $KILL -9 "$old" 2>/dev/null
    new=""; for _ in $(seq 1 "$WAIT"); do
        new="$($SYSTEMCTL show -p MainPID --value "$UNIT" 2>/dev/null)"
        if [ -n "$new" ] && [ "$new" != 0 ] && [ "$new" != "$old" ] && [ "$(delivery_of "$TO")" = on ]; then break; fi
        new=""; sleep 1
    done
    if [ -n "$new" ]; then pass "6a SIGKILL $old: systemd started $new, which answers with delivery on"
    else fail "6a after SIGKILL $old no new sidecar answered within ${WAIT}s"; fi
    stage="$(stage_of "$TO" "$MSG")"
    if [ -n "$stage" ]; then pass "6b probe $MSG survived the kill (stage $stage)"
    else fail "6b probe $MSG is gone after the restart"; fi
fi

echo "W1 kill-switch drill: $PASSED passed, $FAILED failed (probe $MSG)"
[ "$FAILED" = 0 ]
