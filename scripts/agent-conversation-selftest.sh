#!/usr/bin/env bash
# T-1829 — loopback doorbell+mail validation.
#
# Closes the pre-flight gap for adopting the doorbell+mail runtime (T-1800
# arc). Operators validating a fresh host don't yet have a live peer running
# `/check-arc respond` — and they won't turn loose real autonomous traffic
# until they've confirmed the loop is healthy locally. This verb produces a
# synthetic but real round-trip on an ephemeral topic, then asserts on the
# observable state via agent-conversation-status.sh (T-1826).
#
# What it does NOT validate: PTY inject (the doorbell wake itself). That
# requires a live peer session. This selftest covers the half that's
# self-contained — channel post + receipt + diagnostic read.
#
# Composes:
#   - channel create  (ephemeral topic)
#   - channel post --msg-type turn    (sender side)
#   - channel post --msg-type receipt (receiver side, same identity)
#   - agent-conversation-status.sh    (verify via the read-side primitive)
#
# Exit codes:
#   0 = pass (turn delivered, receipt watermark covers all turns)
#   1 = assertion-fail (turn missing, receipt missing, or pending > 0)
#   2 = usage error
#   3 = setup-fail (channel create / post / status verb failed)
set -euo pipefail

TERMLINK="${TERMLINK_BIN:-termlink}"
STATUS_VERB="${STATUS_VERB:-scripts/agent-conversation-status.sh}"

die() { echo "agent-conversation-selftest: $*" >&2; exit 2; }

usage() {
    cat <<'EOF'
Usage: agent-conversation-selftest.sh [--hub <addr>] [--verbose] [--json]

A self-contained loopback validation of the doorbell+mail runtime
(T-1800 arc). Creates an ephemeral topic, posts a synthetic turn + receipt
under one conversation_id, then verifies via agent-conversation-status.sh
that the resulting state shows DELIVERED (pending_count = 0).

Optional:
  --hub <addr>   target hub (default: local)
  --verbose      print each step with timing
  --json         emit machine-readable JSON envelope instead of text

Exit:
  0 pass | 1 assertion-fail | 2 usage | 3 setup-fail (hub unreachable)
EOF
}

hub="" verbose=0 json=0

while [ $# -gt 0 ]; do
    case "$1" in
        --hub)      hub="${2:-}"; shift 2 ;;
        --verbose)  verbose=1; shift ;;
        --json)     json=1; shift ;;
        -h|--help)  usage; exit 0 ;;
        *)          die "unknown arg: $1 (try --help)" ;;
    esac
done

# Identifiers — unique per-run.
topic="agent-conv-selftest-$$-$(date +%s)-${RANDOM}"
cid="cid-selftest-$$-$(date +%s)"
verdict="pass"
turns_posted=0
receipts_posted=0
status_result="{}"

start_ms="$(date +%s%3N)"

vlog() {
    [ "$verbose" -eq 1 ] && echo "[selftest] $*"
    return 0
}

emit_result() {
    local end_ms; end_ms="$(date +%s%3N)"
    local elapsed=$((end_ms - start_ms))
    if [ "$json" -eq 1 ]; then
        jq -n -c \
            --arg hub "${hub:-local}" \
            --arg topic "$topic" \
            --arg cid "$cid" \
            --argjson turns "$turns_posted" \
            --argjson receipts "$receipts_posted" \
            --argjson elapsed "$elapsed" \
            --argjson status "$status_result" \
            --arg verdict "$verdict" \
            '{
                ok: ($verdict == "pass"),
                hub: $hub,
                ephemeral_topic: $topic,
                conversation_id: $cid,
                turns_posted: $turns,
                receipts_posted: $receipts,
                status_result: $status,
                elapsed_ms: $elapsed,
                verdict: $verdict
            }'
    else
        printf 'verdict:         %s\n' "$verdict"
        printf 'hub:             %s\n' "${hub:-local}"
        printf 'ephemeral_topic: %s\n' "$topic"
        printf 'conversation_id: %s\n' "$cid"
        printf 'turns_posted:    %s\n' "$turns_posted"
        printf 'receipts_posted: %s\n' "$receipts_posted"
        printf 'elapsed_ms:      %s\n' "$elapsed"
    fi
}

# Common hub args.
hub_args=()
[ -n "$hub" ] && hub_args+=(--hub "$hub")

# --- Step 1: create ephemeral topic ---
vlog "step 1: create topic $topic"
if ! "$TERMLINK" channel create "${hub_args[@]}" "$topic" --retention messages:10 >/dev/null 2>&1; then
    verdict="setup-fail"
    emit_result
    exit 3
fi

# --- Step 2: post synthetic turn ---
vlog "step 2: post turn (cid=$cid)"
if ! "$TERMLINK" channel post "${hub_args[@]}" "$topic" \
        --msg-type turn --payload "selftest-turn" \
        --metadata conversation_id="$cid" >/dev/null 2>&1; then
    verdict="setup-fail"
    emit_result
    exit 3
fi
turns_posted=1

# --- Step 3: post synthetic receipt acking that turn (up_to=0, the turn's offset) ---
vlog "step 3: post receipt (up_to=0)"
if ! "$TERMLINK" channel post "${hub_args[@]}" "$topic" \
        --msg-type receipt --payload "selftest-ack" \
        --metadata conversation_id="$cid" \
        --metadata up_to=0 >/dev/null 2>&1; then
    verdict="setup-fail"
    emit_result
    exit 3
fi
receipts_posted=1

# --- Step 4: read state via the read-side primitive ---
vlog "step 4: query status verb"
status_args=(--topic "$topic" --conversation-id "$cid" --json)
[ -n "$hub" ] && status_args+=(--hub "$hub")

if ! status_result="$(bash "$STATUS_VERB" "${status_args[@]}" 2>/dev/null)"; then
    verdict="setup-fail"
    emit_result
    exit 3
fi

# --- Step 5: assert on observable state ---
vlog "step 5: assert turn>=1, receipt>=1, pending==0"
tc="$(printf '%s' "$status_result" | jq -r '.summary.turn_count // 0')"
rc="$(printf '%s' "$status_result" | jq -r '.summary.receipt_count // 0')"
pc="$(printf '%s' "$status_result" | jq -r '.summary.pending_count // -1')"

if [ "$tc" -ge 1 ] && [ "$rc" -ge 1 ] && [ "$pc" = "0" ]; then
    verdict="pass"
    emit_result
    exit 0
else
    verdict="assertion-fail"
    if [ "$verbose" -eq 1 ] || [ "$json" -eq 0 ]; then
        echo "[selftest] FAIL — turn_count=$tc receipt_count=$rc pending_count=$pc" >&2
    fi
    emit_result
    exit 1
fi
