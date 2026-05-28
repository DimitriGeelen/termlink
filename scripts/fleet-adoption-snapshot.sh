#!/usr/bin/env bash
# T-1843 — Fleet doorbell+mail adoption snapshot.
#
# Complement to T-1831 fleet-doorbell-mail-canary. The canary measures
# health (loopback plumbing works); this verb measures *adoption* (real
# traffic happening). The two are distinct gauges — the canary returns
# PASS forever on a hub where nobody uses the rail, which is exactly the
# silent-failure mode this snapshot was built to surface.
#
# Per-hub it counts:
#   - live_listeners      via agent-listeners.sh (presence heartbeats)
#   - chat_arc_posts      windowed count on `agent-chat-arc` topic
#   - dm_topic_count      number of `dm:*:*` topics that exist with >0 posts
#
# Fleet rollup classifies adoption_state:
#   HOT   — ≥1 live listener AND ≥1 chat_arc post in the window
#   WARM  — ≥1 live listener but no chat_arc activity in window
#   COLD  — zero live listeners fleet-wide
#
# G-060 graceful: missing `agent-presence` / `agent-chat-arc` topic on a
# fresh hub → zeros for that hub, not a failure. (Same T-1842 pattern.)
#
# Read-only; never auths, never writes.
#
# Exit codes:
#   0  sweep completed (any adoption_state, including COLD)
#   2  usage error
#   3  setup-fail (hubs.toml missing, jq missing, listeners verb missing)
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
LISTENERS_VERB="${LISTENERS_VERB:-scripts/agent-listeners.sh}"
HUBS_FILE="${HUBS_FILE_OVERRIDE:-$HOME/.termlink/hubs.toml}"

FORMAT=human
WINDOW_HOURS=24

# Per-hub call timeout. Without this, `termlink channel info|subscribe`
# against an unreachable hub hangs indefinitely (no client-side timeout in
# the binary as of 0.11.323). Observed in the wild: ~50 zombie `channel
# info --hub laptop-141 agent-chat-arc` processes leaked over a week.
# 8s is enough for a healthy LAN hub round-trip, short enough that a dead
# hub does not block the sweep more than 8s × N_calls_per_hub.
PER_CALL_TIMEOUT="${FLEET_ADOPT_TIMEOUT:-8}"
if command -v timeout >/dev/null 2>&1; then
    TIMEOUT_CMD="timeout $PER_CALL_TIMEOUT"
else
    TIMEOUT_CMD=""
fi

die_usage() {
    echo "fleet-adoption-snapshot: $*" >&2
    echo "Try --help for usage." >&2
    exit 2
}

die_setup() {
    if [ "$FORMAT" = json ]; then
        printf '{"ok":false,"error":"%s"}\n' "$1"
    else
        echo "fleet-adoption-snapshot: $1" >&2
    fi
    exit 3
}

usage() {
    cat <<'EOF'
Usage: fleet-adoption-snapshot.sh [OPTIONS]

Measure REAL doorbell+mail adoption across the fleet. Distinct from the
T-1831 health canary (which only validates plumbing). This verb answers
"is anyone actually using the rail right now?".

Options:
  --since <hours>     Window for chat-arc post count (default 24, 1..=720)
  --hubs-file PATH    Override default ~/.termlink/hubs.toml
  --json              Emit JSON envelope (one object) instead of text
  -h, --help          Print this help and exit 0

Exit codes:
  0  sweep completed
  2  usage error
  3  setup-fail (hubs.toml missing, jq missing, listeners verb missing)

Adoption state:
  HOT   ≥1 live listener AND ≥1 chat_arc post in the window
  WARM  ≥1 live listener but no chat_arc activity
  COLD  zero live listeners fleet-wide
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --since)      WINDOW_HOURS="${2:-}"; shift 2 ;;
        --hubs-file)  HUBS_FILE="${2:-}"; shift 2 ;;
        --json)       FORMAT=json; shift ;;
        -h|--help)    usage; exit 0 ;;
        *)            die_usage "unknown arg: $1" ;;
    esac
done

case "$WINDOW_HOURS" in
    ''|*[!0-9]*) die_usage "--since must be a positive integer (hours)" ;;
esac
[ "$WINDOW_HOURS" -ge 1 ] || die_usage "--since must be >= 1"
[ "$WINDOW_HOURS" -le 720 ] || die_usage "--since must be <= 720"

command -v jq >/dev/null 2>&1 || die_setup "jq not in PATH"
[ -f "$HUBS_FILE" ] || die_setup "hubs file not found: $HUBS_FILE"
[ -x "$LISTENERS_VERB" ] || die_setup "listeners verb not executable: $LISTENERS_VERB"

# Parse hubs.toml — same minimal pattern as check-fleet-doorbell-mail-health.sh.
current_name=""
declare -a profile_names=()
declare -a profile_addrs=()

while IFS= read -r raw_line || [ -n "$raw_line" ]; do
    line="${raw_line%$'\r'}"
    line="${line%%#*}"
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line%"${line##*[![:space:]]}"}"
    [ -z "$line" ] && continue

    if [[ "$line" =~ ^\[hubs\.([A-Za-z0-9_.-]+)\][[:space:]]*$ ]]; then
        current_name="${BASH_REMATCH[1]}"
    elif [ -n "$current_name" ] && [[ "$line" =~ ^address[[:space:]]*=[[:space:]]*\"([^\"]+)\"[[:space:]]*$ ]]; then
        addr="${BASH_REMATCH[1]}"
        profile_names+=("$current_name")
        profile_addrs+=("$addr")
        current_name=""
    fi
done < "$HUBS_FILE"

total_hubs="${#profile_names[@]}"
[ "$total_hubs" -gt 0 ] || die_setup "no profiles found in $HUBS_FILE"

now_ms="$(date +%s%3N)"
window_ms=$((WINDOW_HOURS * 3600 * 1000))
since_ms=$((now_ms - window_ms))

tmp_results="$(mktemp -t fleet-adoption.XXXXXX)"
trap 'rm -f "$tmp_results"' EXIT

# Per-hub probe. Each call short-times out via the underlying termlink RPC;
# we don't add our own timeout — the binary handles network errors cleanly.
fleet_listeners=0
fleet_chat_arc=0
fleet_dm_topics=0
fleet_reachable_hubs=0
# T-1848: fleet-wide speaker UNION (not sum) — temp file accumulates one
# agent_id per line across all hubs; final count via sort -u | wc -l.
fleet_speakers_tmp="$(mktemp -t fleet-adoption-speakers.XXXXXX)"
trap 'rm -f "$tmp_results" "$fleet_speakers_tmp"' EXIT

for i in "${!profile_names[@]}"; do
    name="${profile_names[$i]}"
    addr="${profile_addrs[$i]}"

    verdict="ok"
    live_listeners=0
    chat_arc_posts=0
    dm_topic_count=0
    unique_speakers=0
    speakers_payload=""  # newline-delimited agent_ids from this hub's chat_arc scan, for fleet-wide union

    # --- live listeners via agent-listeners.sh ---
    listeners_json="$(bash "$LISTENERS_VERB" --hub "$addr" --json 2>/dev/null || echo '')"
    if [ -z "$listeners_json" ] || ! printf '%s' "$listeners_json" | jq -e . >/dev/null 2>&1; then
        verdict="unreachable"
    else
        live_listeners="$(printf '%s' "$listeners_json" | jq -r '.live // 0')"
    fi

    if [ "$verdict" = "ok" ]; then
        # --- agent-chat-arc windowed post count ---
        # Seek-to-tail pattern (PL-188, T-1844): probe channel info for total
        # count, then subscribe with --cursor max(0, count - SCAN_LIMIT). A
        # cursor=0 + --limit N subscribe returns OLDEST N envelopes — useless
        # for a "recent activity" window AND slow when count > limit because
        # the binary has to page-walk from offset 0.
        SCAN_LIMIT=500
        chat_err="$(mktemp)"
        info_raw="$($TIMEOUT_CMD "$TERMLINK" channel info --hub "$addr" agent-chat-arc --json 2>"$chat_err" || echo '')"
        info_rc=$?
        chat_count=0
        chat_skip=0
        if [ "$info_rc" -ne 0 ] || [ -z "$info_raw" ]; then
            if grep -qE '\-32013|unknown topic|[Nn]ot found' "$chat_err"; then
                # G-060: topic absent on fresh hub. Count stays at 0; not unreachable.
                chat_skip=1
            else
                verdict="partial-unreachable"
            fi
        else
            chat_count="$(printf '%s' "$info_raw" | jq -r '(.count // .posts // 0)' 2>/dev/null || echo 0)"
        fi

        if [ "$chat_skip" -ne 1 ] && [ "$verdict" = "ok" ]; then
            cursor=0
            if [ "$chat_count" -gt "$SCAN_LIMIT" ]; then
                cursor=$((chat_count - SCAN_LIMIT))
            fi
            : > "$chat_err"
            chat_raw="$($TIMEOUT_CMD "$TERMLINK" channel subscribe --hub "$addr" agent-chat-arc \
                            --cursor "$cursor" --since "$since_ms" --limit "$SCAN_LIMIT" --json 2>"$chat_err" || echo '')"
            sub_rc=$?
            if [ -n "$chat_raw" ]; then
                chat_arc_posts="$(printf '%s' "$chat_raw" | grep -cE '^\{' || true)"
                # T-1848 / T-1850: unique speakers in this hub's window.
                # Sender resolution priority (matches T-1849):
                #   1. .metadata.agent_id  (explicit agent identity — /be-reachable)
                #   2. .metadata._from     (vendored-arc heartbeat convention, T-1438)
                #   3. .sender_id          (envelope fingerprint, last resort)
                # T-1848 used #1 only and under-counted vendored-arc posters 75%.
                speakers_payload="$(printf '%s' "$chat_raw" | jq -r -s \
                    '[.[] | select(.msg_type == "chat") | (.metadata.agent_id // .metadata._from // .sender_id // "") | select(. != "")] | unique | .[]' 2>/dev/null || true)"
                if [ -n "$speakers_payload" ]; then
                    unique_speakers="$(printf '%s\n' "$speakers_payload" | sed '/^$/d' | wc -l | tr -d ' ')"
                fi
            elif [ "$sub_rc" -ne 0 ] && ! grep -qE '\-32013|unknown topic|[Nn]ot found' "$chat_err"; then
                verdict="partial-unreachable"
            fi
        fi
        rm -f "$chat_err"

        # --- dm:*:* topic count (lifetime, not windowed — lifetime activity
        # is a better adoption signal for DMs since they're long-lived 1:1) ---
        topics_json="$($TIMEOUT_CMD "$TERMLINK" channel list --hub "$addr" --json 2>/dev/null || echo '')"
        if [ -n "$topics_json" ] && printf '%s' "$topics_json" | jq -e . >/dev/null 2>&1; then
            dm_topic_count="$(printf '%s' "$topics_json" | jq -r '[.topics[] | select(.name | test("^dm:[^:]+:[^:]+$")) | select(.count > 0)] | length')"
        fi
    fi

    if [ "$verdict" != "unreachable" ]; then
        fleet_reachable_hubs=$((fleet_reachable_hubs + 1))
        fleet_listeners=$((fleet_listeners + live_listeners))
        fleet_chat_arc=$((fleet_chat_arc + chat_arc_posts))
        fleet_dm_topics=$((fleet_dm_topics + dm_topic_count))
        # Contribute this hub's speakers to the fleet UNION (deduped at end).
        [ -n "$speakers_payload" ] && printf '%s\n' "$speakers_payload" >> "$fleet_speakers_tmp"
    fi

    jq -n -c \
        --arg name "$name" \
        --arg addr "$addr" \
        --arg verdict "$verdict" \
        --argjson listeners "$live_listeners" \
        --argjson chat_arc "$chat_arc_posts" \
        --argjson dm_topics "$dm_topic_count" \
        --argjson speakers "$unique_speakers" \
        '{name:$name, address:$addr, verdict:$verdict, live_listeners:$listeners, chat_arc_posts:$chat_arc, unique_speakers:$speakers, dm_topic_count:$dm_topics}' \
        >> "$tmp_results"
done

profiles_arr="$(jq -s -c '.' "$tmp_results")"

# T-1848: fleet-wide unique speakers = union across all hubs (not sum).
# An agent posting on hub A and hub B counts as 1, not 2.
fleet_unique_speakers=0
if [ -s "$fleet_speakers_tmp" ]; then
    fleet_unique_speakers="$(sort -u "$fleet_speakers_tmp" | sed '/^$/d' | wc -l | tr -d ' ')"
fi

# Classify adoption_state. T-1848 refinement: HOT now requires ≥2 unique
# speakers — a single agent monologuing (even at 178 posts/24h) is NOT
# an active conversation, it's noise. Closes the gap the user's directive
# called out: "no active doorbell+mail conversations arc".
if [ "$fleet_listeners" -eq 0 ]; then
    adoption_state="COLD"
elif [ "$fleet_unique_speakers" -ge 2 ]; then
    adoption_state="HOT"
else
    adoption_state="WARM"
fi

summary_json="$(jq -n -c \
    --argjson hubs "$total_hubs" \
    --argjson reachable "$fleet_reachable_hubs" \
    --argjson listeners "$fleet_listeners" \
    --argjson chat_arc "$fleet_chat_arc" \
    --argjson speakers "$fleet_unique_speakers" \
    --argjson dm_topics "$fleet_dm_topics" \
    --arg state "$adoption_state" \
    '{hubs:$hubs, reachable_hubs:$reachable, live_listeners:$listeners, chat_arc_posts:$chat_arc, unique_speakers:$speakers, dm_topics_active:$dm_topics, adoption_state:$state}')"

if [ "$FORMAT" = json ]; then
    jq -n -c \
        --argjson window "$WINDOW_HOURS" \
        --argjson summary "$summary_json" \
        --argjson profiles "$profiles_arr" \
        '{ok:true, window_hours:$window, summary:$summary, profiles:$profiles}'
else
    echo "Fleet doorbell+mail adoption (window: last ${WINDOW_HOURS}h):"
    echo "  state:              $adoption_state"
    echo "  hubs:               $total_hubs ($fleet_reachable_hubs reachable)"
    echo "  live_listeners:     $fleet_listeners"
    echo "  chat_arc_posts:     $fleet_chat_arc"
    echo "  unique_speakers:    $fleet_unique_speakers"
    echo "  dm_topics_active:   $fleet_dm_topics"
    echo ""
    printf '%-28s %-22s %-12s %-10s %-10s %-9s %-10s\n' "HUB" "ADDRESS" "VERDICT" "LISTENERS" "CHAT_ARC" "SPEAKERS" "DM_TOPICS"
    printf '%s\n' "$profiles_arr" | jq -r '.[] | [.name, .address, .verdict, (.live_listeners|tostring), (.chat_arc_posts|tostring), (.unique_speakers|tostring), (.dm_topic_count|tostring)] | @tsv' \
        | awk -F'\t' '{printf "%-28s %-22s %-12s %-10s %-10s %-9s %-10s\n", $1, $2, $3, $4, $5, $6, $7}'
fi

exit 0
