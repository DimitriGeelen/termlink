#!/usr/bin/env bash
# relay-hop-check.sh — T-2395 (relay-loop B3): deterministic hop-budget
# circuit-breaker for the self-advancing agent exchange.
#
# The relay loop (T-2393) lets two agents advance a thread hop-by-hop without a
# human nudge (B2 continuation contract). B3 stops it from ping-ponging forever:
# each turn carries a `relay_hops` counter (stamped by agent-send.sh on
# initiation and incremented by the reply via agent-respond.sh --relay-hops).
# Before a woken agent auto-replies (check-arc.md Step 6a), it runs THIS helper;
# on `verdict=stop` it surfaces the hop-budget-exhausted blocker LOUDLY and
# halts instead of continuing the loop.
#
# Reads the LATEST turn on (topic, cid), extracts metadata.relay_hops (absent →
# 0), compares to TERMLINK_RELAY_MAX_HOPS (default 4). Bounded autonomy is the
# default (IW-1) — we never trade the nudging problem for a runaway problem.
#
#   Output (stdout, single line):
#     verdict=continue hops=<N> cap=<M> next_hops=<N+1>
#     verdict=stop     hops=<N> cap=<M> reason=hop-budget-exhausted
#   Exit: 0 continue | 10 stop | 2 usage/precondition error
#
# Hub-independent test seam (PL-213, mirror of TERMLINK_LISTENERS_TEST_JSON):
#   TERMLINK_RELAY_HOPCHECK_TEST_JSON=<file> feeds canned `channel subscribe
#   --json` output (NDJSON or a JSON array of envelopes) — no live hub required.
set -euo pipefail

TERMLINK="${TERMLINK:-termlink}"

die() { echo "relay-hop-check: $*" >&2; exit 2; }

topic="" cid=""
while [ $# -gt 0 ]; do
    case "$1" in
        --topic)           topic="${2:-}"; shift 2 ;;
        --conversation-id) cid="${2:-}"; shift 2 ;;
        --cid)             cid="${2:-}"; shift 2 ;;
        -h|--help)
            grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "unknown arg: $1 (try --help)" ;;
    esac
done

[ -n "$topic" ] || die "missing --topic"
[ -n "$cid" ]   || die "missing --conversation-id/--cid"

cap="${TERMLINK_RELAY_MAX_HOPS:-4}"
[[ "$cap" =~ ^[0-9]+$ ]] || die "TERMLINK_RELAY_MAX_HOPS must be a non-negative integer (got '$cap')"

# Fetch the turns on this conversation (test seam or live hub).
if [ -n "${TERMLINK_RELAY_HOPCHECK_TEST_JSON:-}" ]; then
    raw="$(cat "$TERMLINK_RELAY_HOPCHECK_TEST_JSON" 2>/dev/null || true)"
else
    raw="$("$TERMLINK" channel subscribe "$topic" --conversation-id "$cid" \
              --cursor 0 --limit 1000 --json 2>/dev/null || true)"
fi

# Extract relay_hops from the LATEST turn (highest offset, msg_type turn). jq -s
# slurps NDJSON or an array uniformly. Absent metadata.relay_hops → 0 (a thread
# with no relay counter is treated as hop 0 = fresh, always continue).
hops="$(printf '%s' "$raw" | jq -s '
    [ .[] | if type=="array" then .[] else . end ]
    | map(select((.msg_type // "") == "turn"))
    | (sort_by(.offset) | last) // {}
    | (.metadata.relay_hops // 0) | tonumber? // 0
' 2>/dev/null || echo 0)"
[[ "$hops" =~ ^[0-9]+$ ]] || hops=0

if [ "$hops" -ge "$cap" ]; then
    echo "verdict=stop hops=$hops cap=$cap reason=hop-budget-exhausted"
    exit 10
fi

echo "verdict=continue hops=$hops cap=$cap next_hops=$(( hops + 1 ))"
exit 0
