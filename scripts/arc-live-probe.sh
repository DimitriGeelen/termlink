#!/usr/bin/env bash
# arc-live-probe.sh (T-2480, P3a, G-069 "shipped == live" gate)
#
# WHY: capabilities were repeatedly recorded closed=shipped while dark in the
# field for weeks (arc-004 push-transport; .107/.122 stale binaries). The
# framework had DETECTION on a daily cron (T-2359 binary-floor / T-2387 waker /
# T-2415 capability canaries) but NO GATE at closure time and NO synchronous
# "make it live and confirm" probe. This verb is that missing primitive: it
# probes a LIVE hub and confirms a capability/version is ACTUALLY being served
# RIGHT NOW.
#
# HOW IT BECOMES A GATE: it exits non-zero when a capability is shipped-but-not-
# live. An arc-closing task drops one line into its `## Verification` block:
#
#     bash scripts/arc-live-probe.sh --hub <primary-addr> --min-version "$(cat VERSION)" --capability cv-keys
#
# The existing P-011 verification gate already runs `## Verification` commands
# before allowing `--status work-completed` and blocks on any non-zero exit, so
# closure is mechanically blocked until the fleet genuinely serves the capability.
# No AEF arc.sh change; reversible; no user-facing surface removed. The stronger
# mandatory-block inside AEF `arc_close` (T-2477 IW-1) stays a human decision.
#
# It reuses the exact probe invocations of the two capability canaries rather
# than reinventing them:
#   - version:    `termlink fleet doctor --json`  ->  per-hub `.hub_version`   (T-2359)
#   - capability: `termlink channel cv-keys <topic> --hub <addr> --json`       (T-2415)
#   - field:      a named non-null field on the hub's `fleet doctor` entry
#
# CLASSES / EXIT CODES (contract-stable):
#   0  live-confirmed        -- every asserted check passed against the live hub.
#   1  shipped-but-not-live  -- hub reachable but version < floor OR capability
#                              rejected/omitted: the G-069 firing class.
#   2  tooling-error         -- hub unreachable / JSON unparseable / jq missing.
#                              FAIL-CLOSED: a gate that failed open would re-admit
#                              the exact blindness it exists to close, so an
#                              un-probeable hub is NEVER reported live.
#
# USAGE:
#   arc-live-probe.sh --hub <addr|name> [--min-version X] [--capability CAP] [--json]
#
#   --hub          hub address ("host:port") or profile name to probe (required)
#   --min-version  assert the served hub_version is >= X (dotted-numeric compare)
#   --capability   assert a capability is served; one of:
#                    cv-keys        -- hub answers channel.cv_keys (doorbell prereq)
#                    field:<name>   -- named field present + non-null on the hub's
#                                      fleet-doctor entry (e.g. field:hub_version)
#   --json         emit {ok, live, hub, checks[], reason} instead of text
#   -h, --help     show this help
#
# At least one of --min-version / --capability must be given (nothing to gate on
# otherwise -> tooling error, exit 2).
#
# TEST HOOKS (PL-213 -- host-independent unit tests, see test-arc-live-probe.sh):
#   TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON=<file>  canned `fleet doctor --json`
#   TERMLINK_ARC_PROBE_TEST_CVKEYS_RC=<int>     canned cv-keys probe exit code
#   TERMLINK_ARC_PROBE_TEST_CVKEYS_OUT=<file>   canned cv-keys probe stdout
#
# Origin: T-2477 (P3 inception, GO'd) -> this build (part a). Sibling of the
# T-2415 fleet-capability canary (that finds incapable hubs on a schedule; this
# gates ONE arc closure synchronously, at the moment "shipped" is asserted).
set -uo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TERMLINK="${TERMLINK_BIN:-termlink}"
PROBE_TOPIC="${ARC_PROBE_TOPIC:-agent-presence}"
DOCTOR_TIMEOUT="${ARC_PROBE_DOCTOR_TIMEOUT:-180}"
CV_TIMEOUT="${ARC_PROBE_CV_TIMEOUT:-20}"

hub="" min_version="" capability="" want_json=0

die_setup() { # <msg>  -- tooling error, exit 2 (fail-closed)
    if [ "$want_json" -eq 1 ]; then
        jq -cn --arg e "$1" '{ok:false, live:false, reason:$e}' 2>/dev/null \
            || printf '{"ok":false,"live":false,"reason":"%s"}\n' "$1"
    else
        echo "arc-live-probe: $1" >&2
    fi
    exit 2
}

while [ $# -gt 0 ]; do
    case "$1" in
        --hub)         hub="${2:-}"; shift 2 ;;
        --min-version) min_version="${2:-}"; shift 2 ;;
        --capability)  capability="${2:-}"; shift 2 ;;
        --json)        want_json=1; shift ;;
        -h|--help)
            sed -n '2,64p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "arc-live-probe: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v jq >/dev/null 2>&1 || die_setup "jq not found (required)"
[ -n "$hub" ] || die_setup "--hub <addr|name> is required"
if [ -z "$min_version" ] && [ -z "$capability" ]; then
    die_setup "nothing to gate on: pass --min-version and/or --capability"
fi

# --- dotted-numeric version compare (mirrors T-2359 version_lt) --------------
# version_ge A B  -> true (0) iff A >= B. Strips leading 'v', per-segment numeric.
version_ge() {
    local a="${1#v}" b="${2#v}" IFS=.
    local -a av bv; read -r -a av <<<"$a"; read -r -a bv <<<"$b"
    local i len=${#av[@]}; [ ${#bv[@]} -gt "$len" ] && len=${#bv[@]}
    for ((i=0; i<len; i++)); do
        local x="${av[i]:-0}" y="${bv[i]:-0}"
        x="${x//[^0-9]/}"; y="${y//[^0-9]/}"; x="${x:-0}"; y="${y:-0}"
        if ((10#$x > 10#$y)); then return 0; fi
        if ((10#$x < 10#$y)); then return 1; fi
    done
    return 0
}

# --- read fleet doctor (test hook wins; else live) --------------------------
read_doctor_json() {
    if [ -n "${TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON:-}" ]; then
        cat "$TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON" 2>/dev/null || true
    else
        timeout "$DOCTOR_TIMEOUT" "$TERMLINK" fleet doctor --json 2>/dev/null || true
    fi
}

# Extract the hub entry matching --hub on either .address or .hub (name).
# Echoes the single JSON object, or empty if no match.
hub_entry() { # <doctor-json>
    printf '%s' "$1" | jq -c --arg h "$hub" \
        'if (.hubs|type)=="array" then (.hubs[] | select(.address==$h or .hub==$h)) else empty end' \
        2>/dev/null | head -n1
}

checks_json="[]"          # accumulates {check, ok, detail} objects
add_check() {             # <check> <ok:true|false> <detail>
    checks_json="$(printf '%s' "$checks_json" | jq -c --arg c "$1" --argjson ok "$2" --arg d "$3" \
        '. + [{check:$c, ok:$ok, detail:$d}]')"
}

fail_live() {  # <reason>  -- shipped-but-not-live, exit 1
    if [ "$want_json" -eq 1 ]; then
        jq -cn --arg hub "$hub" --argjson checks "$checks_json" --arg r "$1" \
            '{ok:true, live:false, hub:$hub, checks:$checks, reason:$r}'
    else
        echo "arc-live-probe: SHIPPED-BUT-NOT-LIVE (hub '$hub') — $1"
        echo "  ↳ 'shipped' does not yet mean live here (G-069). Roll the capability live"
        echo "    on this hub (rebuild + restart THROUGH systemd per G-070), then re-probe."
    fi
    exit 1
}

# --- version + field checks both need the doctor entry ----------------------
need_doctor=0
[ -n "$min_version" ] && need_doctor=1
case "$capability" in field:*) need_doctor=1 ;; esac

entry=""
if [ "$need_doctor" -eq 1 ]; then
    doctor_json="$(read_doctor_json)"
    [ -n "$doctor_json" ] || die_setup "could not read 'fleet doctor --json' (hub unreachable / tooling)"
    printf '%s' "$doctor_json" | jq -e '(.hubs|type)=="array"' >/dev/null 2>&1 \
        || die_setup "fleet doctor JSON has no .hubs array (unparseable)"
    entry="$(hub_entry "$doctor_json")"
    [ -n "$entry" ] || die_setup "hub '$hub' not present in fleet doctor output (not in hubs.toml?)"
fi

# --- version assertion -------------------------------------------------------
if [ -n "$min_version" ]; then
    served="$(printf '%s' "$entry" | jq -r '.hub_version // ""')"
    if [ -z "$served" ] || [ "$served" = "null" ]; then
        add_check "min-version" false "served=unknown floor=$min_version"
        fail_live "hub reports no version (too old to answer, or unreachable at probe time)"
    fi
    if version_ge "$served" "$min_version"; then
        add_check "min-version" true "served=$served floor=$min_version"
    else
        add_check "min-version" false "served=$served floor=$min_version"
        fail_live "served version $served is below floor $min_version"
    fi
fi

# --- capability assertion ----------------------------------------------------
if [ -n "$capability" ]; then
    case "$capability" in
        cv-keys)
            if [ -n "${TERMLINK_ARC_PROBE_TEST_CVKEYS_RC:-}" ]; then
                probe_rc="$TERMLINK_ARC_PROBE_TEST_CVKEYS_RC"
                probe_out="$(cat "${TERMLINK_ARC_PROBE_TEST_CVKEYS_OUT:-/dev/null}" 2>/dev/null || true)"
            else
                probe_out="$(timeout "$CV_TIMEOUT" "$TERMLINK" channel cv-keys "$PROBE_TOPIC" \
                                --hub "$hub" --json 2>&1)"; probe_rc=$?
            fi
            # capable: rc 0 AND a numeric .count (count 0 is healthy per T-2106)
            if [ "$probe_rc" -eq 0 ] && printf '%s' "$probe_out" | jq -e '.count | numbers' >/dev/null 2>&1; then
                cnt="$(printf '%s' "$probe_out" | jq -r '.count')"
                add_check "capability:cv-keys" true "count=$cnt"
            else
                case "$probe_out" in
                    *-32001*|*-32601*|*"Method not found"*|*"method not found"*|*"Unknown method"*)
                        add_check "capability:cv-keys" false "rpc-rejected (pre-cv_index binary)"
                        fail_live "hub rejects channel.cv_keys — binary predates the doorbell discovery primitive (T-2103)" ;;
                    *)
                        die_setup "cv-keys probe inconclusive (rc=$probe_rc, timeout/network) — cannot confirm live" ;;
                esac
            fi ;;
        field:*)
            field="${capability#field:}"
            [ -n "$field" ] || die_setup "--capability field: requires a field name (e.g. field:hub_version)"
            present="$(printf '%s' "$entry" | jq -r --arg f "$field" 'has($f) and (.[$f] != null)')"
            if [ "$present" = "true" ]; then
                val="$(printf '%s' "$entry" | jq -r --arg f "$field" '.[$f]')"
                add_check "capability:field:$field" true "value=$val"
            else
                add_check "capability:field:$field" false "field absent/null"
                fail_live "hub's fleet-doctor entry has no non-null '$field' (capability not served)"
            fi ;;
        *)
            die_setup "unknown --capability '$capability' (expected: cv-keys | field:<name>)" ;;
    esac
fi

# --- all checks passed -> live-confirmed ------------------------------------
if [ "$want_json" -eq 1 ]; then
    jq -cn --arg hub "$hub" --argjson checks "$checks_json" \
        '{ok:true, live:true, hub:$hub, checks:$checks, reason:"live-confirmed"}'
else
    echo "arc-live-probe: LIVE-CONFIRMED (hub '$hub') — every asserted capability is served."
    printf '%s' "$checks_json" | jq -r '.[] | "  ↳ \(.check): \(.detail)"'
fi
exit 0
