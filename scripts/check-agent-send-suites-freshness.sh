#!/usr/bin/env bash
# T-3334 — daily run of the live-hub agent-send test suites.
#
# WHY THIS EXISTS
# ---------------
# scripts/test-agent-send-orchestration.sh was red from T-2479 until T-3331: agent-send
# exited 1 instead of its documented 3 ("not acked") on every give-up without a
# diagnosis. Nothing noticed, because nothing ran the suite. These suites need a LIVE hub,
# so the guard layer (CI, no hub) cannot run them, and no cron did. Same
# shipped-but-dark class as T-3288: a check that exists but never executes.
#
# DELIBERATELY NOT MARKED `# guard-layer: source` — it needs a live hub and posts to it
# (each suite reaps its own scratch topics). It is a RUNTIME canary and belongs to cron
# (.context/cron/agent-send-suites-canary.crontab). Its fixtures are hermetic and carry
# the marker.
#
# Verdict (T-2557 / T-2696 pattern):
#   hub unreachable (precheck)       -> exit 2, tooling, non-firing (/preflight territory)
#   a suite missing or not runnable  -> exit 2, tooling
#   every suite exits 0              -> exit 0, healthy
#   any suite exits non-zero         -> exit 1, FIRING; names the suite and its FAIL lines
#
# Usage: check-agent-send-suites-freshness.sh [--json] [--quiet] [--no-heartbeat]
# Seams (fixtures): AGENT_SEND_SUITES_DIR (dir holding the suites),
#   AGENT_SEND_SUITES (space-separated names), AGENT_SEND_CANARY_TEST_HUB_RC (precheck rc),
#   AGENT_SEND_CANARY_HEARTBEAT_FILE, AGENT_SEND_SUITE_TIMEOUT (seconds, default 900).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUITES_DIR="${AGENT_SEND_SUITES_DIR:-$ROOT/scripts}"
SUITES="${AGENT_SEND_SUITES:-test-agent-send.sh test-agent-send-orchestration.sh test-agent-send-transport.sh test-agent-send-auto-discover.sh}"
HEARTBEAT_FILE="${AGENT_SEND_CANARY_HEARTBEAT_FILE:-$ROOT/.context/working/.agent-send-suites-canary.heartbeat}"
TIMEOUT_S="${AGENT_SEND_SUITE_TIMEOUT:-900}"
json=0 quiet=0 hb=1
while [ $# -gt 0 ]; do
    case "$1" in
        --json) json=1; shift ;;
        --quiet) quiet=1; shift ;;
        --no-heartbeat) hb=0; shift ;;
        -h|--help) sed -n '2,27p' "$0"; exit 0 ;;
        *) echo "check-agent-send-suites: unknown arg: $1" >&2; exit 2 ;;
    esac
done
# Heartbeat on EXIT: freshness proves the run finished (T-2843), not that cron fired.
_hb() { mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null; date +%s > "$HEARTBEAT_FILE" 2>/dev/null || true; }
[ "$hb" -eq 1 ] && trap _hb EXIT

ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if [ -n "${AGENT_SEND_CANARY_TEST_HUB_RC:-}" ]; then hub_rc="$AGENT_SEND_CANARY_TEST_HUB_RC"
else timeout 15 termlink channel list --json >/dev/null 2>&1; hub_rc=$?; fi
if [ "$hub_rc" -ne 0 ]; then
    echo "check-agent-send-suites: local hub unreachable (rc=$hub_rc) — suites not run; this is a tooling state, not a finding" >&2
    exit 2
fi

results="" failed=0 missing=0
for s in $SUITES; do
    p="$SUITES_DIR/$s"
    if [ ! -r "$p" ]; then
        echo "check-agent-send-suites: suite missing: $p" >&2; missing=1; continue
    fi
    out="$(timeout "$TIMEOUT_S" bash "$p" 2>&1)"; rc=$?
    fails="$(printf '%s\n' "$out" | grep -E '^ *FAIL' | head -5 | sed 's/^ *//' | tr '\n' '|' | sed 's/|$//')"
    [ "$rc" -ne 0 ] && failed=1
    results="$results$(jq -cn --arg s "$s" --argjson rc "$rc" --arg f "$fails" '{suite:$s, rc:$rc, fails:$f}')"$'\n'
done

if [ "$json" -eq 1 ]; then
    printf '%s' "$results" | jq -cs --arg ts "$ts" --argjson f "$failed" --argjson m "$missing" \
        '{ts:$ts, ok:($f==0 and $m==0), firing:($f==1), missing:($m==1), suites:.}'
else
    printf '%s' "$results" | while IFS= read -r r; do
        [ -n "$r" ] || continue
        rc="$(printf '%s' "$r" | jq -r .rc)"; s="$(printf '%s' "$r" | jq -r .suite)"
        if [ "$rc" -ne 0 ]; then
            echo "=== $ts === FAILING $s rc=$rc: $(printf '%s' "$r" | jq -r .fails)"
        elif [ "$quiet" -eq 0 ]; then
            echo "ok $s"
        fi
    done
fi
[ "$failed" -eq 1 ] && exit 1
[ "$missing" -eq 1 ] && exit 2
exit 0
