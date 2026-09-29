#!/usr/bin/env bash
# T-3254 — load-bearing fixture for the chat-arc TAIL derivation in
# scripts/fleet-adoption-snapshot.sh on a hub that serves no `latest_offset`.
#
# Reproduces T-3004 F2/F3 through a mock `termlink`: agent-chat-arc is
# retention-trimmed (messages/1000), so its live offsets run 553..1553 while
# `channel info` reports count=1001. Only offsets >= 1400 are inside the window.
# A stale receipt up_to=2329 sits BEYOND the true tail (the .122 shape).
#
#   pre-fix  : tail = max(count-1, receipt) = 2329 → cursor 1829 → scans nothing → 0
#              (without the receipt: tail 1000 → cursor 500 → scans 553..1052 → 0)
#   fixed    : page forward from count-1 → tail 1553 → cursor 1053 → 154 in-window posts (1400..1553)
#
# The mock honours --cursor / --limit / --since the way the hub does: envelopes with
# offset >= cursor, at most `limit`, and only those at or after `since`.
# No live hub, no network. Exit 0 = pass, 1 = fail.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/../scripts/fleet-adoption-snapshot.sh"
[ -f "$SCRIPT" ] || { echo "FAIL: $SCRIPT not found"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required"; exit 1; }

WORK="$(mktemp -d -t fleet-adoption-tail.XXXXXX)" || { echo "FAIL: mktemp"; exit 1; }
trap 'rm -rf "$WORK"' EXIT

cat > "$WORK/hubs.toml" <<'TOML'
[hubs.testhub]
address = "127.0.0.1:9999"
TOML

LV="$WORK/listeners.sh"
printf '#!/usr/bin/env bash\nprintf "%%s\\n" %s\n' "'{\"live\":1}'" > "$LV"
chmod +x "$LV"

# mk_mock <info-json> — writes the mock termlink with the given channel-info reply
mk_mock() {
    cat > "$WORK/termlink" <<MOCKEOF
#!/usr/bin/env bash
case "\$*" in
  "channel info "*"agent-chat-arc"*) printf '%s\n' '$1' ;;
  "channel subscribe "*"agent-chat-arc"*)
    cursor=0; limit=100; since=0
    while [ \$# -gt 0 ]; do
      case "\$1" in --cursor) cursor="\$2"; shift ;; --limit) limit="\$2"; shift ;; --since) since="\$2"; shift ;; esac
      shift
    done
    now=\$(date +%s%3N); old=\$((now - 30*86400*1000))
    lo=\$cursor; [ "\$lo" -lt 553 ] && lo=553
    n=0; off=\$lo
    while [ "\$off" -le 1553 ] && [ "\$n" -lt "\$limit" ]; do
      if [ "\$off" -ge 1400 ]; then ts=\$now; else ts=\$old; fi
      if [ "\$ts" -ge "\$since" ]; then
        printf '{"offset":%s,"ts":%s,"msg_type":"chat","sender_id":"fp","metadata":{"agent_id":"a%s"}}\n' "\$off" "\$ts" "\$((off % 3))"
        n=\$((n+1))
      fi
      off=\$((off+1))
    done ;;
  "channel list "*) printf '%s\n' '{"topics":[]}' ;;
  *) : ;;
esac
MOCKEOF
    chmod +x "$WORK/termlink"
}
posts() { # prints fleet chat_arc_posts for the given snapshot script
    TERMLINK_BIN="$WORK/termlink" LISTENERS_VERB="$LV" \
        bash "$1" --hubs-file "$WORK/hubs.toml" --json 2>/dev/null \
        | jq -r '.summary.chat_arc_posts // "MISSING"' 2>/dev/null
}

fail=0
pass() { echo "PASS: $1"; }
bad()  { echo "FAIL: $1"; fail=1; }

TRIMMED='{"count":1001,"receipts":[{"up_to":2329}],"retention":{"kind":"messages","value":1000}}'
mk_mock "$TRIMMED"
got="$(posts "$SCRIPT")"
[ "$got" = 154 ] && pass "trimmed topic, no latest_offset, stale receipt beyond tail → 154 in-window posts (1400..1553)" \
                  || bad "trimmed/no-latest_offset/stale-receipt: expected 154, got '$got'"

mk_mock '{"count":1001,"retention":{"kind":"messages","value":1000}}'
got="$(posts "$SCRIPT")"
[ "$got" = 154 ] && pass "trimmed topic, no latest_offset, no receipt → 154" \
                  || bad "trimmed/no-latest_offset/no-receipt: expected 154, got '$got'"

mk_mock '{"count":1001,"latest_offset":1553,"retention":{"kind":"messages","value":1000}}'
got="$(posts "$SCRIPT")"
[ "$got" = 154 ] && pass "latest_offset present → used directly → 154" \
                  || bad "latest_offset path: expected 154, got '$got'"

# Mutant: the pre-fix derivation (count-1 vs receipt, no paging) must read 0 here —
# otherwise this fixture would not have caught T-3004.
mkdir -p "$WORK/s" && ln -s "$HERE/../scripts/lib" "$WORK/s/lib"   # the script loads ./lib
MUT="$WORK/s/mutant.sh"
python3 - "$SCRIPT" "$MUT" <<'PY'
import sys,re
s=open(sys.argv[1]).read()
a=s.index("            if [ -z \"$chat_tail\" ]; then\n                # T-3254")
b=s.index("            cursor=0\n", a)
pre='''            if [ -z "$chat_tail" ]; then
                chat_count_tail=0
                [ "$chat_count" -gt 0 ] && chat_count_tail=$((chat_count - 1))
                chat_receipt="$(printf '%s' "$info_raw" | jq -r '[.receipts[]?.up_to // empty] | max // empty' 2>/dev/null || true)"
                if [ -n "$chat_receipt" ] && [ "$chat_receipt" -gt "$chat_count_tail" ]; then chat_tail="$chat_receipt"; else chat_tail="$chat_count_tail"; fi
            fi
'''
open(sys.argv[2],"w").write(s[:a]+pre+s[b:])
PY
mk_mock "$TRIMMED"
got="$(posts "$MUT")"
[ "$got" = 0 ] && pass "mutant (pre-fix derivation) reproduces the T-3004 zero" \
                || bad "mutant (pre-fix derivation) expected 0, got '$got' — fixture no longer discriminates"

echo
if [ "$fail" -eq 0 ]; then echo "fleet-adoption-tail-fixtures: ALL PASS"; exit 0
else echo "fleet-adoption-tail-fixtures: FAILURES"; exit 1; fi
