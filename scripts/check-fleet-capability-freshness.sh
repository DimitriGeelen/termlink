#!/usr/bin/env bash
# T-2415 — Fleet capability-freshness canary (G-084 prevention).
#
# SIBLING to the T-2359 fleet-binary-freshness canary, answering the question a
# version FLOOR provably cannot. G-084: ring20-dashboard (.121) passes `fleet
# doctor` in 42ms and is version-floor-EXEMPT (`ring20-dashboard -`), so the
# binary-freshness canary reports "healthy" — while the hub CANNOT serve
# `channel.cv_keys` (its binary predates cv_index, T-2103). Without cv_index every
# agent-presence read walks the full backlog and times out, so .121 is invisible
# to /peers, find-idle, and the `agent contact` reachability preflight:
# structurally excluded from the doorbell, with nothing firing.
#
# WHY A CAPABILITY PROBE, NOT A VERSION FLOOR. .121's version (0.11.806) is a
# `git describe` tag-epoch artifact — patch numbers are NOT comparable across
# build lineages/tag epochs (T-2377: .121 is likely a ~1050-commit-STALE build of
# our OWN mainline, not a fork). A floor cannot distinguish "newer" from "older
# lineage", which is exactly why the hub was exempted, which is exactly why the
# framework went blind. "Does this hub answer channel.cv_keys?" is
# lineage-independent, tag-epoch-independent, and decisive — it answers the
# question the operator actually has ("can this hub carry the doorbell?") instead
# of a proxy ("is its integer bigger?"). Capability probing settles what version
# comparison cannot. Version-floor EXEMPTION does NOT exempt a hub here: the two
# instruments are orthogonal by design.
#
# The probed capability is the doorbell's discovery prerequisite:
#   termlink channel cv-keys agent-presence --hub <addr> --json
#     exit 0 + parseable {count:N}                 → CAPABLE (count 0 is healthy,
#                                                     T-2106: empty cv_index is not
#                                                     an error)
#     RPC rejection (-32001 / -32601 / method not  → INCAPABLE → FIRE (binary
#       found / Missing 'target')                     predates cv_index)
#     network-ish failure (timeout / no route)     → inconclusive (informational —
#                                                     a different failure mode;
#                                                     fleet doctor owns reachability)
#
# Firing semantics:
#   - reachable + probe rejected by hub  → FIRE (doorbell-incapable binary)
#   - reachable + probe ok               → healthy
#   - reachable + probe inconclusive     → informational (transient / net)
#   - unreachable/down                   → informational (PL-219 — fleet doctor
#                                          already surfaces down hubs)
#   - capability-exempt (operator opt-out via FLEET_CAP_EXEMPT) → informational
#
# Exit codes:
#   0 — all reachable hubs serve the capability (or are inconclusive/exempt)
#   1 — at least one reachable hub is doorbell-incapable
#   2 — tooling error (fleet doctor unrunnable, jq missing)
#
# Usage:
#   check-fleet-capability-freshness.sh            # human-readable, one-shot
#   check-fleet-capability-freshness.sh --quiet    # print only on firing (cron)
#   check-fleet-capability-freshness.sh --json     # {ok, firing[], hubs[]}
#   check-fleet-capability-freshness.sh --no-heartbeat
#
# Operator opt-out for a hub that legitimately need not serve the doorbell:
#   FLEET_CAP_EXEMPT="hub-a,hub-b"  (comma-separated hub NAMES; distinct from the
#   version-floor exemption — this is an explicit "not a doorbell participant"
#   declaration, not a lineage judgement).
#
# Test hooks (PL-213 — hub-independent verification):
#   TERMLINK_FLEET_CAP_DOCTOR_JSON=<file>  canned `fleet doctor --json` (hub list
#                                          + reachability)
#   TERMLINK_FLEET_CAP_PROBE_DIR=<dir>     per-hub canned probe result: for hub
#                                          address ADDR, slug = ADDR with :/. → _;
#                                          <dir>/<slug>.rc holds the exit code and
#                                          <dir>/<slug>.out holds stdout. Absent
#                                          files → inconclusive.
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
DOCTOR_TIMEOUT="${FLEET_CAP_DOCTOR_TIMEOUT:-180}"
PROBE_TIMEOUT="${FLEET_CAP_PROBE_TIMEOUT:-20}"
PROBE_TOPIC="${FLEET_CAP_PROBE_TOPIC:-agent-presence}"

FORMAT=human
QUIET=0
HEARTBEAT=1

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        -h|--help) sed -n '2,70p' "$0"; exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

die_setup() {
    if [ "$FORMAT" = json ]; then
        printf '{"ok":false,"error":"%s"}\n' "$1"
    else
        echo "fleet-capability-freshness: SETUP-FAIL — $1" >&2
    fi
    exit 2
}

# T-1723 heartbeat: prove this canary ran, even on healthy/error cycles.
# Placed BEFORE the network call so a fleet-doctor hang still leaves a beat.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.fleet-capability-canary.heartbeat}"
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

command -v jq >/dev/null 2>&1 || die_setup "jq not found"

# ── is this hub name capability-exempt? ──────────────────────────────────────
is_exempt() { # $1 = hub name ; reads FLEET_CAP_EXEMPT live (env, comma-separated)
    local h; local IFS=,
    for h in ${FLEET_CAP_EXEMPT:-}; do
        [ "$h" = "$1" ] && return 0
    done
    return 1
}

# ── classify a raw probe result → capable | incapable | inconclusive ─────────
# Pure: (rc, stdout) → verdict. The heart of the canary; unit-tested directly.
classify_probe() { # $1 = rc, $2 = stdout
    local rc="$1" out="$2"
    if [ "$rc" -eq 0 ] && printf '%s' "$out" | jq -e '.count | numbers' >/dev/null 2>&1; then
        printf 'capable'; return
    fi
    # A hub that REJECTS the RPC is doorbell-incapable. Match the JSON-RPC
    # rejection signatures a pre-cv_index binary produces, NOT generic network
    # failure (which fleet doctor already owns as reachability).
    case "$out" in
        *-32001*|*-32601*|*"Method not found"*|*"Missing 'target'"*|*"method not found"*|*"Unknown method"*)
            printf 'incapable'; return ;;
    esac
    printf 'inconclusive'
}

# ── raw probe (live) or fixture (test) ───────────────────────────────────────
addr_slug() { printf '%s' "$1" | tr ':./' '___'; }

probe_raw() { # $1 = addr ; sets PROBE_RC, PROBE_OUT
    local addr="$1" slug
    if [ -n "${TERMLINK_FLEET_CAP_PROBE_DIR:-}" ]; then
        slug=$(addr_slug "$addr")
        if [ -r "$TERMLINK_FLEET_CAP_PROBE_DIR/$slug.rc" ]; then
            PROBE_RC=$(cat "$TERMLINK_FLEET_CAP_PROBE_DIR/$slug.rc")
            PROBE_OUT=$(cat "$TERMLINK_FLEET_CAP_PROBE_DIR/$slug.out" 2>/dev/null || printf '')
        else
            PROBE_RC=124; PROBE_OUT=""   # absent fixture → inconclusive
        fi
        return
    fi
    PROBE_OUT=$(timeout "$PROBE_TIMEOUT" "$TERMLINK" channel cv-keys "$PROBE_TOPIC" \
                    --hub "$addr" --json 2>&1)
    PROBE_RC=$?
}

# Lib mode (FLEET_CAP_LIB=1): stop here so a test can source the file and call
# the pure helpers (classify_probe / is_exempt / addr_slug) without running the
# fleet walk. Mirrors TL_CLAUDE_LIB=1 in tl-claude.sh.
[ -n "${FLEET_CAP_LIB:-}" ] && return 0 2>/dev/null

# ── acquire fleet doctor JSON (hub list + reachability) ──────────────────────
if [ -n "${TERMLINK_FLEET_CAP_DOCTOR_JSON:-}" ]; then
    [ -r "$TERMLINK_FLEET_CAP_DOCTOR_JSON" ] || die_setup "test doctor JSON not readable"
    DOCTOR_JSON=$(cat "$TERMLINK_FLEET_CAP_DOCTOR_JSON")
else
    command -v "$TERMLINK" >/dev/null 2>&1 || die_setup "termlink not on PATH"
    DOCTOR_JSON=$(timeout "$DOCTOR_TIMEOUT" "$TERMLINK" fleet doctor --json 2>/dev/null) || true
fi
echo "$DOCTOR_JSON" | jq -e '.hubs | type == "array"' >/dev/null 2>&1 \
    || die_setup "fleet doctor --json produced no parseable .hubs[]"

# ── walk hubs ────────────────────────────────────────────────────────────────
FIRING_LINES=""
INFO_LINES=""
FIRING_JSON="[]"
HUBS_JSON="[]"
FIRING_COUNT=0

add_hub_json() { # name addr state
    HUBS_JSON=$(echo "$HUBS_JSON" | jq -c \
        --arg hub "$1" --arg addr "$2" --arg state "$3" \
        '. + [{hub:$hub, address:($addr|select(.!="")), state:$state}]')
}

while IFS=$'\t' read -r hub status addr; do
    if [ "$status" != "ok" ]; then
        INFO_LINES="${INFO_LINES}  ~ ${hub}: unreachable (not firing — fleet doctor surfaces down hubs)\n"
        add_hub_json "$hub" "$addr" "unreachable"
        continue
    fi
    if is_exempt "$hub"; then
        INFO_LINES="${INFO_LINES}  ~ ${hub}: capability-exempt (FLEET_CAP_EXEMPT, not firing)\n"
        add_hub_json "$hub" "$addr" "exempt"
        continue
    fi
    if [ -z "$addr" ]; then
        INFO_LINES="${INFO_LINES}  ~ ${hub}: no address in fleet doctor output (cannot probe)\n"
        add_hub_json "$hub" "$addr" "no-address"
        continue
    fi
    probe_raw "$addr"
    verdict=$(classify_probe "$PROBE_RC" "$PROBE_OUT")
    case "$verdict" in
        capable)
            INFO_LINES="${INFO_LINES}  ✓ ${hub}: serves channel.cv_keys (doorbell-capable)\n"
            add_hub_json "$hub" "$addr" "ok" ;;
        incapable)
            FIRING_LINES="${FIRING_LINES}  ! ${hub} (${addr}): CANNOT serve channel.cv_keys — binary predates cv_index (T-2103); doorbell-incapable\n"
            FIRING_COUNT=$((FIRING_COUNT + 1))
            FIRING_JSON=$(echo "$FIRING_JSON" | jq -c --arg hub "$hub" --arg addr "$addr" \
                '. + [{hub:$hub, address:$addr, capability:"channel.cv_keys", reason:"rpc-rejected"}]')
            add_hub_json "$hub" "$addr" "FIRING" ;;
        *)
            INFO_LINES="${INFO_LINES}  ~ ${hub}: cv_keys probe inconclusive (timeout/network — not firing)\n"
            add_hub_json "$hub" "$addr" "inconclusive" ;;
    esac
done < <(echo "$DOCTOR_JSON" | jq -r '.hubs[] | [.hub, (.status // "error"), (.address // "")] | @tsv')

# ── render ───────────────────────────────────────────────────────────────────
if [ "$FORMAT" = json ]; then
    jq -cn --argjson firing "$FIRING_JSON" --argjson hubs "$HUBS_JSON" \
        '{ok: ($firing | length == 0), firing: $firing, hubs: $hubs}'
    [ "$FIRING_COUNT" -eq 0 ] && exit 0 || exit 1
fi

if [ "$FIRING_COUNT" -gt 0 ]; then
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "fleet-capability-freshness: FIRING — ${FIRING_COUNT} reachable hub(s) doorbell-incapable"
    printf '%b' "$FIRING_LINES"
    [ "$QUIET" = 1 ] || printf '%b' "$INFO_LINES"
    echo "operator action: upgrade the named hub(s) to a binary that serves cv_index (T-2103)"
    echo "and restart THROUGH the systemd unit (G-070); needs a foothold on that host."
    echo "If a hub legitimately is not a doorbell participant, exempt it:"
    echo "  FLEET_CAP_EXEMPT=<hub-name> (distinct from the version-floor exemption)."
    echo "---"
    exit 1
fi

if [ "$QUIET" = 0 ]; then
    echo "fleet-capability-freshness: healthy — all reachable hubs serve the doorbell capability"
    printf '%b' "$INFO_LINES"
fi
exit 0
