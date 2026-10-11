#!/usr/bin/env bash
# T-3384 — TOFU pin-event check (RR-18 interim control, operator ruling OQ-8 D″).
#
# Every TOFU pin change (`tofu clear`, `tofu clear --all`, and the `tofu-pin` that
# re-trusts a hub afterwards) is appended as one JSON line to
# ~/.termlink/pin-events.jsonl by termlink-session's KnownHubStore. This check reports
# events newer than the last ACKNOWLEDGED offset. Empty/none-new = healthy.
#
# LIMITS (RR-18): after-the-fact detection only. A same-user attacker can edit or
# truncate the record; a truncated file reads as "ack offset beyond end" and fires.
# It stays until re-pinning is itself a Tier 0 event (CR-18).
#
# Usage: check-pin-events.sh [--json] [--quiet] [--ack]
# Exit: 0 none new · 1 new events since ack · 2 tooling error
# Seams: TERMLINK_PIN_EVENTS_FILE (trail), PIN_EVENTS_SEEN_FILE (ack offset marker).
# guard-layer: source
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
EVENTS="${TERMLINK_PIN_EVENTS_FILE:-${TERMLINK_IDENTITY_DIR:-$HOME/.termlink}/pin-events.jsonl}"
SEEN="${PIN_EVENTS_SEEN_FILE:-$ROOT/.context/working/.pin-events.seen-offset}"
json=0 quiet=0 ack=0
while [ $# -gt 0 ]; do
    case "$1" in
        --json) json=1 ;; --quiet) quiet=1 ;; --ack) ack=1 ;;
        -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
        *) echo "check-pin-events: unknown arg $1" >&2; exit 2 ;;
    esac
    shift
done
command -v jq >/dev/null 2>&1 || { echo "check-pin-events: jq not found" >&2; exit 2; }

total=0
if [ -e "$EVENTS" ]; then
    [ -r "$EVENTS" ] || { echo "check-pin-events: cannot read $EVENTS" >&2; exit 2; }
    total="$(wc -l < "$EVENTS" | tr -dc '0-9')"; total="${total:-0}"
fi
seen=0
if [ -e "$SEEN" ]; then
    seen="$(tr -dc '0-9' < "$SEEN")"; seen="${seen:-0}"
fi
truncated=0
if [ "$seen" -gt "$total" ]; then truncated=1; seen=0; fi   # record shrank: treat all as new

new='[]'
if [ "$total" -gt "$seen" ]; then
    new="$(tail -n +$((seen + 1)) "$EVENTS" | head -n $((total - seen)) | jq -cs '.' 2>/dev/null)" \
        || { echo "check-pin-events: unparseable line in $EVENTS" >&2; exit 2; }
fi
n="$(printf '%s' "$new" | jq 'length')"

if [ "$ack" -eq 1 ]; then
    mkdir -p "$(dirname "$SEEN")" 2>/dev/null
    printf '%s\n' "$total" > "$SEEN" || { echo "check-pin-events: cannot write $SEEN" >&2; exit 2; }
fi

if [ "$json" -eq 1 ]; then
    jq -cn --argjson e "$new" --argjson t "$total" --argjson s "$seen" --argjson tr "$truncated" \
        '{ok: ($e|length==0), new_count: ($e|length), total: $t, acked_offset: $s, truncated: ($tr==1), events: $e,
          scope: "after-the-fact detection; same-user attacker can edit the record (RR-18)"}'
elif [ "$n" -gt 0 ]; then
    echo "TOFU pin events not yet acknowledged: $n"
    [ "$truncated" -eq 1 ] && echo "  NOTE: trail is shorter than the acknowledged offset — it was truncated or replaced"
    printf '%s' "$new" | jq -r '.[] | "  \(.ts)  \(.verb)  \(.address)  \(.old_fingerprint // "-") -> \(.new_fingerprint // "-")  by \(.agent_id // .argv0 // "?") pid=\(.pid)"'
    echo "  Review, then: bash scripts/check-pin-events.sh --ack"
elif [ "$quiet" -eq 0 ]; then
    echo "No new TOFU pin events ($total recorded, all acknowledged). Detection is after-the-fact only (RR-18)."
fi
[ "$ack" -eq 1 ] && exit 0
[ "$n" -gt 0 ] && exit 1
exit 0
