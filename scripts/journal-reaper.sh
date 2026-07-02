#!/usr/bin/env bash
# T-2302 (arc-003 reliable-comms, V6 slice S5) — journal-authoritative firehose reaper.
#
# The LAST V6 slice. S1 (journal-mirror.sh) made the per-conversation SQLite journal
# a durable mirror of dm: turns; S5 makes it AUTHORITATIVE by trimming already-
# journaled dm: turn/receipt envelopes OFF the hub firehose. The firehose reverts to
# a bounded short-window store-and-forward buffer instead of an ever-growing log (the
# T-1991 agent-presence-bloat class, now applied to dm:).
#
# Approach — option (b), client-side (design recommendation, T-2296 §2-S5). Chosen
# over hub-side suppression (channel.rs routing) because it has a smaller blast radius,
# is reversible, and never touches the hot post path. Script-first (no Rust rebuild),
# mirroring the S1–S4 precedent.
#
# TRIM PRIMITIVE (build-subtlety #1, confirmed empirically on loopback):
#   `channel set-retention <topic> --retention messages:N` + `channel sweep <topic>`
#   keeps the newest N envelopes and prunes the oldest (total-N). `redact` is
#   append-only (keeps the original in the store) — NOT a trim. `sweep --json`
#   returns {"ok":true,"pruned":K}.
#
# MESSAGE-LOSS SAFETY (build-subtlety #2, load-bearing — two structural layers):
#   1. journal-mirror runs FIRST for each topic → after it, journal ⊇ firehose by
#      construction, so anything sweep drops is already journaled.
#   2. A GUARD then queries journal.sqlite and REFUSES to sweep a topic unless every
#      offset in the prune-range (the oldest total-WINDOW offsets) is present in the
#      journal. Trimming ahead of the journal is structurally impossible, not a
#      convention: no sweep is issued when the guard fails.
#   The WINDOW (newest N kept) is the store-and-forward buffer — an in-flight
#   direct/fallback send lands at the newest offset and is never in the prune-range.
#
# FEDERATION (build-subtlety #3, G-060 / [[reference_chatarc_dm_federation]]):
#   dm: topics federate with inbound sync lag. Trimming on one hub must not run ahead
#   of the inbound sync + journaling on the hubs that still need to pull the message.
#   This reaper operates on the LOCAL hub by default (or a single --hub); it trims only
#   offsets that are locally present AND locally journaled. Run the reaper on the hub
#   that owns/serves the topic. Multi-hub caveat: mirror on every hub before reaping
#   any one of them.
#
# SCOPE: presence / broadcast / agent-chat-arc / store-and-forward topics are UNTOUCHED
#   — only dm:* topics are enumerated, and --topic refuses a non-dm: name.
#
# Exit: 0 ok (including "nothing to trim") · 2 usage / tooling error
set -uo pipefail

TERMLINK="${TERMLINK_BIN:-termlink}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

JOURNAL="${TERMLINK_JOURNAL_PATH:-$HOME/.termlink/journals/journal.sqlite}"
HUB=""
ONE_TOPIC=""
WINDOW="${TERMLINK_REAP_WINDOW:-50}"   # store-and-forward window: newest N kept per topic
SUB_LIMIT=1000000                       # cap on offsets read per topic per pass
DRY_RUN=0
NO_MIRROR=0                             # skip the pre-sweep journal-mirror (LAYER 1)
FORMAT=human

die() { echo "journal-reaper: $*" >&2; exit 2; }

usage() {
    sed -n '2,38p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: journal-reaper.sh [OPTIONS]
  --hub ADDR           reap on this hub (default: local hub)
  --journal PATH       sqlite journal path (default: ~/.termlink/journals/journal.sqlite,
                       or $TERMLINK_JOURNAL_PATH)
  --topic T            reap only this topic (MUST be a dm: topic; refuses otherwise).
                       Default: every dm:* topic on the hub.
  --window N           store-and-forward window — newest N envelopes kept per topic
                       (default 50, or $TERMLINK_REAP_WINDOW). The prune-range is the
                       oldest (total-N); a topic with <= N envelopes is skipped.
  --dry-run            report what WOULD be pruned per topic; issue no set-retention/sweep
  --no-mirror          skip the pre-sweep journal-mirror (LAYER 1). Use when a cron/
                       sidecar already mirrors continuously. SAFE: the guard (LAYER 2)
                       still refuses any topic whose prune-range is not fully journaled,
                       so trimming ahead of the journal remains structurally impossible.
  --json               emit a JSON summary envelope
  -h, --help           this help

Exit: 0 ok (including "nothing to trim") · 2 usage / tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --hub)     HUB="${2:-}"; shift 2 ;;
        --journal) JOURNAL="${2:-}"; shift 2 ;;
        --topic)   ONE_TOPIC="${2:-}"; shift 2 ;;
        --window)  WINDOW="${2:-}"; shift 2 ;;
        --dry-run) DRY_RUN=1; shift ;;
        --no-mirror) NO_MIRROR=1; shift ;;
        --json)    FORMAT=json; shift ;;
        -h|--help) usage; exit 0 ;;
        *)         die "unknown arg: $1 (try --help)" ;;
    esac
done

command -v "$TERMLINK" >/dev/null 2>&1 || die "termlink not on PATH"
command -v jq >/dev/null 2>&1           || die "jq not available"
command -v sqlite3 >/dev/null 2>&1      || die "sqlite3 not available"
case "$WINDOW" in ''|*[!0-9]*) die "--window must be a non-negative integer" ;; esac

# SCOPE ENFORCEMENT (AC4): an explicit --topic must be a dm: topic. The enumerate-all
# path only ever lists dm:* topics, so non-dm topics can never be reaped.
if [ -n "$ONE_TOPIC" ] && [ "${ONE_TOPIC#dm:}" = "$ONE_TOPIC" ]; then
    die "refusing to reap non-dm topic '$ONE_TOPIC' — the reaper only trims dm:* topics (scope boundary, T-2302 AC4)"
fi

hub_args=()
[ -n "$HUB" ] && hub_args=(--hub "$HUB")

# Resolve topic list — dm:* only.
topics=""
if [ -n "$ONE_TOPIC" ]; then
    topics="$ONE_TOPIC"
else
    topics="$("$TERMLINK" channel list "${hub_args[@]+"${hub_args[@]}"}" --prefix "dm:" --json 2>/dev/null \
        | jq -r '(.topics // .)[]?.name // empty' 2>/dev/null)"
fi

# Journal offset lookup for a topic (single-quote-escaped for the SQL literal).
journal_offsets_for() {
    local t="$1" esc
    esc="${t//\'/\'\'}"
    sqlite3 "$JOURNAL" "SELECT offset FROM messages WHERE topic='$esc' ORDER BY offset;" 2>/dev/null
}

total_topics=0
total_pruned=0
total_skipped_small=0
total_skipped_unsafe=0
total_skipped_mirror=0
declare -a report_lines=()
declare -a json_topics=()

emit_report() {
    # $1=topic $2=verdict $3=detail
    report_lines+=("  $1  $2  $3")
    json_topics+=("$(jq -n -c --arg t "$1" --arg v "$2" --arg d "$3" \
        '{topic:$t, verdict:$v, detail:$d}')")
}

while IFS= read -r t; do
    [ -n "$t" ] || continue
    # Defense-in-depth: never process a non-dm topic even if the list somehow yields one.
    [ "${t#dm:}" = "$t" ] && continue
    total_topics=$((total_topics + 1))

    # LAYER 1: journal-mirror this topic FIRST so the journal is a superset of the
    # firehose before we consider trimming. Skip the topic loudly if it fails.
    # --no-mirror disables this convenience layer; LAYER 2 (the guard) still protects.
    if [ "$NO_MIRROR" -eq 0 ]; then
        if ! bash "$HERE/journal-mirror.sh" --topic "$t" --journal "$JOURNAL" \
                "${hub_args[@]+"${hub_args[@]}"}" >/dev/null 2>&1; then
            total_skipped_mirror=$((total_skipped_mirror + 1))
            emit_report "$t" "SKIP-MIRROR-FAIL" "journal-mirror failed — refusing to trim"
            continue
        fi
    fi

    # Current firehose offsets (sorted ascending). NDJSON: one envelope per line.
    offsets="$("$TERMLINK" channel subscribe "$t" "${hub_args[@]+"${hub_args[@]}"}" \
                  --cursor 0 --limit "$SUB_LIMIT" --json 2>/dev/null \
               | jq -r '.offset' 2>/dev/null | sort -n)"
    total="$(printf '%s\n' "$offsets" | grep -c '^[0-9]' || true)"
    case "$total" in ''|*[!0-9]*) total=0 ;; esac

    if [ "$total" -le "$WINDOW" ]; then
        total_skipped_small=$((total_skipped_small + 1))
        emit_report "$t" "SKIP-WINDOW" "total=$total <= window=$WINDOW (nothing to prune safely)"
        continue
    fi

    prune_count=$((total - WINDOW))
    prune_offsets="$(printf '%s\n' "$offsets" | grep '^[0-9]' | head -n "$prune_count")"

    # LAYER 2 (the structural guard): every prune-range offset MUST be in the journal.
    # Load journal offsets into a set; count prune offsets missing from it. A single
    # miss aborts the sweep for this topic — trimming ahead of the journal is impossible.
    declare -A jset=()
    while IFS= read -r jo; do [ -n "$jo" ] && jset["$jo"]=1; done < <(journal_offsets_for "$t")
    missing=0
    while IFS= read -r po; do
        [ -n "$po" ] || continue
        [ -z "${jset[$po]:-}" ] && missing=$((missing + 1))
    done <<EOF
$prune_offsets
EOF
    unset jset

    if [ "$missing" -gt 0 ]; then
        total_skipped_unsafe=$((total_skipped_unsafe + 1))
        emit_report "$t" "SKIP-UNSAFE" "$missing/$prune_count prune-range offset(s) not in journal — refusing to trim (message-loss safety)"
        continue
    fi

    if [ "$DRY_RUN" -eq 1 ]; then
        emit_report "$t" "DRY-RUN" "would prune $prune_count (keep newest $WINDOW of $total)"
        continue
    fi

    # Safe to trim: set the bounded policy and sweep NOW.
    "$TERMLINK" channel set-retention "$t" "${hub_args[@]+"${hub_args[@]}"}" \
        --retention "messages:$WINDOW" --json >/dev/null 2>&1
    pruned="$("$TERMLINK" channel sweep "$t" "${hub_args[@]+"${hub_args[@]}"}" --json 2>/dev/null \
              | jq -r '.pruned // 0' 2>/dev/null)"
    case "$pruned" in ''|*[!0-9]*) pruned=0 ;; esac
    total_pruned=$((total_pruned + pruned))
    emit_report "$t" "REAPED" "pruned=$pruned (kept newest $WINDOW of $total)"
done <<EOF
$topics
EOF

if [ "$FORMAT" = json ]; then
    topics_arr="$(printf '%s\n' "${json_topics[@]+"${json_topics[@]}"}" | jq -s -c '.' 2>/dev/null || echo '[]')"
    jq -n -c \
        --arg journal "$JOURNAL" \
        --argjson window "$WINDOW" \
        --argjson dry "$([ "$DRY_RUN" -eq 1 ] && echo true || echo false)" \
        --argjson scanned "$total_topics" \
        --argjson pruned "$total_pruned" \
        --argjson skip_small "$total_skipped_small" \
        --argjson skip_unsafe "$total_skipped_unsafe" \
        --argjson skip_mirror "$total_skipped_mirror" \
        --argjson topics "$topics_arr" \
        '{ok:true, journal:$journal, window:$window, dry_run:$dry,
          topics_scanned:$scanned, total_pruned:$pruned,
          skipped_window:$skip_small, skipped_unsafe:$skip_unsafe,
          skipped_mirror_fail:$skip_mirror, topics:$topics}'
else
    echo "journal-reaper: ${total_topics} dm topic(s) scanned, ${total_pruned} envelope(s) pruned, window=${WINDOW}$([ "$DRY_RUN" -eq 1 ] && echo ' (dry-run)')"
    [ "$total_skipped_small" -gt 0 ]  && echo "  ${total_skipped_small} topic(s) under window (skipped)"
    [ "$total_skipped_unsafe" -gt 0 ] && echo "  ${total_skipped_unsafe} topic(s) SKIPPED for message-loss safety (unjournaled offsets in prune range)"
    [ "$total_skipped_mirror" -gt 0 ] && echo "  ${total_skipped_mirror} topic(s) SKIPPED — journal-mirror failed"
    for ln in "${report_lines[@]+"${report_lines[@]}"}"; do echo "$ln"; done
fi
exit 0
