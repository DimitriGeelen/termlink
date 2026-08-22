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
# This is the 12th canary, matching the established idiom (empty log = healthy):
# it flags any LIVE (deprecated==false) MCP tool whose name matches the
# charter-untraceable social-analytics pattern (reactions / emoji / stars / pins /
# typing / polls). After P4 that set is empty; the canary FIRES only if a live
# off-charter tool re-appears (a new one added, or a deprecated one un-marked).
#
# It reads `termlink help --json` — each tool object carries {name, deprecated},
# where `deprecated` is derived from the registry's own is_deprecated(). So the
# canary trusts the binary's classification and only applies the name pattern.
#
# EXIT CODES:
#   0  healthy   -- no live tool matches the off-charter pattern.
#   1  firing    -- >=1 live (deprecated==false) tool matches: charter drift.
#   2  tooling    -- could not read/parse the tool catalog.
#
# USAGE:
#   check-charter-drift-freshness.sh [--quiet] [--json] [--no-heartbeat]
#     --quiet         print only on firing (cron mode); healthy prints nothing
#     --json          emit {ok, firing:[{name,category,reason}], checked, live_off_charter}
#     --no-heartbeat  skip the heartbeat touch (meta-canary invokes with this)
#
# TEST HOOK (PL-213 -- host-independent, no live binary):
#   TERMLINK_CHARTER_DRIFT_TEST_JSON=<file>   canned `termlink help --json`
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

while [ $# -gt 0 ]; do
    case "$1" in
        --quiet) QUIET=1; shift ;;
        --json)  WANT_JSON=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
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

# --- record WHICH binary answered (T-2829) ----------------------------------
# This check asks "has the LIVE tool surface drifted from the charter?" but it can
# only ever see the binary `$TERMLINK` resolves to. `TERMLINK_BIN` defaults to a
# bare `termlink`, so under cron's PATH (/usr/local/bin:/usr/bin:/bin) it reads
# whatever build sits in /usr/local/bin — NOT the binary this project builds and
# installs to ~/.cargo/bin.
#
# That is not hypothetical. From 2026-08-02 to 2026-08-22 this canary appended a
# 40-tool FIRING entry every single day. All 40 were false: /usr/local/bin/termlink
# was a Jul-31 build (0.11.693) reporting them deprecated==false, while the
# binary on the developer's PATH (0.11.1196) reports every one of them
# deprecated==true. Twenty consecutive days of alarms about an already-clean
# surface, and nothing in the output said which binary had been asked.
#
# A version FLOOR was considered and rejected. Deprecation here is monotonic —
# diffing the two catalogs shows 6 deprecated at 0.11.693, 46 at 0.11.1196, and
# NOTHING un-deprecated in between — so the 40 tools that separate the builds are
# exactly this canary's firing set. The staleness discriminator and the drift
# verdict are the same 40 names, and no threshold over them can tell "stale
# binary" from "genuine re-accretion". That is the T-2415 lesson in miniature:
# a version floor is provably blind to a capability question. Worse, VERSION is
# stamped per-commit while the installed binary always trails it, so a
# `>= VERSION` gate would refuse on every healthy run.
#
# What the check CAN always do honestly is say what it looked at. Naming the
# resolved path and version turns an ambiguous 40-tool alarm into a one-line
# diagnosis, and the crontab pins TERMLINK_BIN so the daily run stops resolving
# to a stale /usr/local/bin copy in the first place.
if [ -n "${TERMLINK_CHARTER_DRIFT_TEST_JSON:-}" ]; then
    probe_binary="(test hook: $TERMLINK_CHARTER_DRIFT_TEST_JSON)"
    probe_version="n/a"
else
    probe_binary="$(command -v "$TERMLINK" 2>/dev/null || printf '%s' "$TERMLINK")"
    probe_version="$("$TERMLINK" --version 2>/dev/null | awk '{print $NF}')"
    [ -n "$probe_version" ] || probe_version="unknown"
fi

# --- read the tool catalog (test hook wins; else live binary) ---------------
if [ -n "${TERMLINK_CHARTER_DRIFT_TEST_JSON:-}" ]; then
    catalog="$(cat "$TERMLINK_CHARTER_DRIFT_TEST_JSON" 2>/dev/null || true)"
else
    catalog="$("$TERMLINK" help --json 2>/dev/null || true)"
fi
[ -n "$catalog" ] || { echo "check-charter-drift: could not read 'termlink help --json'" >&2; exit 2; }

# Flatten every tool object carrying {name, deprecated}; keep LIVE ones only.
live_names="$(printf '%s' "$catalog" \
    | jq -r '[.. | objects | select(has("name") and has("deprecated")) | select(.deprecated==false) | .name] | unique[]' 2>/dev/null)"
if [ -z "$live_names" ] && ! printf '%s' "$catalog" | jq -e '.. | objects | select(has("name") and has("deprecated"))' >/dev/null 2>&1; then
    echo "check-charter-drift: tool catalog has no {name,deprecated} objects (unparseable)" >&2
    exit 2
fi

# Firing set: live tools whose name matches the off-charter social pattern.
firing="$(printf '%s' "$live_names" | grep -iE "$CHARTER_DRIFT_PATTERN" | sort -u || true)"
checked_count="$(printf '%s' "$live_names" | grep -c . || true)"

if [ -z "$firing" ]; then
    # healthy
    if [ "$WANT_JSON" -eq 1 ]; then
        jq -cn --argjson checked "${checked_count:-0}" \
            --arg pb "$probe_binary" --arg pv "$probe_version" \
            '{ok:true, firing:[], checked:$checked, live_off_charter:0, probe_binary:$pb, probe_version:$pv}'
    elif [ "$QUIET" -eq 0 ]; then
        echo "check-charter-drift: healthy — 0 live off-charter tools (${checked_count:-0} live tools scanned)."
        echo "  (read from $probe_binary, version $probe_version)"
    fi
    exit 0
fi

# firing
fire_count="$(printf '%s' "$firing" | grep -c .)"
if [ "$WANT_JSON" -eq 1 ]; then
    firing_arr="$(printf '%s' "$firing" | jq -R '{name:., category:"social-analytics", reason:"live (deprecated==false) tool matches charter-untraceable pattern"}' | jq -sc '.')"
    jq -cn --argjson f "$firing_arr" --argjson checked "${checked_count:-0}" --argjson fc "$fire_count" \
        --arg pb "$probe_binary" --arg pv "$probe_version" \
        '{ok:false, firing:$f, checked:$checked, live_off_charter:$fc, probe_binary:$pb, probe_version:$pv}'
else
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-charter-drift: FIRING — $fire_count live tool(s) drift from the charter (off-charter social-analytics, not deprecated):"
    echo "  probed: $probe_binary (version $probe_version)"
    printf '%s\n' "$firing" | sed 's/^/  ↳ /'
    echo "  These re-accrete the 'over-built breadth' the T-2468 review flagged. Either deprecate them"
    echo "  (mirror the remote_inbox_* pattern per docs/operations/p4-surface-reduction.md) or, if one"
    echo "  genuinely traces to a charter verb, rename it / adjust the pattern. See docs/CHARTER.md."
    echo "---"
fi
exit 1
