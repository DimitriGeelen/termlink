#!/usr/bin/env bash
# T-2556 (T-2468 charter-verb completeness) — stuck-claims canary.
#
# TermLink's charter names FOUR core verbs; verb 3 is "claim work" (the substrate
# work-coordination primitive: channel claim → renew → release with lease-based
# ownership, T-2019/T-2029/T-2042/T-2046). It has the RICHEST affirmative prover
# (scripts/substrate-smoke.sh drives the full claim→renew→release lifecycle with
# ownership enforcement) but — until this canary — ZERO passive daily detection.
# A claim can sit expired, or a work-topic can accumulate stuck claims, for days
# with nothing firing: an idle worker's slot never reopens, or a crashed worker's
# lease never gets noticed. That is the same "detection gap on a charter-core verb"
# class the discover/exchange verbs already close (waker-liveness / unconfirmed-
# delivery canaries).
#
# The substrate already ships the detector primitive: `channel claims-summary --all
# --only-stuck --json` (T-2076) computes stuckness via the T-2042 heuristic
# (a lease lapsed within the last 15min OR oldest_active_age_ms > 60_000) and
# returns a truthful fleet-wide `stuck_count`. This canary wraps that: it FIRES
# (exit 1) when stuck_count > 0, turning a silently-stuck work-topic into a
# daily alert. Empty log = healthy — the same convention as the other canaries
# (CLAUDE.md).
#
# T-2709: the first arm used to be `expired_count > 0`, which could never clear.
# Expired claim rows are reaped only when the SAME (topic, offset) is
# re-claimed, so on an abandoned topic the row — and the "stuck" verdict —
# persisted for the life of the hub's SQLite. This canary consequently fired
# daily on 11 topics whose leases had lapsed ~62 days earlier, every one with
# active_count 0 (nothing held, nothing that could BE stuck). That is worse
# than a missing guard: a canary that fires every day regardless of system
# state trains its operator to stop reading it, so it also costs you the one
# firing that mattered. No change was needed here — the gate reads `stuck_count`
# from the CLI, so it inherits the corrected predicate — but the fix is noted
# because this script's own header taught the wrong rule.
#
# Exit codes: 0 healthy (no stuck claims) · 1 firing (>=1 stuck topic) · 2 tooling error
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"

QUIET=0
FORMAT=human
HUB=""
HEARTBEAT=1
HEARTBEAT_FILE=".context/working/.stuck-claims-canary.heartbeat"

usage() {
    sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: check-stuck-claims-freshness.sh [OPTIONS]
  --hub ADDR           Target a specific hub (default: local hub)
  --json               Emit a JSON envelope
  --quiet              Print only on firing (cron-friendly)
  --no-heartbeat       Skip touching the heartbeat companion
  -h, --help           This help

Test hook: TERMLINK_STUCK_CLAIMS_TEST_JSON=<file> feeds canned `channel
claims-summary --all --only-stuck --json` output for hub-independent testing.

Exit: 0 healthy · 1 firing (stuck claims present) · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --hub)          HUB="${2:-}"; shift 2 ;;
        --json)         FORMAT=json; shift ;;
        --quiet)        QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        -h|--help)      usage; exit 0 ;;
        *) echo "check-stuck-claims: unknown arg: $1" >&2; exit 2 ;;
    esac
done

# Heartbeat FIRST (before any check) so /canaries can prove the canary ran even
# on a healthy cycle — mirrors the T-2290/T-2295 convention.
if [ "$HEARTBEAT" -eq 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null && date -u +%Y-%m-%dT%H:%M:%SZ > "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# Fetch the fleet-wide stuck-claims summary (test hook short-circuits the CLI for
# hub-independence). --all sweeps every topic; --only-stuck keeps stuck_count
# truthful while filtering the topics[] array down to the actionable subset.
if [ -n "${TERMLINK_STUCK_CLAIMS_TEST_JSON:-}" ]; then
    raw="$(cat "${TERMLINK_STUCK_CLAIMS_TEST_JSON}" 2>/dev/null)"
    rc=$?
else
    args=(channel claims-summary --all --only-stuck --json)
    [ -n "$HUB" ] && args+=(--hub "$HUB")
    raw="$("$TERMLINK" "${args[@]}" 2>/dev/null)"
    rc=$?
fi
if [ "$rc" -ne 0 ] || [ -z "$raw" ]; then
    echo "check-stuck-claims: could not read claims-summary (exit=$rc); hub down or binary too old?" >&2
    exit 2
fi

# stuck_count is the firing gate (T-2076/T-2042 heuristic, computed hub-side
# before the --only-stuck filter). Validate the envelope parses.
stuck_count="$(printf '%s' "$raw" | jq -r '.stuck_count // empty' 2>/dev/null)"
jq_rc=$?
if [ "$jq_rc" -ne 0 ] || [ -z "$stuck_count" ]; then
    echo "check-stuck-claims: malformed claims-summary JSON (no stuck_count field)" >&2
    exit 2
fi
topic_count="$(printf '%s' "$raw" | jq -r '.topic_count // 0' 2>/dev/null)"

# Per-topic fetch errors (ok:false entries) are surfaced but do NOT themselves
# fire — they could mask a stuck topic, so we note them as a soft warning.
fetch_errors="$(printf '%s' "$raw" | jq -r '[.topics[]? | select(.ok == false)] | length' 2>/dev/null)"
[ -z "$fetch_errors" ] && fetch_errors=0

# One TSV line per stuck topic: topic\tactive\texpired\toldest_age_ms.
stuck_rows="$(printf '%s' "$raw" | jq -r '
    .topics[]?
    | select(.potentially_stuck == true)
    | [.topic, (.active_count|tostring), (.expired_count|tostring), ((.oldest_active_age_ms // 0)|tostring)] | @tsv
' 2>/dev/null)"

if [ "$FORMAT" = json ]; then
    stuck_json="$(printf '%s' "$raw" | jq -c '[.topics[]? | select(.potentially_stuck == true)
        | {topic, active_count, expired_count, oldest_active_age_ms}]' 2>/dev/null)"
    printf '{"ok":%s,"topic_count":%s,"stuck_count":%s,"fetch_errors":%s,"stuck":%s}\n' \
        "$([ "$stuck_count" -eq 0 ] && echo true || echo false)" \
        "${topic_count:-0}" "$stuck_count" "$fetch_errors" "${stuck_json:-[]}"
    [ "$stuck_count" -eq 0 ] && exit 0 || exit 1
fi

if [ "$stuck_count" -eq 0 ]; then
    note=""
    [ "$fetch_errors" -gt 0 ] && note=" (warning: $fetch_errors topic(s) unreadable — could mask a stuck topic)"
    [ "$QUIET" -eq 1 ] || echo "check-stuck-claims: healthy (${topic_count:-0} topics, 0 stuck)${note}"
    exit 0
fi

echo "check-stuck-claims: $stuck_count topic(s) with stuck/expired claims (verb-3 claim-work detection, T-2556):"
printf '%s\n' "$stuck_rows" | while IFS=$'\t' read -r topic active expired age; do
    [ -z "$topic" ] && continue
    echo "  $topic  active=$active expired=$expired oldest_active_age=${age}ms"
done
[ "$fetch_errors" -gt 0 ] && echo "  (+ $fetch_errors topic(s) unreadable — could mask additional stuck topics)"
echo "  Remediation: a claim is expired or a slot never reopened. Inspect with"
echo "  \`/claims <topic>\` (or \`termlink channel claims-summary <topic>\`); if the"
echo "  holder is dead, the lease will auto-expire — or reassign via \`/claim-transfer\`,"
echo "  or force-release (Tier-0) via \`termlink channel claim-force-release\`."
exit 1
