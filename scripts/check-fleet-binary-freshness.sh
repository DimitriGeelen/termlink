#!/usr/bin/env bash
# T-2359 — Fleet binary-freshness canary (G-069 prevention).
#
# G-069: fleet hubs ran stale binaries for weeks and nothing FIRED — .122
# served a pre-arc-004 feature set for ~13 days while the push-transport arc
# was recorded closed=shipped. `fleet doctor` prints per-hub hub_version and
# a fleet_versions histogram, but nothing alerts on it; preflight Check 5
# (T-2184) covers only the LOCAL hub. This canary walks `fleet doctor --json`
# and FIRES when any reachable hub serves a version BELOW its declared floor.
#
# Floors come from a config file (default .context/cron/fleet-version-floors.conf):
#     <hub-name> <min-version>     # fire when served < min-version
#     <hub-name> -                 # exempt (foreign build lineage, transient host)
#     * <min-version>              # optional default for undeclared hubs
# Comments (#) and blank lines are skipped. Hubs with no row and no `*` row
# are informational, never firing.
#
# Deliberately NOT cross-hub skew detection: patch numbers are commits-since-
# tag and are NOT comparable across build lineages (ring20-dashboard serves
# 0.11.806 from its own fork — numerically "newest" while lacking our
# commits). A declared floor is the only sound gate; bumping it when hub-side
# rails ship is the operator's declaration that "shipped" must mean
# "capability-live".
#
# Firing semantics:
#   - reachable + floored + served < floor          → FIRE
#   - reachable + floored + version missing/unknown → FIRE (a hub too old to
#     report its version IS the staleness class)
#   - unreachable/down                              → informational (PL-219 —
#     fleet doctor/status already surface down hubs; a down hub serves nothing)
#   - exempt (-) or no floor declared               → informational
#
# Exit codes:
#   0 — all floored reachable hubs at/above floor
#   1 — at least one firing hub
#   2 — tooling error (fleet doctor unrunnable, floors file unreadable, jq missing)
#
# Usage:
#   check-fleet-binary-freshness.sh            # human-readable, one-shot
#   check-fleet-binary-freshness.sh --quiet    # print only on firing (cron)
#   check-fleet-binary-freshness.sh --json     # {ok, firing[], hubs[]}
#   check-fleet-binary-freshness.sh --floors F # alternate floors file
#
# Test hook (PL-213): TERMLINK_FLEET_FRESHNESS_TEST_JSON=<file> feeds canned
# `fleet doctor --json` output for hub-independent verification.
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
FLOORS_FILE="${FLEET_FLOORS_FILE:-.context/cron/fleet-version-floors.conf}"
DOCTOR_TIMEOUT="${FLEET_FRESHNESS_TIMEOUT:-180}"

FORMAT=human
QUIET=0
HEARTBEAT=1

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        --floors) shift; FLOORS_FILE="${1:?--floors needs a path}" ;;
        -h|--help) sed -n '2,48p' "$0"; exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

die_setup() {
    if [ "$FORMAT" = json ]; then
        printf '{"ok":false,"error":"%s"}\n' "$1"
    else
        echo "fleet-binary-freshness: SETUP-FAIL — $1" >&2
    fi
    exit 2
}

# T-1723 heartbeat: prove this canary ran, even on healthy/error cycles.
# Placed BEFORE the network call so a fleet-doctor hang still leaves a beat.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.fleet-binary-canary.heartbeat}"
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

command -v jq >/dev/null 2>&1 || die_setup "jq not found"
[ -r "$FLOORS_FILE" ] || die_setup "floors file not readable: $FLOORS_FILE"

# ── acquire fleet doctor JSON ────────────────────────────────────────────────
if [ -n "${TERMLINK_FLEET_FRESHNESS_TEST_JSON:-}" ]; then
    [ -r "$TERMLINK_FLEET_FRESHNESS_TEST_JSON" ] || die_setup "test JSON not readable"
    DOCTOR_JSON=$(cat "$TERMLINK_FLEET_FRESHNESS_TEST_JSON")
else
    command -v "$TERMLINK" >/dev/null 2>&1 || die_setup "termlink not on PATH"
    # fleet doctor exits non-zero when any hub is down — that is data, not a
    # tooling error. Only an unparseable/empty body is a setup failure.
    DOCTOR_JSON=$(timeout "$DOCTOR_TIMEOUT" "$TERMLINK" fleet doctor --json 2>/dev/null) || true
fi
echo "$DOCTOR_JSON" | jq -e '.hubs | type == "array"' >/dev/null 2>&1 \
    || die_setup "fleet doctor --json produced no parseable .hubs[]"

# ── floors lookup ────────────────────────────────────────────────────────────
floor_for() { # $1 = hub name → prints floor, "-", or "" (no row)
    local hub="$1" name ver hit="" star=""
    while read -r name ver _; do
        case "$name" in ''|'#'*) continue ;; esac
        [ "$name" = "$hub" ] && hit="$ver"
        [ "$name" = "*" ] && star="$ver"
    done < "$FLOORS_FILE"
    if [ -n "$hit" ]; then printf '%s' "$hit"; else printf '%s' "$star"; fi
}

version_lt() { # returns 0 iff $1 < $2, numeric per dotted segment
    local -a a b
    IFS=. read -r -a a <<< "${1#v}"
    IFS=. read -r -a b <<< "${2#v}"
    local i x y
    for i in 0 1 2 3; do
        x="${a[i]:-0}"; y="${b[i]:-0}"
        x="${x//[!0-9]/}"; y="${y//[!0-9]/}"
        x="${x:-0}"; y="${y:-0}"
        if [ "$x" -lt "$y" ] 2>/dev/null; then return 0; fi
        if [ "$x" -gt "$y" ] 2>/dev/null; then return 1; fi
    done
    return 1
}

# ── walk hubs ────────────────────────────────────────────────────────────────
FIRING_LINES=""
INFO_LINES=""
FIRING_JSON="[]"
HUBS_JSON="[]"
FIRING_COUNT=0

add_hub_json() { # name served floor state
    HUBS_JSON=$(echo "$HUBS_JSON" | jq -c \
        --arg hub "$1" --arg served "$2" --arg floor "$3" --arg state "$4" \
        '. + [{hub:$hub, served:(if $served=="" then null else $served end), floor:(if $floor=="" then null else $floor end), state:$state}]')
}

while IFS=$'\t' read -r hub status served; do
    floor=$(floor_for "$hub")
    if [ "$status" != "ok" ]; then
        INFO_LINES="${INFO_LINES}  ~ ${hub}: unreachable (not firing — fleet doctor surfaces down hubs)\n"
        add_hub_json "$hub" "$served" "$floor" "unreachable"
        continue
    fi
    if [ -z "$floor" ] || [ "$floor" = "-" ]; then
        state="exempt"; [ -z "$floor" ] && state="no-floor"
        INFO_LINES="${INFO_LINES}  ~ ${hub}: served=${served:-unknown} (${state}, not firing)\n"
        add_hub_json "$hub" "$served" "$floor" "$state"
        continue
    fi
    if [ -z "$served" ]; then
        FIRING_LINES="${FIRING_LINES}  ! ${hub}: version UNKNOWN, floor=${floor} — hub too old to report its version (pre-version-field binary)\n"
        FIRING_COUNT=$((FIRING_COUNT + 1))
        FIRING_JSON=$(echo "$FIRING_JSON" | jq -c --arg hub "$hub" --arg floor "$floor" \
            '. + [{hub:$hub, served:null, floor:$floor, reason:"version-unknown"}]')
        add_hub_json "$hub" "" "$floor" "FIRING"
        continue
    fi
    if version_lt "$served" "$floor"; then
        FIRING_LINES="${FIRING_LINES}  ! ${hub}: served=${served} < floor=${floor} — stale binary live (restart the hub with the upgraded binary)\n"
        FIRING_COUNT=$((FIRING_COUNT + 1))
        FIRING_JSON=$(echo "$FIRING_JSON" | jq -c --arg hub "$hub" --arg served "$served" --arg floor "$floor" \
            '. + [{hub:$hub, served:$served, floor:$floor, reason:"below-floor"}]')
        add_hub_json "$hub" "$served" "$floor" "FIRING"
    else
        INFO_LINES="${INFO_LINES}  ✓ ${hub}: served=${served} >= floor=${floor}\n"
        add_hub_json "$hub" "$served" "$floor" "ok"
    fi
done < <(echo "$DOCTOR_JSON" | jq -r '.hubs[] | [.hub, (.status // "error"), (.hub_version // "")] | @tsv')

# ── render ───────────────────────────────────────────────────────────────────
if [ "$FORMAT" = json ]; then
    jq -cn --argjson firing "$FIRING_JSON" --argjson hubs "$HUBS_JSON" \
        '{ok: ($firing | length == 0), firing: $firing, hubs: $hubs}'
    [ "$FIRING_COUNT" -eq 0 ] && exit 0 || exit 1
fi

if [ "$FIRING_COUNT" -gt 0 ]; then
    # Framed for the log (=== ts === ... ---) so /canaries and forensics can
    # split entries; the cron line appends this verbatim.
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "fleet-binary-freshness: FIRING — ${FIRING_COUNT} hub(s) below declared version floor"
    printf '%b' "$FIRING_LINES"
    [ "$QUIET" = 1 ] || printf '%b' "$INFO_LINES"
    echo "operator action: restart the named hub(s) onto the upgraded binary"
    echo "(systemd hosts: THROUGH the unit — stop any detached process, let systemd start; see G-070),"
    echo "or lower/exempt the floor in ${FLOORS_FILE} if the expectation changed."
    echo "---"
    exit 1
fi

if [ "$QUIET" = 0 ]; then
    echo "fleet-binary-freshness: healthy — all floored reachable hubs at/above floor"
    printf '%b' "$INFO_LINES"
fi
exit 0
