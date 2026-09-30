#!/usr/bin/env bash
# T-3277 (SQ-19) — sender-classification fixtures for scripts/check-receiver-ack-lag.sh.
#
# ack-status has a row for every identity that POSTED anything, so the only poster
# on a topic read NEVER-ACKED with nothing inbound to ack (6 of 11 live rows), and a
# recipient that never posts had no row at all. These cases pin the refinement:
#
#   S1  sole content poster, never acked            -> SOLE-SENDER, not fired (rc 0)
#   S2  never acked while ANOTHER identity posted   -> NEVER-ACKED fires (rc 1)
#   S3  content senders unreadable                  -> NEVER-ACKED still fires, says "unrefined"
#   S4  a receipt from someone else is NOT content  -> still SOLE-SENDER (rc 0)
#   S5  hex party with no row, other party posted   -> RECIPIENT-SILENT reported, not fired
#   S6  named (non-hex) party                       -> "unresolved", never guessed
#   M1  mutant: sole-sender branch disabled         -> S1 goes red
#
# Mock termlink via TERMLINK_BIN. No hub, no network. Exit 0 pass / 1 fail.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/../scripts/check-receiver-ack-lag.sh"
[ -f "$SCRIPT" ] || { echo "FAIL: $SCRIPT not found"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required"; exit 1; }
W="$(mktemp -d -t ack-lag-sender.XXXXXX)" || { echo "FAIL: mktemp"; exit 1; }
trap 'rm -rf "$W"' EXIT

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "PASS: $1"; }
bad() { FAIL=$((FAIL+1)); echo "FAIL: $1"; sed 's/^/      /' "$W/out" | head -20; }

# mock <topic> — channel list reports <topic>; ack-status from $W/ack.json;
# subscribe from $W/sub.ndjson (absent => subscribe fails).
mock() {
    cat > "$W/termlink" <<MOCK
#!/usr/bin/env bash
case "\$1 \$2" in
  "channel list") printf '{"topics":[{"name":"%s","count":10}]}\n' "$1" ;;
  "channel ack-status") cat "$W/ack.json" ;;
  "channel subscribe") [ -f "$W/sub.ndjson" ] && cat "$W/sub.ndjson" || exit 1 ;;
esac
MOCK
    chmod +x "$W/termlink"
}
msg() { printf '{"sender_id":"%s","msg_type":"%s","offset":%s}\n' "$1" "$2" "$3"; }
run() { (unset CI; TERMLINK_BIN="$W/termlink" bash "$1") >"$W/out" 2>&1; echo $?; }
reset() { rm -f "$W/ack.json" "$W/sub.ndjson"; }

X=aaaa1111aaaa1111; Y=bbbb2222bbbb2222
T="dm:$X:$Y"

# S1 — sole poster
reset; mock "$T"
printf '[{"sender_id":"%s","lag":3,"up_to":null}]\n' "$X" > "$W/ack.json"
{ msg "$X" topic_metadata 0; msg "$X" chat 1; msg "$X" chat 2; } > "$W/sub.ndjson"
rc=$(run "$SCRIPT")
if [ "$rc" = "0" ] && grep -q "SOLE-SENDER  $X" "$W/out" && ! grep -q "NEVER-ACKED  $X" "$W/out"; then ok "S1 sole content poster -> SOLE-SENDER, rc 0"
else bad "S1 sole sender (rc=$rc)"; fi

# S2 — another identity posted content
reset; mock "$T"
printf '[{"sender_id":"%s","lag":3,"up_to":null},{"sender_id":"%s","lag":0,"up_to":2}]\n' "$X" "$Y" > "$W/ack.json"
{ msg "$X" chat 0; msg "$Y" chat 1; msg "$Y" chat 2; } > "$W/sub.ndjson"
rc=$(run "$SCRIPT")
if [ "$rc" = "1" ] && grep -q "NEVER-ACKED  $X" "$W/out"; then ok "S2 inbound content from another identity -> NEVER-ACKED fires"
else bad "S2 real never-acked (rc=$rc)"; fi

# S3 — subscribe unreadable: fail toward signal
reset; mock "$T"
printf '[{"sender_id":"%s","lag":3,"up_to":null}]\n' "$X" > "$W/ack.json"
rc=$(run "$SCRIPT")
if [ "$rc" = "1" ] && grep -q "NEVER-ACKED  $X" "$W/out" && grep -q "unrefined" "$W/out"; then ok "S3 unreadable senders -> unrefined NEVER-ACKED still fires"
else bad "S3 fail toward signal (rc=$rc)"; fi

# S4 — a receipt from another identity is not content
reset; mock "$T"
printf '[{"sender_id":"%s","lag":2,"up_to":null},{"sender_id":"%s","lag":0,"up_to":1}]\n' "$X" "$Y" > "$W/ack.json"
{ msg "$X" chat 0; msg "$X" chat 1; msg "$Y" receipt 2; } > "$W/sub.ndjson"
rc=$(run "$SCRIPT")
if [ "$rc" = "0" ] && grep -q "SOLE-SENDER  $X" "$W/out"; then ok "S4 receipts are not content -> still SOLE-SENDER"
else bad "S4 receipt not content (rc=$rc)"; fi

# S5 — hex party Y has no row; X posted content (X acked, so nothing fires)
reset; mock "$T"
printf '[{"sender_id":"%s","lag":0,"up_to":1}]\n' "$X" > "$W/ack.json"
{ msg "$X" chat 0; msg "$X" chat 1; msg "$X" receipt 2; } > "$W/sub.ndjson"
rc=$(run "$SCRIPT")
if [ "$rc" = "0" ] && grep -q "RECIPIENT-SILENT  $Y .*reported, not fired" "$W/out"; then ok "S5 silent hex party -> RECIPIENT-SILENT reported, not fired"
else bad "S5 recipient silent (rc=$rc)"; fi

# S6 — named party is unresolvable, never guessed
reset; mock "dm:$X:some-named-agent"
printf '[{"sender_id":"%s","lag":0,"up_to":1}]\n' "$X" > "$W/ack.json"
{ msg "$X" chat 0; msg "$X" chat 1; } > "$W/sub.ndjson"
rc=$(run "$SCRIPT")
if [ "$rc" = "0" ] && grep -q "unresolved   some-named-agent" "$W/out" && ! grep -q "RECIPIENT-SILENT  some-named" "$W/out"; then ok "S6 named party -> unresolved, not guessed"
else bad "S6 named party (rc=$rc)"; fi

# M1 — mutant: disable the sole-sender branch; S1 must go red
MUT="$W/mutant.sh"
sed 's/if \[ "\$senders_ok" = "1" \] && \[ -z "\$others" \]; then/if false; then/' "$SCRIPT" > "$MUT"
if cmp -s "$SCRIPT" "$MUT"; then FAIL=$((FAIL+1)); echo "FAIL: M1 mutant construction (pattern not found)"
else
    reset; mock "$T"
    printf '[{"sender_id":"%s","lag":3,"up_to":null}]\n' "$X" > "$W/ack.json"
    { msg "$X" chat 1; msg "$X" chat 2; } > "$W/sub.ndjson"
    rc=$(run "$MUT")
    if [ "$rc" = "1" ]; then ok "M1 mutant (sole-sender branch off) killed by S1"
    else bad "M1 mutant survived (rc=$rc)"; fi
fi

echo ""
[ "$FAIL" = "0" ] && { echo "receiver-ack-lag-sender-fixtures: ALL PASS ($PASS)"; exit 0; }
echo "receiver-ack-lag-sender-fixtures: $FAIL FAILED, $PASS passed"; exit 1
