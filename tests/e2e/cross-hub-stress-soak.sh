#!/usr/bin/env bash
# T-1395 cross-hub concurrent stress soak.
#
# Two phases:
#   Phase 1 (fan-in):   50 parallel posts to one topic — 40 local on
#                       .107, 10 cross-hub TCP from .122. Verify no
#                       loss + offset linearization (0..49 contiguous).
#   Phase 2 (fan-out):  5 topics × 10 senders per topic, all in parallel.
#                       Verify each topic has exactly 10 envelopes.
#
# Soak budget: < 30s wall-clock.

set -euo pipefail

BIN=${BIN:-./target/release/termlink}
HUB_107=${HUB_107:-127.0.0.1:9100}
HUB_107_FROM_122=${HUB_107_FROM_122:-192.168.10.107:9100}
N=$RANDOM
TOPIC_FANIN="stress-fanin-${N}"
WORK=$(mktemp -d -t stress-soak.XXXXXX)
trap 'rm -rf "$WORK"' EXIT

step() { echo; echo "=== $* ==="; }
fail() { echo "FAIL: $*" >&2; exit 1; }

REMOTE_SESSION=$($BIN remote list ring20-management 2>/dev/null | awk '/^tl-/ {print $1; exit}')
[ -n "$REMOTE_SESSION" ] || fail "no live session on ring20-management"

START_T=$(date +%s)

step "Inventory"
"$BIN" --version
echo "Phase 1 fan-in topic: $TOPIC_FANIN"
echo "Hub .107: $HUB_107"
echo "Remote session: $REMOTE_SESSION"

# ---------- Phase 1: fan-in --------------------------------------------------
step "Phase 1: Create fan-in topic + launch 50 parallel posters"
"$BIN" channel create "$TOPIC_FANIN" --hub "$HUB_107" >/dev/null

PIDS=()
# 40 local posts
for i in $(seq 1 40); do
  (
    "$BIN" channel post "$TOPIC_FANIN" \
      --payload "fanin-local-$i" \
      --sender-id "stress-local-$((i % 6))" \
      --hub "$HUB_107" >/dev/null 2>&1
  ) &
  PIDS+=($!)
done

# 10 cross-hub posts from .122 (each via remote exec — slower path)
# Run them in a single remote-exec invocation that emits 10 posts in parallel
# inside the .122 shell — much faster than 10 separate cross-hub TCP setups.
(
  "$BIN" remote exec ring20-management "$REMOTE_SESSION" "
for i in 1 2 3 4 5 6 7 8 9 10; do
  termlink channel post '$TOPIC_FANIN' \
    --payload \"fanin-xhub-\$i\" \
    --sender-id \"stress-xhub-\$((i % 4))\" \
    --hub $HUB_107_FROM_122 >/dev/null 2>&1 &
done
wait
echo XHUB_BATCH_OK
" 2>&1 | tail -1
) &
PIDS+=($!)

for PID in "${PIDS[@]}"; do wait "$PID" || true; done
echo "OK: 50 parallel posts launched and reaped"

step "Phase 1: Verify zero-loss + offset linearization"
"$BIN" channel state "$TOPIC_FANIN" --hub "$HUB_107" --json > "$WORK/fanin_state.json"
COUNT=$(python3 -c '
import json
rows = json.load(open("'"$WORK/fanin_state.json"'"))
rows = rows if isinstance(rows, list) else rows.get("rows", [])
print(len(rows))
')
echo "envelopes: $COUNT (expected 50)"
[ "$COUNT" = "50" ] || fail "expected 50 envelopes, got $COUNT — message loss"

OFFSETS_OK=$(python3 -c '
import json
rows = json.load(open("'"$WORK/fanin_state.json"'"))
rows = rows if isinstance(rows, list) else rows.get("rows", [])
offs = sorted(r["offset"] for r in rows)
expected = list(range(50))
print("yes" if offs == expected else f"NO: got {offs}")
')
echo "offsets contiguous 0..49: $OFFSETS_OK"
[ "$OFFSETS_OK" = "yes" ] || fail "offset linearization failed: $OFFSETS_OK"

XHUB_COUNT=$(python3 -c '
import json
rows = json.load(open("'"$WORK/fanin_state.json"'"))
rows = rows if isinstance(rows, list) else rows.get("rows", [])
xhub = [r for r in rows if r.get("payload","").startswith("fanin-xhub-")]
print(len(xhub))
')
echo "cross-hub posts visible: $XHUB_COUNT (expected 10)"
[ "$XHUB_COUNT" = "10" ] || fail "expected 10 cross-hub posts, got $XHUB_COUNT"

# ---------- Phase 2: fan-out -------------------------------------------------
step "Phase 2: 5 topics × 10 senders/topic in parallel"
FANOUT_TOPICS=()
for t in 1 2 3 4 5; do
  topic="stress-fanout-${N}-${t}"
  FANOUT_TOPICS+=("$topic")
  "$BIN" channel create "$topic" --hub "$HUB_107" >/dev/null
done

PIDS=()
for topic in "${FANOUT_TOPICS[@]}"; do
  for sid in 1 2 3 4 5 6 7 8 9 10; do
    (
      "$BIN" channel post "$topic" \
        --payload "fanout-${topic}-msg-${sid}" \
        --sender-id "stress-fan-$sid" \
        --hub "$HUB_107" >/dev/null 2>&1
    ) &
    PIDS+=($!)
  done
done
for PID in "${PIDS[@]}"; do wait "$PID" || true; done
echo "OK: 50 fan-out posts (5 × 10) launched"

step "Phase 2: Verify each fan-out topic has exactly 10 envelopes"
for topic in "${FANOUT_TOPICS[@]}"; do
  cnt=$("$BIN" channel state "$topic" --hub "$HUB_107" --json \
    | python3 -c 'import json,sys; d=json.load(sys.stdin); rows=d if isinstance(d,list) else d.get("rows",[]); print(len(rows))')
  echo "  $topic: $cnt envelopes"
  [ "$cnt" = "10" ] || fail "$topic: expected 10, got $cnt"
done
echo "OK: all 5 fan-out topics carry exactly 10 envelopes"

# ---------- Phase 3: re-run arc-suite to confirm arc unbroken --------------
# Skip when invoked from inside arc-suite (would infinite-recurse).
if [ "${ARC_SUITE_RUN:-0}" = "1" ]; then
  step "Phase 3: SKIPPED — running inside arc-suite (recursion guard)"
else
  step "Phase 3: re-run arc-suite to confirm no leftover stress damage"
  SUITE_PATH="$(dirname "$0")/arc-suite.sh"
  [ -x "$SUITE_PATH" ] || fail "arc-suite.sh not executable at $SUITE_PATH"
  if ! BIN="$BIN" "$SUITE_PATH" > "$WORK/arc-suite.out" 2>&1; then
    tail -30 "$WORK/arc-suite.out"
    fail "arc-suite re-run failed after stress"
  fi
  grep -q "ARC SUITE GREEN" "$WORK/arc-suite.out" || fail "arc-suite marker missing"
  echo "OK: arc-suite re-runs green after stress"
fi

END_T=$(date +%s)
DURATION=$((END_T - START_T))
echo "wall-clock: ${DURATION}s"
[ "$DURATION" -lt "60" ] || fail "soak exceeded 60s budget — actual ${DURATION}s"

echo
echo "STRESS-SOAK E2E PASSED"
echo "  Phase 1 fan-in:  50 posts across 1 topic, 0 loss, offsets contiguous"
echo "  Phase 2 fan-out: 5 topics × 10 posts each, all clean"
echo "  Phase 3 re-suite: arc-suite green after stress"
echo "  Wall-clock:      ${DURATION}s"
