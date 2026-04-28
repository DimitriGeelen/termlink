#!/usr/bin/env bash
# T-1398 self-validating arc health monitor.
#
# Runs the arc-suite N times in sequence, posting PASS/FAIL + duration
# to a `arc-health:report` topic via channel.post. Cross-hub verification
# at the end reads the same topic from .122 to confirm the validator's
# outputs are visible cross-hub. The arc validates itself, then uses
# the arc to report on its own validation. Eat-your-own-dog-food.

set -euo pipefail

BIN=${BIN:-./target/release/termlink}
HUB_107=${HUB_107:-127.0.0.1:9100}
HUB_107_FROM_122=${HUB_107_FROM_122:-192.168.10.107:9100}
RUNS=${RUNS:-5}
TOPIC="arc-health:report:$(date -u +%Y%m%d-%H%M%S)"
SUITE="$(dirname "$0")/arc-suite.sh"
WORK=$(mktemp -d -t arc-health.XXXXXX)
trap 'rm -rf "$WORK"' EXIT

step() { echo; echo "=== $* ==="; }
fail() { echo "FAIL: $*" >&2; exit 1; }

REMOTE_SESSION=$($BIN remote list ring20-management 2>/dev/null | awk '/^tl-/ {print $1; exit}')
[ -n "$REMOTE_SESSION" ] || fail "no live session on ring20-management"

[ -x "$SUITE" ] || fail "arc-suite.sh not executable at $SUITE"

step "Inventory"
"$BIN" --version
echo "Suite:  $SUITE"
echo "Runs:   $RUNS"
echo "Topic:  $TOPIC"

step "Step 1: Create arc-health topic"
"$BIN" channel create "$TOPIC" --retention messages:200 --hub "$HUB_107" >/dev/null
echo "OK"

step "Step 2: Run arc-suite $RUNS times, post each result"
declare -a DURATIONS=()
PASS_COUNT=0
FAIL_COUNT=0

for i in $(seq 1 "$RUNS"); do
  echo "  --- Run $i/$RUNS ---"
  start=$(date +%s)
  if BIN="$BIN" "$SUITE" > "$WORK/run-$i.out" 2>&1; then
    end=$(date +%s)
    dur=$((end - start))
    DURATIONS+=("$dur")
    if grep -q "ARC SUITE GREEN" "$WORK/run-$i.out"; then
      result="PASS"
      PASS_COUNT=$((PASS_COUNT + 1))
    else
      result="FAIL_NO_MARKER"
      FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
  else
    end=$(date +%s)
    dur=$((end - start))
    DURATIONS+=("$dur")
    result="FAIL_RC"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi

  payload=$(printf 'run=%d result=%s duration_s=%d ts=%s' "$i" "$result" "$dur" "$(date -Iseconds)")
  "$BIN" channel post "$TOPIC" \
    --payload "$payload" \
    --sender-id "arc-health-monitor" \
    --metadata "run_index=$i" --metadata "result=$result" --metadata "duration_s=$dur" \
    --hub "$HUB_107" >/dev/null
  echo "    $result in ${dur}s — posted to $TOPIC"

  # If a run failed, dump the tail for debug + bail before posting more.
  if [ "$result" != "PASS" ]; then
    echo
    echo "  Run $i FAILED — last 30 lines:"
    tail -30 "$WORK/run-$i.out"
    break
  fi
done

step "Step 3: Compute summary stats"
if [ "${#DURATIONS[@]}" -gt 0 ]; then
  STATS=$(printf '%s\n' "${DURATIONS[@]}" | python3 -c '
import sys, statistics
xs = [int(x) for x in sys.stdin.read().split()]
xs.sort()
print(f"min={min(xs)}s max={max(xs)}s median={int(statistics.median(xs))}s mean={statistics.mean(xs):.1f}s")
')
else
  STATS="(no runs completed)"
fi
echo "passes:    $PASS_COUNT/$RUNS"
echo "failures:  $FAIL_COUNT"
echo "stats:     $STATS"

step "Step 4: Cross-hub READ — verify all run reports visible from .122"
"$BIN" remote exec ring20-management "$REMOTE_SESSION" \
  "termlink channel state '$TOPIC' --hub $HUB_107_FROM_122 --json" \
  > "$WORK/state_122_raw.txt" 2>/dev/null || fail "cross-hub state read failed"

python3 - <<PY > "$WORK/state_122.json"
import sys
raw = open("$WORK/state_122_raw.txt").read()
def extract(s, openc, closec):
    depth=0; start=-1
    for i,c in enumerate(s):
        if c==openc:
            if depth==0: start=i
            depth+=1
        elif c==closec:
            depth-=1
            if depth==0 and start>=0:
                return s[start:i+1]
    return None
out = extract(raw,'[',']') or extract(raw,'{','}')
if not out: sys.exit("no JSON")
sys.stdout.write(out)
PY

ROW_COUNT=$(python3 -c '
import json
rows = json.load(open("'"$WORK/state_122.json"'"))
rows = rows if isinstance(rows, list) else rows.get("rows", [])
print(len(rows))
')
echo "rows seen from .122: $ROW_COUNT (expected $PASS_COUNT or $((PASS_COUNT + FAIL_COUNT)))"
EXPECTED_ROWS=$((PASS_COUNT + FAIL_COUNT))
[ "$ROW_COUNT" = "$EXPECTED_ROWS" ] || fail "cross-hub row count mismatch — 107 saw $EXPECTED_ROWS posts, .122 saw $ROW_COUNT"

step "Step 5: Final assertion — all runs must have passed"
[ "$FAIL_COUNT" = "0" ] || fail "$FAIL_COUNT/$RUNS suite runs failed"

echo
echo "ARC HEALTH MONITOR OK"
echo "  Topic:    $TOPIC"
echo "  Runs:     $RUNS (all pass)"
echo "  Stats:    $STATS"
echo "  Cross-hub report visibility: $ROW_COUNT/$EXPECTED_ROWS rows seen from .122"
