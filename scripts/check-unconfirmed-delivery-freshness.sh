#!/usr/bin/env bash
# T-2295 (arc-003 reliable-comms, V3b) — unconfirmed-delivery canary.
#
# RC3b made delivery-confirmation observable: `channel post --await-ack` writes a
# durable obligation row to ~/.termlink/awaiting_ack.sqlite, and `channel
# awaiting-ack` (T-2287) surfaces every post still waiting for a recipient ack —
# INCLUDING rows retained after their retry loop was exhausted. Those exhausted
# rows are the "sent-but-never-confirmed" class: the exact failure G-063 named
# (framework:pickup at 36-sent / 0-received — a write-only sink nobody noticed).
#
# Nothing surfaces them on its own. This canary does: it FIRES (exit 1) when any
# awaiting-ack row has been outstanding longer than --threshold-secs, turning a
# silent lost handoff into a daily alert. Empty/healthy = exit 0. Mirror of the
# mirror-drift / substrate-preflight / frozen-husk canaries (CLAUDE.md): empty
# log = healthy.
#
# Exit codes: 0 healthy (no stale unconfirmed sends) · 1 firing · 2 tooling error
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"

QUIET=0
FORMAT=human
THRESHOLD_SECS=600          # 10 min: a send unacked this long is a stuck delivery
TRACKER_PATH=""
HEARTBEAT=1
HEARTBEAT_FILE=".context/working/.unconfirmed-delivery-canary.heartbeat"

usage() {
    sed -n '2,18p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: check-unconfirmed-delivery-freshness.sh [OPTIONS]
  --threshold-secs N   Age beyond which an awaiting-ack row fires (default 600)
  --tracker-path P     awaiting_ack.sqlite path (default: termlink's own default)
  --json               Emit a JSON envelope
  --quiet              Print only on firing (cron-friendly)
  --no-heartbeat       Skip touching the heartbeat companion
  -h, --help           This help

Test hook: TERMLINK_UNCONFIRMED_TEST_JSON=<file> feeds canned `channel
awaiting-ack --json` output for hub-independent testing.

Exit: 0 healthy · 1 firing (stale unconfirmed sends) · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --threshold-secs) THRESHOLD_SECS="${2:-}"; shift 2 ;;
        --tracker-path)   TRACKER_PATH="${2:-}"; shift 2 ;;
        --json)           FORMAT=json; shift ;;
        --quiet)          QUIET=1; shift ;;
        --no-heartbeat)   HEARTBEAT=0; shift ;;
        -h|--help)        usage; exit 0 ;;
        *) echo "check-unconfirmed-delivery: unknown arg: $1" >&2; exit 2 ;;
    esac
done

case "$THRESHOLD_SECS" in
    ''|*[!0-9]*) echo "check-unconfirmed-delivery: --threshold-secs must be a positive integer" >&2; exit 2 ;;
esac

# Heartbeat FIRST (before any check) so /canaries can prove the canary ran even
# on a healthy cycle — mirrors the T-2290 convention.
if [ "$HEARTBEAT" -eq 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null && date -u +%Y-%m-%dT%H:%M:%SZ > "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# Fetch awaiting-ack rows (test hook short-circuits the CLI for hub-independence).
if [ -n "${TERMLINK_UNCONFIRMED_TEST_JSON:-}" ]; then
    raw="$(cat "${TERMLINK_UNCONFIRMED_TEST_JSON}" 2>/dev/null)"
    rc=$?
else
    args=(channel awaiting-ack --json)
    [ -n "$TRACKER_PATH" ] && args+=(--tracker-path "$TRACKER_PATH")
    raw="$("$TERMLINK" "${args[@]}" 2>/dev/null)"
    rc=$?
fi
if [ "$rc" -ne 0 ] || [ -z "$raw" ]; then
    echo "check-unconfirmed-delivery: could not read awaiting-ack tracker (exit=$rc)" >&2
    exit 2
fi

now_ms=$(( $(date +%s) * 1000 ))

# Compute stale rows: enqueued_ms older than threshold. jq emits one TSV line per
# stale row: topic\trecipient\tage_secs\tattempts.
stale="$(printf '%s' "$raw" | jq -r --argjson now "$now_ms" --argjson thr "$THRESHOLD_SECS" '
    (.rows // [])[]
    | ((($now - .enqueued_ms) / 1000) | floor) as $age
    | select($age > $thr)
    | [.dm_topic, .recipient_sender_id, ($age|tostring), (.attempts|tostring)] | @tsv
' 2>/dev/null)"
jq_rc=$?
if [ "$jq_rc" -ne 0 ]; then
    echo "check-unconfirmed-delivery: malformed awaiting-ack JSON" >&2
    exit 2
fi

total_pending="$(printf '%s' "$raw" | jq -r '.pending // ((.rows // []) | length)' 2>/dev/null)"
stale_count=0
[ -n "$stale" ] && stale_count="$(printf '%s\n' "$stale" | grep -c .)"

if [ "$FORMAT" = json ]; then
    rows_json="$(printf '%s' "$raw" | jq -c --argjson now "$now_ms" --argjson thr "$THRESHOLD_SECS" '
        [ (.rows // [])[]
          | ((($now - .enqueued_ms)/1000)|floor) as $age
          | select($age > $thr)
          | {dm_topic, recipient_sender_id, msg_offset, attempts, age_secs:$age} ]' 2>/dev/null)"
    printf '{"ok":%s,"pending":%s,"stale_count":%s,"threshold_secs":%s,"stale":%s}\n' \
        "$([ "$stale_count" -eq 0 ] && echo true || echo false)" \
        "${total_pending:-0}" "$stale_count" "$THRESHOLD_SECS" "${rows_json:-[]}"
    [ "$stale_count" -eq 0 ] && exit 0 || exit 1
fi

if [ "$stale_count" -eq 0 ]; then
    [ "$QUIET" -eq 1 ] || echo "check-unconfirmed-delivery: healthy (${total_pending:-0} pending, 0 stale > ${THRESHOLD_SECS}s)"
    exit 0
fi

echo "check-unconfirmed-delivery: $stale_count unconfirmed delivery(ies) stale > ${THRESHOLD_SECS}s (write-only-sink class, G-063):"
printf '%s\n' "$stale" | while IFS=$'\t' read -r topic recip age attempts; do
    echo "  $topic → $recip  age=${age}s  attempts=$attempts"
done
echo "  Remediation: the recipient never acked. Confirm the peer is LIVE (/peers --all),"
echo "  re-send via /agent-handoff, or drop the stale obligation if the thread is dead."
exit 1
