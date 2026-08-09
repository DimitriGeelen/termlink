#!/usr/bin/env bash
# T-2558 (T-2468 antifragility lens) — dead-letter canary.
#
# T-2243 (R4) stopped SILENT poison-drops on the charter-core "exchange durable
# messages" path: a `channel post` rejected POISON_THRESHOLD=10 times (unknown
# topic, bad signature, or a governance message rejected during a hub blip) is now
# durably MOVED to the offline queue's `dead_letters` table instead of being
# DELETE'd. That converts a silent data-loss into a durable, inspectable record —
# BUT the record is surfaced only if a human runs `/queue-status`. Nothing reads it
# on a schedule: the T-2083–2087 queue observability arc fires only on
# Drained/Pending transitions, never on dead-letter growth. So a poison-dropped
# GUARANTEED message sits forever with nothing firing — the "write-only sink nobody
# noticed" class (G-063) that the unconfirmed-delivery canary (T-2295) closed for
# the await-ack path. This canary closes it for the dead-letter path.
#
# `channel queue-status --json` already surfaces `dead_letters` (exact count) +
# `dead_letter_rows` (capped). This canary FIRES (exit 1) when dead_letters > 0.
# Empty log = healthy — the same convention as the other canaries (CLAUDE.md).
#
# Exit codes: 0 healthy (no dead letters) · 1 firing (>=1 dead letter) · 2 tooling error
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"

QUIET=0
FORMAT=human
HEARTBEAT=1
HEARTBEAT_FILE=".context/working/.dead-letter-canary.heartbeat"

usage() {
    sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: check-dead-letter-freshness.sh [OPTIONS]
  --json               Emit a JSON envelope
  --quiet              Print only on firing (cron-friendly)
  --no-heartbeat       Skip touching the heartbeat companion
  -h, --help           This help

Test hook: TERMLINK_DEAD_LETTER_TEST_JSON=<file> feeds canned `channel
queue-status --json` output for hub-independent testing.

Exit: 0 healthy · 1 firing (dead-lettered posts present) · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json)         FORMAT=json; shift ;;
        --quiet)        QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        -h|--help)      usage; exit 0 ;;
        *) echo "check-dead-letter: unknown arg: $1" >&2; exit 2 ;;
    esac
done

# Heartbeat FIRST (before the check) so /canaries can prove the canary ran even on
# a healthy cycle — mirrors the T-2290/T-2295/T-2556 convention.
if [ "$HEARTBEAT" -eq 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null && date -u +%Y-%m-%dT%H:%M:%SZ > "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# Fetch queue status (test hook short-circuits the CLI for hub-independence).
# queue-status is a pure local SQLite read — no hub round-trip — so it works even
# when the hub is down (which is exactly when the queue is absorbing writes).
if [ -n "${TERMLINK_DEAD_LETTER_TEST_JSON:-}" ]; then
    raw="$(cat "${TERMLINK_DEAD_LETTER_TEST_JSON}" 2>/dev/null)"
    rc=$?
else
    raw="$("$TERMLINK" channel queue-status --json 2>/dev/null)"
    rc=$?
fi
if [ "$rc" -ne 0 ] || [ -z "$raw" ]; then
    echo "check-dead-letter: could not read queue-status (exit=$rc)" >&2
    exit 2
fi

# dead_letters is absent when the queue file has never been created (exists:false)
# — treat as 0 (healthy). `// 0` in jq covers both the missing-field and null cases.
dead_count="$(printf '%s' "$raw" | jq -r '.dead_letters // 0' 2>/dev/null)"
jq_rc=$?
if [ "$jq_rc" -ne 0 ] || ! printf '%s' "$dead_count" | grep -qE '^[0-9]+$'; then
    echo "check-dead-letter: malformed queue-status JSON (no numeric dead_letters)" >&2
    exit 2
fi
pending="$(printf '%s' "$raw" | jq -r '.pending // 0' 2>/dev/null)"

# One TSV line per dead-letter row: topic\treason\tattempts.
dead_rows="$(printf '%s' "$raw" | jq -r '
    (.dead_letter_rows // [])[]
    | [.topic, (.reason // "?"), (.attempts|tostring)] | @tsv
' 2>/dev/null)"

if [ "$FORMAT" = json ]; then
    rows_json="$(printf '%s' "$raw" | jq -c '[(.dead_letter_rows // [])[]
        | {topic, reason, attempts, dead_lettered_ms}]' 2>/dev/null)"
    printf '{"ok":%s,"pending":%s,"dead_letters":%s,"rows":%s}\n' \
        "$([ "$dead_count" -eq 0 ] && echo true || echo false)" \
        "${pending:-0}" "$dead_count" "${rows_json:-[]}"
    [ "$dead_count" -eq 0 ] && exit 0 || exit 1
fi

if [ "$dead_count" -eq 0 ]; then
    [ "$QUIET" -eq 1 ] || echo "check-dead-letter: healthy (${pending:-0} pending, 0 dead-lettered)"
    exit 0
fi

echo "check-dead-letter: $dead_count dead-lettered post(s) — durable poison-drop, never delivered (G-063 class, T-2558):"
printf '%s\n' "$dead_rows" | while IFS=$'\t' read -r topic reason attempts; do
    [ -z "$topic" ] && continue
    echo "  $topic  reason=$reason  attempts=$attempts"
done
echo "  Remediation: a guaranteed post was rejected ${attempts:-10}x and moved to the"
echo "  dead_letters table. Inspect with \`/queue-status\` (full rows), fix the reject"
echo "  cause (unknown topic → create it; bad signature → \`fleet reauth\`), then re-send"
echo "  via \`/agent-handoff\`. Drop the row from ~/.termlink/outbound.sqlite if truly dead."
exit 1
