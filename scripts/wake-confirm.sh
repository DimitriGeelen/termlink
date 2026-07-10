#!/usr/bin/env bash
# wake-confirm.sh — T-2396 (G-083 / PL-253): standalone consumption-confirmation
# for a wake. Answers the one question the doorbell never did on its own:
# "was my rung message actually CONSUMED, or is it sitting unread?"
#
# WHY THIS EXISTS. agent-send.sh already fails loud (exit 3) when a recipient
# never acks — but ONLY because it owns the ring+await-receipt loop. The paths
# people actually reach for in the field BYPASS that loop and therefore fail
# SILENTLY (proven live, T-2396):
#   - a raw `termlink inject <session> <text> --enter` (the "delivered by PTY"
#     path) types into the PTY and returns; it never waits for a receipt. If the
#     session is busy or in manual-accept mode the text lands UNSUBMITTED and is
#     discarded on the next `claude --continue` — message durably written, never
#     read.
#   - a plain thread post (e.g. a proposal on a `T-175` thread) has no doorbell
#     and no await at all.
# This verb extracts the receipt-wait so ANY delivery path can get the loud
# rung-but-not-consumed signal: ring the PTY (or post the thread turn), then run
#   wake-confirm.sh --topic <rail> --cid <cid> --since-offset <posted-offset>
#
# CONSUMED  = a receipt acking up_to >= since_offset appeared within --timeout.
#             A receipt is what `/check-arc respond` (agent-respond.sh) posts, so
#             CONSUMED means the recipient genuinely read + processed the turn.
# NOT-CONSUMED = no such receipt within the window → LOUD diagnosis + remedy.
#
# Exit: 0 CONSUMED | 3 NOT-CONSUMED (mirrors agent-send's exit 3) | 2 usage error
#
# Hub-independent test seam (PL-213): TERMLINK_WAKECONFIRM_TEST_JSON=<file> feeds
# canned `channel subscribe --json` output (NDJSON or a JSON array) and skips the
# live hub + the poll loop (single evaluation).
set -euo pipefail

TERMLINK="${TERMLINK:-termlink}"

die() { echo "wake-confirm: $*" >&2; exit 2; }

topic="" cid="" since_offset="" timeout=10 hub_addr="" json=0
while [ $# -gt 0 ]; do
    case "$1" in
        --topic)           topic="${2:-}"; shift 2 ;;
        --conversation-id) cid="${2:-}"; shift 2 ;;
        --cid)             cid="${2:-}"; shift 2 ;;
        --since-offset)    since_offset="${2:-}"; shift 2 ;;
        --timeout)         timeout="${2:-}"; shift 2 ;;
        --hub)             hub_addr="${2:-}"; shift 2 ;;
        --json)            json=1; shift ;;
        -h|--help)         grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "unknown arg: $1 (try --help)" ;;
    esac
done

[ -n "$topic" ]        || die "missing --topic"
[ -n "$cid" ]          || die "missing --conversation-id/--cid"
[ -n "$since_offset" ] || die "missing --since-offset (the offset of the turn you posted)"
[[ "$since_offset" =~ ^[0-9]+$ ]] || die "--since-offset must be a non-negative integer (got '$since_offset')"
[[ "$timeout" =~ ^[0-9]+$ ]]      || die "--timeout must be a non-negative integer (got '$timeout')"

hub_args=()
[ -n "$hub_addr" ] && hub_args=(--hub "$hub_addr")

# THE canonical receipt selector (identical to agent-send.sh's inline wait,
# lines ~533-538): a receipt on this cid whose up_to acks the turn we posted.
# A stale receipt from an earlier turn (up_to < since_offset) does NOT count —
# that was the T-1808 multi-turn false-DELIVERED bug.
receipt_from_json() { # stdin: subscribe json ; stdout: receipt json or empty
    jq -c -s --argjson po "$since_offset" '
        [ .[] | if type=="array" then .[] else . end ]
        | map(select((.msg_type // "") == "receipt"
                     and ((.metadata.up_to | tonumber? // -1) >= $po)))
        | (.[0] // empty)' 2>/dev/null || true
}

receipt_json=""
if [ -n "${TERMLINK_WAKECONFIRM_TEST_JSON:-}" ]; then
    # Seam: single evaluation, no polling.
    receipt_json="$(cat "$TERMLINK_WAKECONFIRM_TEST_JSON" 2>/dev/null | receipt_from_json)"
else
    waited=0
    while (( waited < timeout )); do
        receipt_json="$( { "$TERMLINK" channel subscribe "$topic" --conversation-id "$cid" \
                             --cursor 0 --limit 1000 --json "${hub_args[@]+"${hub_args[@]}"}" 2>/dev/null \
                          | receipt_from_json ; } || true )"
        [ -n "$receipt_json" ] && [ "$receipt_json" != "null" ] && break
        receipt_json=""
        sleep 1; waited=$((waited+1))
    done
fi

if [ -n "$receipt_json" ] && [ "$receipt_json" != "null" ]; then
    off="$(printf '%s' "$receipt_json" | jq -r '.offset // empty')"
    stage="$(printf '%s' "$receipt_json" | jq -r '.metadata.stage // empty')"
    [ "$stage" = "null" ] && stage=""
    if [ "$json" -eq 1 ]; then
        jq -cn --arg cid "$cid" --argjson off "${off:-null}" --arg stage "$stage" \
            '{consumed:true, cid:$cid, receipt_offset:$off, stage:(if $stage=="" then null else $stage end)}'
    else
        echo "wake-confirm: CONSUMED${stage:+ (stage=$stage)} — receipt for cid=$cid at offset=${off:-?}"
    fi
    exit 0
fi

# NOT CONSUMED — the loud rung-but-not-consumed signal G-083 was blind to.
if [ "$json" -eq 1 ]; then
    jq -cn --arg cid "$cid" --argjson so "$since_offset" --argjson t "$timeout" \
        '{consumed:false, cid:$cid, since_offset:$so, timeout_secs:$t,
          reason:"rung-but-not-consumed",
          diagnosis:"recipient rung but did not read within window; session likely busy or in manual-accept mode; message unread"}'
else
    echo "wake-confirm: NOT CONSUMED — no receipt for cid=$cid within ${timeout}s." >&2
    echo "  The recipient was rung but did NOT read your message (unread at offset $since_offset)." >&2
    echo "  Most likely: the session is busy on its own work, or in manual-accept mode, so the" >&2
    echo "  injected wake landed UNSUBMITTED and was discarded. Remedies:" >&2
    echo "    - recipient runs '/check-arc respond' to consume + ack, OR" >&2
    echo "    - relaunch it via 'tl-claude.sh start --reachable' (auto-accept injectable PTY), OR" >&2
    echo "    - a human advances that session to read the thread." >&2
fi
exit 3
