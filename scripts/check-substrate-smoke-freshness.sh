#!/usr/bin/env bash
# T-2696 — substrate-smoke canary (verb-3 "claim work" END-TO-END composition).
#
# scripts/substrate-smoke.sh (T-2151) proves the substrate's canonical
# work-stealing composition (create → post → claim → claim-transfer →
# worker-loop → verify-clean + the four arc-demo regression gates) in one
# command — and NOTHING executed it on a schedule. The stuck-claims canary
# (T-2556) watches claim STATE; this canary proves the claim COMPOSITION
# still round-trips. T-2557-pattern verdict translation:
#   smoke exit 0 (healthy) -> canary exit 0
#   smoke exit 1 (a stage FAILed) -> canary exit 1 (FIRE — names the stage)
#   smoke exit 2 (usage / missing dep) -> canary exit 2 (tooling, non-firing)
#
# THE LOAD-BEARING REMAP (T-2694 F2/G3): smoke's `create` stage on an
# UNREACHABLE hub is a stage_fail -> smoke exits 1 — i.e. the prover reports
# "substrate BROKEN" on a quiet/hub-down host. A canary that translated that
# 1 -> FIRE would fill the log on every hub blip and teach its operator to
# stop reading it (the T-2818 fatigue class). So this canary prechecks hub
# reachability itself (cheap `channel list` read) BEFORE invoking smoke and
# exits 2 (non-firing, /preflight territory) when the hub is unreachable.
# Smoke's own exit-2 arm covers only usage/missing-dep and cannot make this
# distinction for us.
#
# Empty log = healthy — same convention as the other canaries (CLAUDE.md).
# Exit codes: 0 healthy · 1 firing (composition broken) · 2 tooling error
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SMOKE="${SUBSTRATE_SMOKE_BIN:-scripts/substrate-smoke.sh}"

QUIET=0
FORMAT=human
HUB=""
HEARTBEAT=1
HEARTBEAT_FILE=".context/working/.substrate-smoke-canary.heartbeat"

usage() {
    sed -n '2,26p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: check-substrate-smoke-freshness.sh [OPTIONS]
  --hub ADDR           Pass through to substrate-smoke.sh + the reachability precheck
  --json               Emit a JSON envelope
  --quiet              Print only on firing (cron-friendly)
  --no-heartbeat       Skip touching the heartbeat companion
  -h, --help           This help

Test hooks (PL-213 — hub-independent verification):
  TERMLINK_SMOKE_CANARY_TEST_HUB_RC=<n>   canned reachability-precheck rc (0 = reachable)
  TERMLINK_SMOKE_CANARY_TEST_JSON=<file>  canned substrate-smoke --json verdict
  TERMLINK_SMOKE_CANARY_TEST_RC=<n>       canned substrate-smoke exit code

Exit: 0 healthy · 1 firing (work-stealing composition broken) · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --hub)          HUB="${2:-}"; shift 2 ;;
        --json)         FORMAT=json; shift ;;
        --quiet)        QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        -h|--help)      usage; exit 0 ;;
        *) echo "check-substrate-smoke: unknown arg: $1" >&2; exit 2 ;;
    esac
done

# Heartbeat FIRST (before the check) so /canaries can prove the canary ran even
# on a healthy cycle — mirrors the T-2290/T-2295/T-2556/T-2557 convention.
if [ "$HEARTBEAT" -eq 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null && date -u +%Y-%m-%dT%H:%M:%SZ > "$HEARTBEAT_FILE" 2>/dev/null || true
fi

emit() { # <state> <rc> <broken_stage> <detail>
    local state="$1" rc="$2" stage="$3" detail="$4"
    if [ "$FORMAT" = json ]; then
        printf '{"ok":%s,"state":"%s","smoke_rc":%s,"broken_stage":%s,"detail":%s}\n' \
            "$([ "$rc" -eq 0 ] && echo true || echo false)" \
            "$state" "$rc" \
            "$(printf '%s' "$stage" | jq -R . 2>/dev/null || echo '"unknown"')" \
            "$(printf '%s' "$detail" | jq -R . 2>/dev/null || echo '""')"
    fi
}

# ---- Reachability precheck (the F2/G3 remap) ------------------------------
if [ -n "${TERMLINK_SMOKE_CANARY_TEST_HUB_RC:-}" ]; then
    hub_rc="${TERMLINK_SMOKE_CANARY_TEST_HUB_RC}"
else
    probe_args=(channel list --json)
    [ -n "$HUB" ] && probe_args+=(--hub "$HUB")
    timeout 15 "$TERMLINK" "${probe_args[@]}" >/dev/null 2>&1
    hub_rc=$?
fi
if [ "$hub_rc" -ne 0 ]; then
    emit tooling 2 "n/a" "hub unreachable (precheck rc=$hub_rc)"
    if [ "$FORMAT" != json ] && [ "$QUIET" -eq 0 ]; then
        echo "check-substrate-smoke: tooling — hub unreachable (precheck rc=$hub_rc). NOT a substrate regression; /preflight territory. Smoke was not invoked." >&2
    fi
    exit 2
fi

# ---- Run the prover (test hooks short-circuit for hub-independence) -------
if [ -n "${TERMLINK_SMOKE_CANARY_TEST_JSON:-}" ] || [ -n "${TERMLINK_SMOKE_CANARY_TEST_RC:-}" ]; then
    verdict="$(cat "${TERMLINK_SMOKE_CANARY_TEST_JSON:-/dev/null}" 2>/dev/null)"
    rc="${TERMLINK_SMOKE_CANARY_TEST_RC:-0}"
else
    args=(--json)
    [ -n "$HUB" ] && args+=(--hub "$HUB")
    verdict="$(bash "$SMOKE" "${args[@]}" 2>/dev/null)"
    rc=$?
fi

broken_stage="$(printf '%s' "$verdict" | jq -r '(.stages_failed // [])[0] // "unknown"' 2>/dev/null || echo unknown)"
errors="$(printf '%s' "$verdict" | jq -r '.errors // ""' 2>/dev/null || echo "")"

case "$rc" in
    0)
        emit healthy 0 "" ""
        if [ "$FORMAT" != json ] && [ "$QUIET" -eq 0 ]; then
            echo "check-substrate-smoke: healthy (verb-3 work-stealing composition proven end-to-end)"
        fi
        exit 0 ;;
    1)
        emit firing 1 "$broken_stage" "$errors"
        if [ "$FORMAT" != json ]; then
            echo "check-substrate-smoke: verb-3 work-stealing composition BROKEN at stage '${broken_stage}' (T-2696)"
            echo "  Hub was reachable at precheck, so this is a genuine substrate regression, not a blip."
            echo "  Reproduce: bash scripts/substrate-smoke.sh   (names the broken stage)"
            echo "  Then inspect: /claims --all · termlink channel claims-summary --all --only-stuck"
        fi
        exit 1 ;;
    *)
        emit tooling "$rc" "n/a" "smoke rc=$rc (usage/missing dep)"
        if [ "$FORMAT" != json ] && [ "$QUIET" -eq 0 ]; then
            echo "check-substrate-smoke: tooling error (smoke rc=$rc — usage or missing dependency; not a substrate regression)." >&2
        fi
        exit 2 ;;
esac
