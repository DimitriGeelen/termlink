#!/usr/bin/env bash
# T-2231 — black-box tests for check-framework-pickup-freshness.sh.
# Exercises the canary against the live framework:pickup topic using a
# throwaway marker file (never touches the real marker). Asserts the
# healthy/firing/ack paths and the cron no-leak guard (T-2225 lesson).
set -u
CANARY="scripts/check-framework-pickup-freshness.sh"
pass=0; fail=0
ok(){ echo "  PASS: $1"; pass=$((pass+1)); }
no(){ echo "  FAIL: $1"; fail=$((fail+1)); }

MARK="$(mktemp)"; trap 'rm -f "$MARK"' EXIT

# Derive current max offset from JSON (no hardcoded offsets).
MAX="$(echo -1 > "$MARK"; FW_PICKUP_CANARY_MARKER="$MARK" bash "$CANARY" --json 2>/dev/null | python3 -c 'import sys,json;print(json.load(sys.stdin)["max_offset"])')"
if ! printf '%s' "$MAX" | grep -qE '^[0-9]+$'; then
    echo "SETUP FAIL: could not read max_offset (hub down?). MAX='$MAX'"; exit 2
fi
echo "Live framework:pickup max_offset=$MAX"

# T1: marker == max → healthy, exit 0
echo "$MAX" > "$MARK"
FW_PICKUP_CANARY_MARKER="$MARK" bash "$CANARY" --quiet >/dev/null 2>&1
[ $? -eq 0 ] && ok "T1 marker==max → exit 0 (healthy)" || no "T1 expected exit 0"

# T2: cron no-leak — quiet + healthy produces ZERO bytes on BOTH streams
echo "$MAX" > "$MARK"
OUT="$(FW_PICKUP_CANARY_MARKER="$MARK" bash "$CANARY" --quiet 2>&1)"
[ -z "$OUT" ] && ok "T2 quiet healthy → 0 bytes (no leak)" || no "T2 leaked ${#OUT} bytes: $OUT"

# T3: marker == max-1 → fires (exit 1) and names the top offset
if [ "$MAX" -ge 1 ]; then
    echo $((MAX-1)) > "$MARK"
    OUT="$(FW_PICKUP_CANARY_MARKER="$MARK" bash "$CANARY" --quiet 2>&1)"; RC=$?
    if [ "$RC" -eq 1 ] && echo "$OUT" | grep -q "off=$MAX"; then ok "T3 marker==max-1 → exit 1 + names off=$MAX"
    else no "T3 expected exit 1 naming off=$MAX (rc=$RC)"; fi
else
    echo "  SKIP: T3 (need >=1 filing)"
fi

# T4: --ack bumps marker to current max
echo -1 > "$MARK"
FW_PICKUP_CANARY_MARKER="$MARK" bash "$CANARY" --ack --quiet >/dev/null 2>&1
GOT="$(cat "$MARK")"
[ "$GOT" = "$MAX" ] && ok "T4 --ack sets marker to max ($MAX)" || no "T4 marker='$GOT' expected '$MAX'"

# T5: after ack, next run is healthy exit 0
FW_PICKUP_CANARY_MARKER="$MARK" bash "$CANARY" --quiet >/dev/null 2>&1
[ $? -eq 0 ] && ok "T5 post-ack run → exit 0 (healthy)" || no "T5 expected exit 0 after ack"

echo ""
echo "Results: $pass pass / $fail fail"
[ "$fail" -eq 0 ]
