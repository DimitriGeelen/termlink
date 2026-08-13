#!/usr/bin/env bash
# check-charter-drift-freshness.sh (T-2483, G-019 breadth-accretion prevention)
#
# WHY: the T-2468 purpose review found TermLink "over-built in breadth". P4
# (T-2471 delete + T-2478 deprecate) pruned 52 charter-untraceable
# social-analytics tools. But NOTHING prevents that breadth from re-accreting —
# there was no automated detector for "the tool surface has drifted from the
# charter's four verbs (discover / exchange durable messages / claim work /
# control terminal sessions)". So the surface silently grows off-purpose and the
# only correction is a human periodically asking for a hand review. That recurring
# manual review IS the symptom of a missing structural check (G-019: fix why the
# framework was blind, not just the instance).
#
# This is the 12th canary, matching the established idiom (empty log = healthy).
#
# TWO DETECTORS (T-2680). A LIVE (deprecated==false) tool is off-charter if EITHER:
#   1. name-pattern -- its name matches the social-analytics families
#      (reactions / emoji / stars / pins / typing / polls); the original T-2483
#      detector, kept as-is.
#   2. category     -- its catalog category is one the binary itself names as
#      analytics (agent_rankings / agent_stats / agent_thread_health /
#      channel_engagement / *_poll / *engagement_metrics).
#
# WHY DETECTOR 2 EXISTS. T-2483 shipped with the name regex alone and reported
# `{checked:214, live_off_charter:0}` -- which reads as a full-surface
# charter-traceability clean bill. It was not one: the regex knew only the six
# families P4 had just deleted, so `termlink_agent_top_reacted` fired while
# `termlink_agent_top_repliers` -- a functionally identical social leaderboard --
# passed clean, and 28 live tools in explicitly-analytics categories were
# invisible. Those 28 are precisely what T-2548 (owner: human) is incepting to
# subtract, so the project had an open decision about ~30 off-charter tools AND a
# daily canary reporting that surface as clean. A guard reporting green is why
# nobody looks -- Directive #2 violated in the guard layer itself. See T-2678 F2.
#
# ACKNOWLEDGEMENT ALLOWLIST. Detecting the 28 must not mean alarming daily on a
# decision the human has not made. `.context/checks/charter-drift-allowlist`
# (git-tracked on purpose -- see T-2681) lists `<tool>  # <reason>` entries that
# are still COUNTED and REPORTED as off-charter but do not fire. Removing a line
# re-fires it; a NEW off-charter tool fires immediately because it is not listed.
# The allowlist makes the pending decision visible instead of invisible.
#
# SCOPE -- read this before quoting a healthy result. This canary detects known
# off-charter SHAPES. It does NOT assess every tool's traceability to a charter
# verb, and a healthy exit must never be reported as "the whole surface is
# charter-clean". Every output path carries that disclaimer explicitly.
#
# It reads `termlink help --json` -- a {category: [tool, ...]} map where each tool
# carries {name, deprecated}, `deprecated` derived from the registry's own
# is_deprecated(). The canary trusts the binary's own classification for both
# liveness and category.
#
# EXIT CODES:
#   0  healthy   -- no UNACKNOWLEDGED live tool matches an off-charter shape.
#   1  firing    -- >=1 live, unacknowledged tool matches: charter drift.
#   2  tooling   -- could not read/parse the tool catalog (fail-closed).
#
# USAGE:
#   check-charter-drift-freshness.sh [--quiet] [--json] [--no-heartbeat] [--allowlist P]
#     --quiet         print only on firing (cron mode); healthy prints nothing
#     --json          emit {ok, firing:[{name,category,why,reason}], checked,
#                     live_off_charter, acknowledged_count, acknowledged[],
#                     off_charter_total, detectors[], scope}
#     --no-heartbeat  skip the heartbeat touch (meta-canary invokes with this)
#     --allowlist P   use an alternate acknowledgement allowlist (fixtures)
#
# TEST HOOK (PL-213 -- host-independent, no live binary):
#   TERMLINK_CHARTER_DRIFT_TEST_JSON=<file>   canned `termlink help --json`
#   CHARTER_DRIFT_ALLOWLIST=<file>            alternate allowlist path
#   Fixture suite: bash tests/charter-drift-check-fixtures.sh
#
# Origin: T-2468 purpose review (5th re-issue) -> this build. Turns "please review
# purpose" from a manual chore into a daily automated check. Sibling of the other
# 11 canaries; /canaries auto-discovers the log.
set -uo pipefail

TERMLINK="${TERMLINK_BIN:-termlink}"
QUIET=0 WANT_JSON=0 HEARTBEAT=1

# The charter-untraceable social-analytics pattern. Anchored/whole-word forms so it
# does NOT false-positive on core primitives — validated against the live catalog:
#   _pin$   matches agent_pin/channel_pin but NOT _ping (termlink_ping)
#   _star$  matches agent_star/channel_star but NOT _start (hub_start) / _starters
#   _poll_(start|vote|end|results) matches social polls but NOT event_poll
CHARTER_DRIFT_PATTERN='react|reaction|emoji|top_reacted|_star$|starred|starrer|_pin$|pinned|pinner|pin_history|typing|typers|_poll_start|_poll_vote|_poll_end|_poll_results'

# T-2680 — SECOND detector: the tool's CATEGORY, as the binary itself reports it.
# The name regex above is a proxy for "off-charter" that only knows the six
# families P4 happened to delete; it read `top_reacted` as drift and
# `top_repliers` — a functionally identical leaderboard — as clean, while
# reporting `live_off_charter:0` over the whole 214-tool surface. Categories are
# the binary's own classification, so this detector does not depend on guessing
# which word a future analytics tool will be named after.
#
# Deliberately NOT included: channel_moderation (redact/pin-hygiene reads as
# coordination), agent_read / agent_thread / agent_inbox (charter verb 2).
CHARTER_DRIFT_CATEGORY_PATTERN='^(agent_rankings|agent_stats|agent_thread_health|channel_engagement|agent_engagement_metrics|agent_poll|channel_poll)$'

# Acknowledgement allowlist — git-tracked on purpose (see the file's own header
# and T-2681). A listed tool is still classified off-charter, just not alarmed on.
ALLOWLIST="${CHARTER_DRIFT_ALLOWLIST:-.context/checks/charter-drift-allowlist}"

while [ $# -gt 0 ]; do
    case "$1" in
        --quiet) QUIET=1; shift ;;
        --json)  WANT_JSON=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --allowlist) ALLOWLIST="${2:-}"; shift 2 ;;
        -h|--help)
            sed -n '2,47p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "check-charter-drift: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v jq >/dev/null 2>&1 || { echo "check-charter-drift: jq not found (required)" >&2; exit 2; }

# T-1723 heartbeat: prove this canary ran, even on healthy/error cycles.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.charter-drift-canary.heartbeat}"
if [ "$HEARTBEAT" -eq 1 ]; then
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# --- read the tool catalog (test hook wins; else live binary) ---------------
if [ -n "${TERMLINK_CHARTER_DRIFT_TEST_JSON:-}" ]; then
    catalog="$(cat "$TERMLINK_CHARTER_DRIFT_TEST_JSON" 2>/dev/null || true)"
else
    catalog="$("$TERMLINK" help --json 2>/dev/null || true)"
fi
[ -n "$catalog" ] || { echo "check-charter-drift: could not read 'termlink help --json'" >&2; exit 2; }

# Flatten to `<category>\t<name>` for every LIVE tool. The category is the
# catalog's own top-level key, so the category detector uses the binary's
# classification rather than a guess about naming (T-2680). Tool objects nested
# deeper than one level still resolve — their nearest enclosing array key is
# used — and a tool with no discoverable category gets "-" (name detector only).
live_pairs="$(printf '%s' "$catalog" | jq -r '
    if type=="object" then
        to_entries[]
        | .key as $cat
        | (if (.value|type)=="array" then .value else [] end)[]
        | select(type=="object" and has("name") and has("deprecated"))
        | select(.deprecated==false)
        | "\($cat)\t\(.name)"
    else empty end' 2>/dev/null | sort -u)"

# Fallback: catalogs that are not {category: [tools]} still get name-only scanning.
if [ -z "$live_pairs" ]; then
    live_pairs="$(printf '%s' "$catalog" \
        | jq -r '[.. | objects | select(has("name") and has("deprecated")) | select(.deprecated==false) | .name] | unique[] | "-\t\(.)"' 2>/dev/null | sort -u)"
fi

if [ -z "$live_pairs" ] && ! printf '%s' "$catalog" | jq -e '.. | objects | select(has("name") and has("deprecated"))' >/dev/null 2>&1; then
    echo "check-charter-drift: tool catalog has no {name,deprecated} objects (unparseable)" >&2
    exit 2
fi

checked_count="$(printf '%s' "$live_pairs" | grep -c . || true)"

# --- load the acknowledgement allowlist ------------------------------------
# `<tool_name>  # <reason>` per line; `#`-comment and blank lines skipped.
ack_names=""
if [ -f "$ALLOWLIST" ]; then
    ack_names="$(sed 's/#.*//' "$ALLOWLIST" | tr -d '\r' | awk 'NF{print $1}' | sort -u)"
fi

# --- classify --------------------------------------------------------------
# A live tool is off-charter if EITHER detector matches. Emit `<name>\t<why>`.
off_charter="$(printf '%s' "$live_pairs" | awk -F'\t' \
    -v npat="$CHARTER_DRIFT_PATTERN" -v cpat="$CHARTER_DRIFT_CATEGORY_PATTERN" '
    NF < 2 { next }
    {
        cat = $1; name = $2; why = ""
        if (tolower(name) ~ npat) why = "name-pattern"
        if (cat ~ cpat) why = (why == "" ? "category:" cat : why "+category:" cat)
        if (why != "") print name "\t" why
    }' | sort -u)"

# Split off-charter into acknowledged (listed) vs firing (not listed).
firing="" acknowledged=""
if [ -n "$off_charter" ]; then
    if [ -n "$ack_names" ]; then
        ack_file="$(mktemp)"; printf '%s\n' "$ack_names" > "$ack_file"
        firing="$(printf '%s\n' "$off_charter" | awk -F'\t' 'NR==FNR{a[$1];next} !($1 in a)' "$ack_file" - || true)"
        acknowledged="$(printf '%s\n' "$off_charter" | awk -F'\t' 'NR==FNR{a[$1];next} ($1 in a)' "$ack_file" - || true)"
        rm -f "$ack_file"
    else
        firing="$off_charter"
    fi
fi

ack_count="$(printf '%s' "$acknowledged" | grep -c . || true)"
fire_count="$(printf '%s' "$firing" | grep -c . || true)"
total_off="$(printf '%s' "$off_charter" | grep -c . || true)"

# Scope disclaimer — the honesty fix. This canary detects a KNOWN off-charter
# SHAPE (social-analytics naming + analytics categories). It does not and cannot
# assess every tool's traceability to a charter verb, so it must never be read
# as a full-surface clean bill (T-2680; that misreading is what T-2483 shipped).
SCOPE_NOTE="detects known off-charter shapes (social-analytics names + analytics categories); not a full charter-traceability audit of every tool"

if [ "$fire_count" -eq 0 ]; then
    if [ "$WANT_JSON" -eq 1 ]; then
        ack_arr="$(printf '%s' "$acknowledged" | awk -F'\t' 'NF{print $1"\t"$2}' \
            | jq -Rc 'select(length>0) | split("\t") | {name:.[0], why:.[1]}' | jq -sc '.')"
        jq -cn --argjson checked "${checked_count:-0}" --argjson ack "${ack_count:-0}" \
               --argjson total "${total_off:-0}" --argjson acks "${ack_arr:-[]}" --arg note "$SCOPE_NOTE" \
            '{ok:true, firing:[], checked:$checked, live_off_charter:0,
              acknowledged_count:$ack, acknowledged:$acks, off_charter_total:$total,
              detectors:["name-pattern","category"], scope:$note}'
    elif [ "$QUIET" -eq 0 ]; then
        echo "check-charter-drift: healthy — 0 UNACKNOWLEDGED off-charter tools."
        echo "  scanned:      ${checked_count:-0} live tools, by 2 detectors (name-pattern, category)"
        echo "  off-charter:  ${total_off:-0} total, ${ack_count:-0} acknowledged in $ALLOWLIST"
        echo "  scope:        $SCOPE_NOTE"
        if [ "${ack_count:-0}" -gt 0 ]; then
            echo "  NOTE: the acknowledged tools are a PENDING decision, not a clean bill —"
            echo "        see T-2548 (owner: human) and the allowlist's own header."
        fi
    fi
    exit 0
fi

# firing
if [ "$WANT_JSON" -eq 1 ]; then
    firing_arr="$(printf '%s' "$firing" | awk -F'\t' 'NF{print $1"\t"$2}' \
        | jq -Rc 'select(length>0) | split("\t") | {name:.[0], category:"off-charter", why:.[1], reason:"live (deprecated==false) tool matches an off-charter shape and is not acknowledged"}' | jq -sc '.')"
    ack_arr="$(printf '%s' "$acknowledged" | awk -F'\t' 'NF{print $1"\t"$2}' \
        | jq -Rc 'select(length>0) | split("\t") | {name:.[0], why:.[1]}' | jq -sc '.')"
    jq -cn --argjson f "${firing_arr:-[]}" --argjson checked "${checked_count:-0}" --argjson fc "${fire_count:-0}" \
           --argjson ack "${ack_count:-0}" --argjson total "${total_off:-0}" --argjson acks "${ack_arr:-[]}" --arg note "$SCOPE_NOTE" \
        '{ok:false, firing:$f, checked:$checked, live_off_charter:$fc,
          acknowledged_count:$ack, acknowledged:$acks, off_charter_total:$total,
          detectors:["name-pattern","category"], scope:$note}'
else
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-charter-drift: FIRING — $fire_count live tool(s) drift from the charter (off-charter, not deprecated, not acknowledged):"
    printf '%s\n' "$firing" | awk -F'\t' 'NF{printf "  ↳ %s  [%s]\n", $1, $2}'
    echo "  scanned ${checked_count:-0} live tools; ${total_off:-0} off-charter total, ${ack_count:-0} acknowledged."
    echo "  These re-accrete the 'over-built breadth' the T-2468 review flagged. Either deprecate them"
    echo "  (mirror the remote_inbox_* pattern per docs/operations/p4-surface-reduction.md); or, if one"
    echo "  genuinely traces to a charter verb, rename it / adjust the detector; or — if it is a known"
    echo "  pending decision — acknowledge it in $ALLOWLIST with a cited reason."
    echo "  scope: $SCOPE_NOTE"
    echo "---"
fi
exit 1
