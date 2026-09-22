#!/usr/bin/env bash
# T-3070 — the SENDER's own ledger (arc-011 slice S6).
#
# WHAT IT IS FOR
# --------------
# Operator spec step 6: "the sender notes it in its own ledger". Today a sender has
# no local record of anything it has sent. The only evidence lives on the hub as
# receipts, so to answer "what am I still waiting on?" it must re-poll every topic
# it has ever used — and a sender that RESTARTS knows nothing at all. The outbox is
# effectively write-only from the sender's point of view, which is the G-063 class
# (a sink nobody reads) pointed at ourselves.
#
# WHAT IT RECORDS
# ---------------
# One row per message sent, carrying the rung it has reached:
#
#   sent       we posted it. Nothing more is claimed.
#   delivered  an L2 stage=delivered receipt was READ BACK from the hub
#   read       an L3 stage=read receipt was READ BACK, with valid evidence
#
# So "delivered but never read" becomes a state the sender can SEE, rather than an
# absence it has to notice.
#
# TWO RULES THAT ARE THE WHOLE INTEGRITY OF IT
# --------------------------------------------
# 1. IT NEVER INVENTS A RUNG. `sync` advances a rung only on a receipt actually
#    read back. No receipt leaves the rung exactly where it was. A ledger that
#    optimistically marks things delivered is strictly worse than no ledger: it
#    manufactures the false confidence the whole arc exists to remove.
#
# 2. A DISAVOWED RECEIPT DOES NOT COUNT. `stage=read` carrying
#    `evidence=wake-consumer` is IGNORED. That evidence kind was withdrawn
#    (T-3068) because a consumer noticing a flag has injected nothing, so those
#    receipts assert something false. They exist on live topics today; if this
#    ledger honoured them it would relaunder a retracted claim as fact.
#
# Exit: 0 ok · 1 stuck messages found (`status --stuck` only) · 2 tooling.
# FAIL-CLOSED: an unreadable or uncreatable ledger exits 2, never a clean
# "nothing stuck" — a ledger that could not be opened has not told you anything.
set -u

TERMLINK="${TERMLINK:-termlink}"
LEDGER="${NOTIFY_LEDGER_PATH:-$HOME/.termlink/ledger.sqlite}"
SELF_FP=""
QUIET=0
JSON=0

usage() {
    cat <<'USAGE'
Usage: notify-ledger.sh <record|sync|status> [options]

The sender's own record of what it sent and what came back. Answers "what am I
still waiting on?" without re-polling every topic, and survives a restart.

Verbs:
  record --topic T --offset N [--peer FP] [--summary TEXT]
        note that we sent something. Rung starts at 'sent'.
  sync [--topic T]
        read receipts back from the hub and advance rungs. Never advances
        without a receipt; never honours evidence=wake-consumer.
  status [--stuck] [--older-than SECS] [--topic T]
        show outstanding messages and their rung. --stuck exits 1 if any
        message has sat below 'read' longer than --older-than (default 600).

Options:
  --ledger PATH   ledger location (default ~/.termlink/ledger.sqlite)
  --self-fp FP    our own fingerprint, so our own receipts are not counted
  --json          machine-readable output
  --quiet         suppress progress chatter
  --help          this text

Exit: 0 ok · 1 stuck found (--stuck) · 2 tooling (fail-closed)
USAGE
}

VERB="${1:-}"; [ $# -gt 0 ] && shift
case "$VERB" in
    record|sync|status) : ;;
    --help|-h|"") usage; exit 0 ;;
    *) echo "notify-ledger: unknown verb '$VERB'" >&2; usage >&2; exit 2 ;;
esac

TOPIC="" OFFSET="" PEER="" SUMMARY="" STUCK=0 OLDER_THAN=600
while [ $# -gt 0 ]; do
    case "$1" in
        --topic)       TOPIC="${2:-}"; shift 2 ;;
        --offset)      OFFSET="${2:-}"; shift 2 ;;
        --peer)        PEER="${2:-}"; shift 2 ;;
        --summary)     SUMMARY="${2:-}"; shift 2 ;;
        --ledger)      LEDGER="${2:-}"; shift 2 ;;
        --self-fp)     SELF_FP="${2:-}"; shift 2 ;;
        --stuck)       STUCK=1; shift ;;
        --older-than)  OLDER_THAN="${2:-}"; shift 2 ;;
        --json)        JSON=1; shift ;;
        --quiet)       QUIET=1; shift ;;
        --help|-h)     usage; exit 0 ;;
        *) echo "notify-ledger: unknown argument: $1" >&2; exit 2 ;;
    esac
done

command -v sqlite3 >/dev/null 2>&1 || { echo "notify-ledger: sqlite3 not found" >&2; exit 2; }
log() { [ "$QUIET" -eq 1 ] || echo "notify-ledger: $*"; }

mkdir -p "$(dirname "$LEDGER")" 2>/dev/null || { echo "notify-ledger: cannot create $(dirname "$LEDGER")" >&2; exit 2; }
sqlite3 "$LEDGER" "CREATE TABLE IF NOT EXISTS outbox (
    topic     TEXT NOT NULL,
    offset    INTEGER NOT NULL,
    peer      TEXT NOT NULL DEFAULT '',
    summary   TEXT NOT NULL DEFAULT '',
    rung      TEXT NOT NULL DEFAULT 'sent',
    sent_ms   INTEGER NOT NULL DEFAULT 0,
    rung_ms   INTEGER NOT NULL DEFAULT 0,
    evidence  TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (topic, offset)
);" 2>/dev/null || { echo "notify-ledger: cannot open ledger $LEDGER" >&2; exit 2; }

now_ms() { date +%s%3N; }
q() { printf '%s' "${1//\'/\'\'}"; }

case "$VERB" in

record)
    [ -n "$TOPIC" ]  || { echo "notify-ledger: record needs --topic" >&2; exit 2; }
    [ -n "$OFFSET" ] || { echo "notify-ledger: record needs --offset (never invent one)" >&2; exit 2; }
    case "$OFFSET" in ''|*[!0-9]*) echo "notify-ledger: --offset must be numeric" >&2; exit 2 ;; esac
    n="$(now_ms)"
    sqlite3 "$LEDGER" "INSERT OR IGNORE INTO outbox (topic,offset,peer,summary,rung,sent_ms,rung_ms)
        VALUES ('$(q "$TOPIC")',$OFFSET,'$(q "$PEER")','$(q "$SUMMARY")','sent',$n,$n);" || exit 2
    log "recorded sent: $TOPIC #$OFFSET${PEER:+ -> $PEER}"
    ;;

sync)
    # Read receipts BACK FROM THE HUB and advance rungs. The only source of truth
    # for a rung is a receipt that actually exists.
    topics="$(sqlite3 "$LEDGER" "SELECT DISTINCT topic FROM outbox WHERE rung != 'read'${TOPIC:+ AND topic='$(q "$TOPIC")'};")"
    [ -n "$topics" ] || { log "nothing outstanding to sync"; exit 0; }
    advanced=0
    while IFS= read -r t; do
        [ -n "$t" ] || continue
        if [ -n "${LEDGER_TEST_RECEIPTS:-}" ]; then
            blob="$(cat "$LEDGER_TEST_RECEIPTS" 2>/dev/null)"
        else
            blob="$(timeout 30 "$TERMLINK" channel subscribe "$t" --cursor 0 --limit 1000 --json 2>/dev/null)"
        fi
        [ -n "$blob" ] || { log "no receipts readable for $t (rungs unchanged)"; continue; }

        # Highest up_to per rung, from receipts NOT sent by us, excluding the
        # withdrawn evidence kind. jq -s so a partial read cannot half-apply.
        l2="$(printf '%s\n' "$blob" | jq -rs --arg me "$SELF_FP" '
            [.[] | (.envelope//.) | select(.msg_type=="receipt")
             | select(($me=="") or (.sender_id != $me))
             | (.metadata//{}) | select(.stage=="delivered") | (.up_to|tonumber?)] | max // -1' 2>/dev/null)"
        l3="$(printf '%s\n' "$blob" | jq -rs --arg me "$SELF_FP" '
            [.[] | (.envelope//.) | select(.msg_type=="receipt")
             | select(($me=="") or (.sender_id != $me))
             | (.metadata//{}) | select(.stage=="read")
             | select((.evidence // "") != "wake-consumer")
             | (.up_to|tonumber?)] | max // -1' 2>/dev/null)"
        case "$l2" in ''|null) l2=-1 ;; esac
        case "$l3" in ''|null) l3=-1 ;; esac
        n="$(now_ms)"
        if [ "$l2" -ge 0 ] 2>/dev/null; then
            c="$(sqlite3 "$LEDGER" "SELECT changes() FROM (SELECT 1); UPDATE outbox SET rung='delivered', rung_ms=$n
                 WHERE topic='$(q "$t")' AND offset<=$l2 AND rung='sent'; SELECT changes();" | tail -1)"
            advanced=$((advanced + ${c:-0}))
        fi
        if [ "$l3" -ge 0 ] 2>/dev/null; then
            c="$(sqlite3 "$LEDGER" "UPDATE outbox SET rung='read', rung_ms=$n, evidence='idle-gated-inject'
                 WHERE topic='$(q "$t")' AND offset<=$l3 AND rung!='read'; SELECT changes();" | tail -1)"
            advanced=$((advanced + ${c:-0}))
        fi
    done <<EOF
$topics
EOF
    log "sync complete: $advanced rung advance(s)"
    ;;

status)
    case "$OLDER_THAN" in ''|*[!0-9]*) echo "notify-ledger: --older-than must be numeric" >&2; exit 2 ;; esac
    cutoff=$(( $(now_ms) - OLDER_THAN * 1000 ))
    where="rung != 'read'${TOPIC:+ AND topic='$(q "$TOPIC")'}"
    [ "$STUCK" -eq 1 ] && where="$where AND sent_ms < $cutoff"
    rows="$(sqlite3 -separator '|' "$LEDGER" "SELECT topic,offset,peer,rung,sent_ms,summary FROM outbox WHERE $where ORDER BY sent_ms ASC;")"
    total="$(sqlite3 "$LEDGER" "SELECT COUNT(*) FROM outbox;")"
    n_out="$(printf '%s' "$rows" | grep -c . 2>/dev/null)"; n_out="${n_out:-0}"

    if [ "$JSON" -eq 1 ]; then
        printf '{"ok":true,"total":%s,"outstanding":%s,"stuck_filter":%s,"older_than_secs":%s}\n' \
            "$total" "$n_out" "$STUCK" "$OLDER_THAN"
    else
        if [ "$n_out" -eq 0 ]; then
            if [ "$STUCK" -eq 1 ]; then
                echo "notify-ledger: nothing stuck — $total message(s) tracked, none below 'read' for more than ${OLDER_THAN}s"
            else
                echo "notify-ledger: nothing outstanding — all $total tracked message(s) reached 'read'"
            fi
        else
            printf '%-42s %6s %-10s %-10s %s\n' TOPIC OFFSET PEER RUNG SUMMARY
            while IFS='|' read -r t o p r sm su; do
                [ -n "$t" ] || continue
                printf '%-42s %6s %-10s %-10s %s\n' "${t:0:42}" "$o" "${p:0:10}" "$r" "${su:0:40}"
            done <<EOF2
$rows
EOF2
            echo
            echo "$n_out of $total tracked message(s) have not reached 'read'."
            echo "A rung here only ever moves on a receipt read back from the hub —"
            echo "absence of a receipt is shown as absence, never assumed to be progress."
        fi
    fi
    [ "$STUCK" -eq 1 ] && [ "$n_out" -gt 0 ] && exit 1
    ;;
esac
exit 0
