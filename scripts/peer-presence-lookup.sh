#!/usr/bin/env bash
#
# peer-presence-lookup.sh (T-1896)
#
# Resolve peer IDENTITY fingerprints (16-char host fps, NOT TLS leaf-cert
# sha256) to current presence status: LIVE / STALE / OFFLINE / UNKNOWN.
#
# Extracted from T-1895's check-outbox.sh (sections A/B/C) so both
# /check-outbox (OUTBOUND skill) and /check-arc (INBOUND skill, T-1896)
# share one fp→status join with one source of truth.
#
# Read-only by contract — no `KnownHubStore` mutation, no posts, no acks.
#
# Usage:
#   echo -e "<fp1>\n<fp2>" | scripts/peer-presence-lookup.sh   # stdin (one per line)
#   scripts/peer-presence-lookup.sh <fp> [<fp> ...]            # positional args
#   scripts/peer-presence-lookup.sh --all                      # every known fp
#   scripts/peer-presence-lookup.sh --json [--all|<fp>...]     # JSON array form
#
# Output (default): TSV `<fp>\t<status>` per input fp, one per line.
# Output (--json):  `[{fp, status, hub}, ...]` — hub is null when UNKNOWN.
#
# Status semantics:
#   LIVE     — listener on the host emitting this fp is currently advertising
#   STALE    — listener last advertised >grace_period ago
#   OFFLINE  — fp's hub is known but no listener of any state
#   UNKNOWN  — fp not found on any reachable hub's agent-presence/agent-chat-arc
#
# Failure-tolerant: if section A or B returns no data, affected fps render as
# UNKNOWN and a one-line diagnostic goes to stderr. Exit 0.
#
# Options:
#   --hubs-file PATH   Custom hubs.toml (default: ~/.termlink/hubs.toml)
#   --all              Dump every known fp from sections A+B
#   --json             Emit JSON array
#   -h, --help         This help
#
# Related:
#   T-1895 (check-outbox.sh --with-presence — the OUTBOUND skill this serves)
#   T-1896 (check-arc.sh --with-presence — the INBOUND skill this serves)
#   PL-116 (symmetric SEND+RECEIVE deployment — why both callers exist)
#   PL-195 / T-1693 (shared-host identity — every co-resident claude session
#                    signs as the host's identity key)

set -u

usage() {
    sed -n '2,42p' "$0" | sed 's/^# \{0,1\}//'
}

TERMLINK="${TERMLINK:-termlink}"
TIMEOUT_CMD="${TIMEOUT_CMD:-timeout 8}"
HUBS_FILE="${HUBS_FILE:-$HOME/.termlink/hubs.toml}"
ALL=0
FORMAT="tsv"
declare -a INPUT_FPS=()

while [ $# -gt 0 ]; do
    case "$1" in
        --hubs-file) HUBS_FILE="$2"; shift 2 ;;
        --all) ALL=1; shift ;;
        --json) FORMAT="json"; shift ;;
        -h|--help) usage; exit 0 ;;
        --) shift; while [ $# -gt 0 ]; do INPUT_FPS+=("$1"); shift; done ;;
        --*) echo "peer-presence-lookup: unknown flag: $1" >&2; usage >&2; exit 2 ;;
        *) INPUT_FPS+=("$1"); shift ;;
    esac
done

# Collect stdin fps if not --all and stdin is a pipe.
if [ "$ALL" -eq 0 ] && [ ${#INPUT_FPS[@]} -eq 0 ] && [ ! -t 0 ]; then
    while IFS= read -r _line; do
        _line="${_line%$'\r'}"
        _line="${_line#"${_line%%[![:space:]]*}"}"
        _line="${_line%"${_line##*[![:space:]]}"}"
        [ -z "$_line" ] && continue
        INPUT_FPS+=("$_line")
    done
fi

if [ "$ALL" -eq 0 ] && [ ${#INPUT_FPS[@]} -eq 0 ]; then
    echo "peer-presence-lookup: no fingerprints provided (pass as args, stdin, or use --all)" >&2
    usage >&2
    exit 2
fi

command -v jq >/dev/null 2>&1 || { echo "peer-presence-lookup: jq not on PATH" >&2; exit 3; }

if [ ! -f "$HUBS_FILE" ]; then
    echo "peer-presence-lookup: hubs file not found: $HUBS_FILE" >&2
    exit 3
fi

# Section A — walk hubs.toml, query each hub's agent-presence (fallback chat-arc),
# build fp → MULTI-HUB SET (colon-delimited list). A fp may post to several
# hubs (broadcast fan-out via /broadcast-chat), so a single "first-seen wins"
# mapping mis-resolves when the LIVE listener is on a non-first hub. Section C
# walks the set and prefers LIVE > STALE > OFFLINE > UNKNOWN.
declare -A _fp_to_hubs=()
declare -a _prof_names=()
declare -a _prof_addrs=()
_cur_name=""
while IFS= read -r _raw; do
    _l="${_raw%$'\r'}"
    _l="${_l%%#*}"
    _l="${_l#"${_l%%[![:space:]]*}"}"
    _l="${_l%"${_l##*[![:space:]]}"}"
    [ -z "$_l" ] && continue
    if [[ "$_l" =~ ^\[hubs\.([A-Za-z0-9_.-]+)\][[:space:]]*$ ]]; then
        _cur_name="${BASH_REMATCH[1]}"
    elif [ -n "$_cur_name" ] && [[ "$_l" =~ ^address[[:space:]]*=[[:space:]]*\"([^\"]+)\"[[:space:]]*$ ]]; then
        _prof_names+=("$_cur_name")
        _prof_addrs+=("${BASH_REMATCH[1]}")
        _cur_name=""
    fi
done < "$HUBS_FILE"

_section_a_err=""
if [ "${#_prof_names[@]}" -eq 0 ]; then
    _section_a_err="hubs.toml has no profiles"
else
    for _i in "${!_prof_names[@]}"; do
        _pn="${_prof_names[$_i]}"
        _pa="${_prof_addrs[$_i]}"
        _senders_json="$($TIMEOUT_CMD "$TERMLINK" channel info agent-presence --hub "$_pn" --json 2>/dev/null || true)"
        _has_sender="$(printf '%s' "$_senders_json" | jq -r '.senders[]?.sender_id' 2>/dev/null | head -1)"
        if [ -z "$_has_sender" ]; then
            _senders_json="$($TIMEOUT_CMD "$TERMLINK" channel info agent-chat-arc --hub "$_pn" --json 2>/dev/null || true)"
        fi
        while IFS= read -r _sid; do
            [ -n "$_sid" ] || continue
            _existing="${_fp_to_hubs[$_sid]:-:}"
            if [[ "$_existing" != *:"$_pn":* ]]; then
                _fp_to_hubs["$_sid"]="${_existing}${_pn}:"
            fi
        done < <(printf '%s' "$_senders_json" | jq -r '.senders[]?.sender_id' 2>/dev/null)
    done
fi

# Section B — listener fleet → hub_name → max(status). Invert addr→name via _prof_*.
declare -A _hub_name_to_status=()
_section_b_err=""
_listeners_json="$(timeout 15 bash "$(dirname "$0")/agent-listeners-fleet.sh" --include-offline --hubs-file "$HUBS_FILE" --json 2>/dev/null || true)"
if [ -z "$_listeners_json" ] || ! printf '%s' "$_listeners_json" | jq -e '.ok' >/dev/null 2>&1; then
    _section_b_err="agent-listeners-fleet returned no data"
else
    declare -A _addr_to_name=()
    for _i in "${!_prof_names[@]}"; do
        _addr_to_name["${_prof_addrs[$_i]}"]="${_prof_names[$_i]}"
    done
    while IFS=$'\t' read -r _l_hub_addr _l_status; do
        [ -n "$_l_hub_addr" ] || continue
        _l_name="${_addr_to_name[$_l_hub_addr]:-$_l_hub_addr}"
        _cur="${_hub_name_to_status[$_l_name]:-}"
        case "$_l_status" in
            LIVE)    _hub_name_to_status["$_l_name"]=LIVE ;;
            STALE)   [ "$_cur" != "LIVE" ] && _hub_name_to_status["$_l_name"]=STALE ;;
            OFFLINE) [ "$_cur" != "LIVE" ] && [ "$_cur" != "STALE" ] && _hub_name_to_status["$_l_name"]=OFFLINE ;;
        esac
    done < <(printf '%s' "$_listeners_json" | jq -r '.listeners[]? | "\(.hub)\t\(.status)"' 2>/dev/null)
fi

# Build effective input list (--all expands to every fp known from section A).
if [ "$ALL" -eq 1 ]; then
    INPUT_FPS=()
    for _fp in "${!_fp_to_hubs[@]}"; do
        INPUT_FPS+=("$_fp")
    done
fi

# Section C — resolve each input fp to (status, hub).
# Walk the fp's hub-set; prefer LIVE > STALE > OFFLINE. The best-status hub
# is the one we surface so the operator knows where to reach the peer.
resolve_one() {
    local fp="$1" hubs_str best_status best_hub _h _s
    hubs_str="${_fp_to_hubs[$fp]:-}"
    if [ -z "$hubs_str" ]; then
        printf '%s\t%s\t%s\n' "$fp" "UNKNOWN" ""
        return
    fi
    best_status="OFFLINE"
    best_hub=""
    # Iterate colon-delimited hub_names.
    _rest="${hubs_str#:}"
    while [ -n "$_rest" ]; do
        _h="${_rest%%:*}"
        _rest="${_rest#*:}"
        [ -z "$_h" ] && continue
        _s="${_hub_name_to_status[$_h]:-OFFLINE}"
        [ -z "$best_hub" ] && best_hub="$_h"
        case "$_s" in
            LIVE)    best_status="LIVE"; best_hub="$_h"; break ;;
            STALE)   [ "$best_status" != "LIVE" ] && { best_status="STALE"; best_hub="$_h"; } ;;
            OFFLINE) : ;;
        esac
    done
    printf '%s\t%s\t%s\n' "$fp" "$best_status" "$best_hub"
}

if [ "$FORMAT" = "json" ]; then
    {
        echo "["
        first=1
        for fp in "${INPUT_FPS[@]}"; do
            row="$(resolve_one "$fp")"
            _f="$(printf '%s' "$row" | cut -f1)"
            _s="$(printf '%s' "$row" | cut -f2)"
            _h="$(printf '%s' "$row" | cut -f3)"
            if [ "$first" -eq 1 ]; then first=0; else echo ","; fi
            if [ -n "$_h" ]; then
                jq -n -c --arg fp "$_f" --arg status "$_s" --arg hub "$_h" '{fp:$fp, status:$status, hub:$hub}'
            else
                jq -n -c --arg fp "$_f" --arg status "$_s" '{fp:$fp, status:$status, hub:null}'
            fi
        done
        echo "]"
    } | jq -s 'add // []'
else
    for fp in "${INPUT_FPS[@]}"; do
        resolve_one "$fp" | cut -f1,2
    done
fi

[ -n "$_section_a_err" ] && echo "peer-presence-lookup: section A partial: $_section_a_err (affected fps → UNKNOWN)" >&2
[ -n "$_section_b_err" ] && echo "peer-presence-lookup: section B partial: $_section_b_err (known fps → OFFLINE)" >&2

exit 0
