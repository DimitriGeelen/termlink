#!/usr/bin/env bash
# T-1891 — /check-outbox: OUTBOUND complement of /check-arc.
#
# Answers "did peers read the DMs I sent them?" by walking dm:<self-fp>:*
# topics, comparing each topic's envelope count against the peer's receipt
# `up_to` offset. Surfaces topics where peer hasn't kept up.
#
# Read-only — never posts, never acks, never modifies KnownHubStore.
#
# Why this exists: /check-arc surfaces INBOUND unread (who's waiting for me
# to read them). /check-outbox surfaces OUTBOUND unread (whose mailbox am I
# filling without them reading). T-1457 surfaced the canonical case:
# .141 had 5 DMs from .107 in dm:6604a2af:d1993c2c with no receipts —
# operator had no way to detect this without manually inspecting each topic
# per hub. This skill closes the loop.
#
# Self-fp resolution chain (mirrors /check-arc skill, PL-195):
#   1. channel info agent-presence --json | jq .senders[0].sender_id
#   2. channel info agent-chat-arc --json | jq .senders[] | select(posts>0)
#   3. exit 2 with hint
#
# Exit codes:
#   0 — surveyed without error (may have zero results)
#   1 — at least one hub returned malformed data
#   2 — tooling / self-fp resolution failure
#
# Usage:
#   check-outbox.sh                              # local hub only
#   check-outbox.sh --fleet                      # walk ~/.termlink/hubs.toml
#   check-outbox.sh --hubs-file PATH             # custom hubs.toml
#   check-outbox.sh --include-self               # include dm:<self>:<self>
#   check-outbox.sh --with-presence              # T-1895: enrich each row with
#                                                # peer's LIVE/STALE/OFFLINE status
#                                                # via agent-listeners-fleet.sh
#   check-outbox.sh --json
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
HUBS_FILE="${HOME}/.termlink/hubs.toml"
PER_HUB_TIMEOUT=8
FORMAT=human
FLEET=0
INCLUDE_SELF=0
WITH_PRESENCE=0

die() {
    if [ "$FORMAT" = json ]; then
        printf '{"ok":false,"error":"%s"}\n' "$1"
    else
        echo "check-outbox: $1" >&2
    fi
    exit 2
}

usage() {
    sed -n '2,36p' "$0"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --fleet)         FLEET=1; shift ;;
        --hubs-file)     HUBS_FILE="${2:-}"; FLEET=1; shift 2 ;;
        --include-self)  INCLUDE_SELF=1; shift ;;
        --with-presence) WITH_PRESENCE=1; shift ;;
        --json)          FORMAT=json; shift ;;
        --timeout-secs)  PER_HUB_TIMEOUT="${2:-}"; shift 2 ;;
        -h|--help)       usage; exit 0 ;;
        *)               echo "unknown arg: $1 (try --help)" >&2; exit 2 ;;
    esac
done

command -v jq >/dev/null 2>&1 || die "jq not in PATH"
command -v "$TERMLINK" >/dev/null 2>&1 || die "termlink binary not in PATH (set TERMLINK_BIN)"

# Validate timeout is a positive integer.
case "$PER_HUB_TIMEOUT" in
    ''|*[!0-9]*) die "invalid --timeout-secs: $PER_HUB_TIMEOUT" ;;
esac

if command -v timeout >/dev/null 2>&1; then
    TIMEOUT_CMD="timeout $PER_HUB_TIMEOUT"
else
    TIMEOUT_CMD=""
fi

# Resolve self-fp on the local hub (canonical chain, PL-195).
resolve_self_fp() {
    local hub_arg="${1:-}"
    local hub_flag=""
    [ -n "$hub_arg" ] && hub_flag="--hub $hub_arg"
    local fp=""
    # Primary: agent-presence
    fp="$($TIMEOUT_CMD $TERMLINK channel info agent-presence $hub_flag --json 2>/dev/null \
        | jq -r '.senders[]? | select(.posts > 0) | .sender_id' 2>/dev/null \
        | head -1 || true)"
    if [ -z "$fp" ]; then
        # Fallback: agent-chat-arc
        fp="$($TIMEOUT_CMD $TERMLINK channel info agent-chat-arc $hub_flag --json 2>/dev/null \
            | jq -r '.senders[]? | select(.posts > 0) | .sender_id' 2>/dev/null \
            | head -1 || true)"
    fi
    printf '%s' "$fp"
}

# Build list of (hub_name, hub_addr) pairs. For fleet mode, walk hubs.toml
# with the same minimal TOML parsing as the sibling wrappers. For local
# mode, use a single sentinel "local".
declare -a hub_names=()
declare -a hub_addrs=()
if [ "$FLEET" -eq 1 ]; then
    [ -f "$HUBS_FILE" ] || die "hubs file not found: $HUBS_FILE"
    current_name=""
    while IFS= read -r raw_line || [ -n "$raw_line" ]; do
        line="${raw_line%$'\r'}"
        line="${line%%#*}"
        line="${line#"${line%%[![:space:]]*}"}"
        line="${line%"${line##*[![:space:]]}"}"
        [ -z "$line" ] && continue
        if [[ "$line" =~ ^\[hubs\.([A-Za-z0-9_.-]+)\][[:space:]]*$ ]]; then
            current_name="${BASH_REMATCH[1]}"
        elif [ -n "$current_name" ] && [[ "$line" =~ ^address[[:space:]]*=[[:space:]]*\"([^\"]+)\"[[:space:]]*$ ]]; then
            hub_names+=("$current_name")
            hub_addrs+=("${BASH_REMATCH[1]}")
            current_name=""
        fi
    done < "$HUBS_FILE"
    # T-1889 sibling: dedup by TLS fingerprint so two profiles → same hub
    # don't produce duplicate rows.
    declare -A _fp_seen=()
    declare -a _hub_names_dedup=()
    declare -a _hub_addrs_dedup=()
    for i in "${!hub_addrs[@]}"; do
        addr="${hub_addrs[$i]}"
        name="${hub_names[$i]}"
        fp_out="$($TIMEOUT_CMD $TERMLINK hub probe "$addr" --json 2>/dev/null || true)"
        fp="$(printf '%s' "$fp_out" | jq -r '.fingerprint // empty' 2>/dev/null || true)"
        if [ -n "$fp" ] && [ -n "${_fp_seen[$fp]:-}" ]; then
            continue
        fi
        [ -n "$fp" ] && _fp_seen[$fp]="$name"
        _hub_names_dedup+=("$name")
        _hub_addrs_dedup+=("$addr")
    done
    hub_names=("${_hub_names_dedup[@]}")
    hub_addrs=("${_hub_addrs_dedup[@]}")
else
    hub_names=("local")
    hub_addrs=("")
fi

[ "${#hub_names[@]}" -gt 0 ] || die "no hubs to scan"

# Resolve self-fp ONCE from the local hub. On a shared-host setup (PL-195),
# every hub sees this host as the same key — there is no per-hub fingerprint
# divergence to discover. Resolving once avoids the per-hub fallback timeout
# cascade (when a remote hub doesn't have agent-presence + agent-chat-arc
# is slow, the per-hub resolution burns ~16s per hub).
GLOBAL_SELF_FP="$(resolve_self_fp "")"
if [ -z "$GLOBAL_SELF_FP" ]; then
    die "could not resolve self-fp on local hub. Run /be-reachable first or post once via /broadcast-chat to establish identity."
fi

tmp_rows="$(mktemp -t check-outbox.XXXXXX)"
trap 'rm -f "$tmp_rows"' EXIT

hubs_scanned=0
hubs_failed=0
for i in "${!hub_names[@]}"; do
    name="${hub_names[$i]}"
    addr="${hub_addrs[$i]}"
    hub_flag=""
    if [ -n "$addr" ]; then
        # Fleet mode — use the profile name as --hub.
        hub_flag="--hub $name"
    fi
    # Local mode: hub_flag stays empty so the CLI talks to the local default.

    self_fp="$GLOBAL_SELF_FP"

    # List dm:* topics on this hub.
    dm_topics="$($TIMEOUT_CMD $TERMLINK channel list --prefix "dm:" $hub_flag --json 2>/dev/null \
        | jq -r '.channels[]?.name // .topics[]?.name // empty' 2>/dev/null || true)"
    if [ -z "$dm_topics" ]; then
        # Fallback: parse human output `  dm:foo:bar  [forever]`
        dm_topics="$($TIMEOUT_CMD $TERMLINK channel list --prefix "dm:" $hub_flag 2>/dev/null \
            | awk '/^[[:space:]]+dm:/ {print $1}' || true)"
    fi
    [ -z "$dm_topics" ] && continue
    hubs_scanned=$((hubs_scanned + 1))

    while IFS= read -r topic; do
        [ -n "$topic" ] || continue
        # topic looks like dm:A:B (possibly dm:A:B-suffix-suffix for legacy
        # forms, but we only care if self_fp appears in the first two
        # colon-separated parts after dm:).
        rest="${topic#dm:}"
        a="${rest%%:*}"
        b="${rest#*:}"
        # Strip any trailing suffix on b (legacy `dm:fp:fp-suffix-suffix`).
        b="${b%%[!a-zA-Z0-9]*}"
        if [ "$a" = "$self_fp" ]; then
            peer="$b"
        elif [ "$b" = "$self_fp" ]; then
            peer="$a"
        else
            continue   # this topic is not addressed to/from self
        fi
        if [ "$INCLUDE_SELF" -eq 0 ] && [ "$peer" = "$self_fp" ]; then
            continue   # skip dm:<self>:<self> by default
        fi

        # Fetch topic info.
        info="$($TIMEOUT_CMD $TERMLINK channel info "$topic" $hub_flag --json 2>/dev/null || true)"
        [ -n "$info" ] || continue
        count="$(printf '%s' "$info" | jq -r '.count // 0' 2>/dev/null || echo 0)"
        # Self must have actually posted to this topic.
        self_posts="$(printf '%s' "$info" \
            | jq -r --arg s "$self_fp" '.senders[]? | select(.sender_id == $s) | .posts' \
            2>/dev/null | head -1)"
        [ -n "$self_posts" ] && [ "$self_posts" != "0" ] || continue

        # Peer's max ack offset (defaults to -1 if peer never acked).
        peer_acked="$(printf '%s' "$info" \
            | jq -r --arg p "$peer" '[.receipts[]? | select(.sender_id == $p) | .up_to] | max // -1' \
            2>/dev/null)"
        [ -z "$peer_acked" ] && peer_acked=-1

        # outbound_unread = posts after the offset peer last acked, capped at
        # the number of posts authored by SELF in that range. Approximation
        # (without per-envelope scan): use (count-1 - peer_acked) as the
        # upper bound. In practice for dm:* topics with only two senders,
        # most posts are mine, and the approximation is close enough as a
        # backpressure signal.
        outbound_unread=$((count - 1 - peer_acked))
        [ "$outbound_unread" -gt 0 ] || continue

        row="$(jq -n -c \
            --arg hub "$name" \
            --arg topic "$topic" \
            --arg self "$self_fp" \
            --arg peer "$peer" \
            --argjson count "$count" \
            --argjson self_posts "$self_posts" \
            --argjson peer_acked "$peer_acked" \
            --argjson unread "$outbound_unread" \
            '{hub:$hub, topic:$topic, self_fp:$self, peer_fp:$peer, count:$count,
              self_posts:$self_posts, peer_acked:$peer_acked, outbound_unread:$unread}')"
        printf '%s\n' "$row" >> "$tmp_rows"
    done <<< "$dm_topics"
done

# T-1895 presence enrichment — refactored T-1897 to consume the shared
# scripts/peer-presence-lookup.sh helper (single source of truth for fp→status
# semantics, shared with /check-arc per PL-116 SEND+RECEIVE symmetry).
#
# The helper internally does ONE hubs.toml walk + ONE agent-listeners-fleet
# probe, builds fp → MULTI-HUB SET (handles broadcast fan-out), and resolves
# each fp by walking its set preferring LIVE > STALE > OFFLINE. This fixes
# the prior inline "first-seen wins" rule which mis-routed a peer-fp posting
# on multiple hubs to whichever hub appeared first in hubs.toml regardless
# of where their LIVE listener actually is.
#
# Failure-tolerant: if the helper returns empty / non-zero, all rows render
# with peer_status=UNKNOWN and a stderr diagnostic appears.
presence_err=""
if [ "$WITH_PRESENCE" -eq 1 ]; then
    _peer_fps="$(jq -r '.peer_fp' "$tmp_rows" 2>/dev/null | sort -u)"
    declare -A _peer_fp_to_status=()
    if [ -n "$_peer_fps" ]; then
        _lookup_out="$(printf '%s' "$_peer_fps" | timeout 90 bash "$(dirname "$0")/peer-presence-lookup.sh" --hubs-file "$HUBS_FILE" 2>/dev/null || true)"
        if [ -z "$_lookup_out" ]; then
            presence_err="peer-presence-lookup returned no data"
        else
            while IFS=$'\t' read -r _fp _status; do
                [ -n "$_fp" ] || continue
                _peer_fp_to_status["$_fp"]="$_status"
            done <<< "$_lookup_out"
        fi
    fi

    _tmp_enriched="$(mktemp -t check-outbox-enriched.XXXXXX)"
    while IFS= read -r _row; do
        [ -n "$_row" ] || continue
        _peer_fp="$(printf '%s' "$_row" | jq -r '.peer_fp')"
        _peer_status="${_peer_fp_to_status[$_peer_fp]:-UNKNOWN}"
        printf '%s' "$_row" | jq -c --arg s "$_peer_status" '. + {peer_status: $s}' >> "$_tmp_enriched"
    done < "$tmp_rows"
    mv "$_tmp_enriched" "$tmp_rows"

    [ -n "$presence_err" ] && echo "check-outbox: presence lookup partial: $presence_err (rows shown with peer_status=UNKNOWN)" >&2
fi

# Render.
rows_arr="$(jq -s -c 'sort_by(-.outbound_unread)' "$tmp_rows" 2>/dev/null || echo '[]')"
n_rows="$(printf '%s' "$rows_arr" | jq 'length' 2>/dev/null || echo 0)"

if [ "$FORMAT" = json ]; then
    jq -n -c \
        --argjson rows "$rows_arr" \
        --arg self "$GLOBAL_SELF_FP" \
        --argjson hs "$hubs_scanned" \
        --argjson hf "$hubs_failed" \
        --argjson nr "$n_rows" \
        '{ok:true, self_fp:$self, topics:$rows,
          summary:{hubs_scanned:$hs, hubs_failed:$hf, topics_with_unread:$nr}}'
else
    if [ -z "$GLOBAL_SELF_FP" ]; then
        echo "check-outbox: could not resolve self-fp on any hub. To establish identity, run /be-reachable first or post once via /broadcast-chat." >&2
        exit 2
    fi
    self_fp_short="${GLOBAL_SELF_FP:0:8}"
    if [ "$n_rows" -eq 0 ]; then
        if [ "$FLEET" -eq 1 ]; then
            echo "check-outbox: no outbound-unread DMs across $hubs_scanned hub(s) — all peers caught up (self=$self_fp_short…)"
        else
            echo "check-outbox: no outbound-unread DMs on local hub — all peers caught up (self=$self_fp_short…)"
        fi
    else
        echo "check-outbox: $n_rows topic(s) with unread peer (self=$self_fp_short…)"
        # T-1895: when --with-presence was set, append [LIVE]/[STALE]/[OFFLINE] marker
        # (UNKNOWN suppressed — no noise when presence lookup is unavailable).
        printf '%s\n' "$rows_arr" | jq -r '.[] |
            (.peer_status // "UNKNOWN") as $st |
            (if $st == "LIVE" then "  [LIVE]   "
             elif $st == "STALE" then "  [STALE]  "
             elif $st == "OFFLINE" then "  [OFFLINE]"
             else "           " end) as $marker |
            "\($marker) \(.hub | tostring | .[0:18]) \(.topic | tostring | .[0:48]) peer=\(.peer_fp | tostring | .[0:8])…  unread=\(.outbound_unread)  (count=\(.count), peer_acked=\(.peer_acked))"
        '
        echo
        echo "DMs you sent that peer hasn't acked. If peer is unreachable, consider:"
        echo "  • /agent-handoff <peer> T-XXX \"<follow-up>\"     # nudge"
        # T-1895: only suggest /peers when --with-presence is OFF (otherwise the
        # status is already inline). Always suggest broadcast for OFFLINE peers.
        if [ "$WITH_PRESENCE" -eq 0 ]; then
            echo "  • /peers --all                                    # check if peer is LIVE"
            echo "  • /check-outbox --with-presence                   # inline LIVE/STALE/OFFLINE markers"
        else
            _any_offline="$(printf '%s' "$rows_arr" | jq -r '[.[] | select(.peer_status == "OFFLINE" or .peer_status == "UNKNOWN")] | length' 2>/dev/null || echo 0)"
            if [ "$_any_offline" -gt 0 ]; then
                echo "  • /broadcast-chat \"<follow-up>\"                # peer has no LIVE listener — broadcast may be the only path"
            fi
        fi
    fi
    if [ "$hubs_failed" -gt 0 ]; then
        echo "  ($hubs_failed hub(s) failed — see stderr lines above)" >&2
    fi
fi

exit 0
