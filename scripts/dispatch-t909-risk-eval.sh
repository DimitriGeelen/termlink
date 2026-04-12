#!/bin/bash
# dispatch-t909-risk-eval.sh — Fires 3 termlink workers in parallel to evaluate the
# symlink fix proposed in T-909 from different risk angles.
#
# Each worker is a detached Claude Code session fed a risk-eval prompt.
# Each writes its findings to docs/reports/T-909-symlink-fix-risk-<angle>.md
# and prints DONE-T909-<ANGLE> on completion.
#
# After dispatch, this script polls for the three output files and reports when all are ready.

set -euo pipefail
cd /opt/termlink

OUT_DIR="docs/reports"
mkdir -p "$OUT_DIR"

# Remove any stale outputs from prior runs
rm -f "$OUT_DIR/T-909-symlink-fix-risk-tech.md" \
      "$OUT_DIR/T-909-symlink-fix-risk-state.md" \
      "$OUT_DIR/T-909-symlink-fix-risk-blast.md"

echo "==> Dispatching 3 workers via termlink dispatch (background backend)"

# Worker 1: Technical / Path Resolution
termlink dispatch \
    --count 1 \
    --name risk-tech \
    --tags "T-909,risk-eval,technical" \
    --backend background \
    --timeout 900 \
    --json \
    -- bash -c 'claude -p "$(cat /tmp/t909-risk-tech.md)" --dangerously-skip-permissions' \
    > /tmp/t909-dispatch-tech.log 2>&1 &
TECH_PID=$!

# Worker 2: State / Data Preservation
termlink dispatch \
    --count 1 \
    --name risk-state \
    --tags "T-909,risk-eval,state" \
    --backend background \
    --timeout 900 \
    --json \
    -- bash -c 'claude -p "$(cat /tmp/t909-risk-state.md)" --dangerously-skip-permissions' \
    > /tmp/t909-dispatch-state.log 2>&1 &
STATE_PID=$!

# Worker 3: Multi-project Blast Radius
termlink dispatch \
    --count 1 \
    --name risk-blast \
    --tags "T-909,risk-eval,blast" \
    --backend background \
    --timeout 900 \
    --json \
    -- bash -c 'claude -p "$(cat /tmp/t909-risk-blast.md)" --dangerously-skip-permissions' \
    > /tmp/t909-dispatch-blast.log 2>&1 &
BLAST_PID=$!

echo "==> All 3 dispatches fired"
echo "    tech  pid=$TECH_PID  log=/tmp/t909-dispatch-tech.log"
echo "    state pid=$STATE_PID log=/tmp/t909-dispatch-state.log"
echo "    blast pid=$BLAST_PID log=/tmp/t909-dispatch-blast.log"
echo ""
echo "==> Polling for output files (max 15 minutes)"

DEADLINE=$(( $(date +%s) + 900 ))
while [ $(date +%s) -lt $DEADLINE ]; do
    COUNT=0
    [ -s "$OUT_DIR/T-909-symlink-fix-risk-tech.md" ] && COUNT=$((COUNT + 1))
    [ -s "$OUT_DIR/T-909-symlink-fix-risk-state.md" ] && COUNT=$((COUNT + 1))
    [ -s "$OUT_DIR/T-909-symlink-fix-risk-blast.md" ] && COUNT=$((COUNT + 1))
    echo "    $(date +%H:%M:%S)  $COUNT/3 files ready"
    if [ $COUNT -eq 3 ]; then
        echo "==> All 3 reports ready"
        break
    fi
    sleep 30
done

echo ""
echo "==> Status:"
ls -l "$OUT_DIR/T-909-symlink-fix-risk-"*.md 2>/dev/null || echo "    (no files)"
echo ""
echo "==> Dispatch logs:"
for name in tech state blast; do
    echo "    --- /tmp/t909-dispatch-$name.log (last 10 lines) ---"
    tail -10 "/tmp/t909-dispatch-$name.log" 2>/dev/null | sed 's/^/      /'
done
