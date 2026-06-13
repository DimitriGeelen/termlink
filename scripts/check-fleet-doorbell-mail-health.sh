#!/usr/bin/env bash
# T-1831 — Fleet-wide doorbell+mail runtime health canary.
#
# Closes T-1829 Recommendation #1: persistent observability for the
# doorbell+mail loop (T-1800 arc). Reads every profile in
# ~/.termlink/hubs.toml, runs the T-1829 loopback selftest against
# each hub's address, and reports per-hub verdict. Empty log = healthy
# fleet (same convention as the release-mirror canary, T-1696).
#
# Becomes load-bearing when T-1830 adoption push lands — runtime drift
# must be visible BEFORE real conversations break.
#
# Exit codes:
#   0  — every reachable profile passed
#   1  — drift (any verdict != pass, or unreachable)
#   2  — tooling error (hubs.toml missing, jq missing, selftest verb missing)
#
# Usage:
#   check-fleet-doorbell-mail-health.sh                 # human-readable, one-shot
#   check-fleet-doorbell-mail-health.sh --json          # JSON envelope for scripting
#   check-fleet-doorbell-mail-health.sh --quiet         # only print on drift (cron-friendly)
#   check-fleet-doorbell-mail-health.sh --hubs-file P   # override default ~/.termlink/hubs.toml
#   check-fleet-doorbell-mail-health.sh --no-heartbeat  # suppress heartbeat touch
#   check-fleet-doorbell-mail-health.sh --transient-file P  # override transient-host declaration file (T-2225)
set -u

FORMAT=human
QUIET=0
HEARTBEAT=1
HUBS_FILE="${HOME}/.termlink/hubs.toml"
SELFTEST="${SELFTEST:-scripts/agent-conversation-selftest.sh}"
# T-2225 — operator-declared "expected-transient" hosts. A profile NAME listed
# here that is unreachable/setup-fail is classified transient_skipped and does
# NOT flip overall_ok (a sleeping laptop must not DRIFT the whole-fleet canary —
# G-019 alert-fatigue prevention). Sources merged: the declaration file (one
# profile name per line, # comments) UNION the FLEET_DM_CANARY_TRANSIENT env var
# (comma-separated). Match is by profile name. A transient host that is REACHABLE
# still counts pass/fail normally — the skip suppresses down-ness, not brokenness.
TRANSIENT_FILE="${FLEET_DM_CANARY_TRANSIENT_FILE:-.context/cron/fleet-dm-canary-transient}"

# T-1845 / PL-189 — per-hub bound. The selftest internally wraps each
# termlink RPC with `timeout 8` (PER_CALL_TIMEOUT in selftest), but the
# selftest itself runs 4 RPCs sequentially plus a status verb. 30s is
# safe upper-bound for the whole sweep against one hub. Exit 124 from
# timeout(1) means "selftest produced no JSON" → treat as unreachable.
PER_HUB_TIMEOUT="${FLEET_DM_CANARY_TIMEOUT:-30}"
if command -v timeout >/dev/null 2>&1; then
    TIMEOUT_CMD="timeout $PER_HUB_TIMEOUT"
else
    TIMEOUT_CMD=""
fi

die() {
    if [ "$FORMAT" = json ]; then
        printf '{"ok":false,"error":"%s"}\n' "$1"
    else
        echo "fleet-doorbell-mail-health: $1" >&2
    fi
    exit 2
}

usage() {
    sed -n '2,24p' "$0"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json)          FORMAT=json; shift ;;
        --quiet)         QUIET=1; shift ;;
        --no-heartbeat)  HEARTBEAT=0; shift ;;
        --hubs-file)     HUBS_FILE="${2:-}"; shift 2 ;;
        --transient-file) TRANSIENT_FILE="${2:-}"; shift 2 ;;
        -h|--help)       usage; exit 0 ;;
        *)               echo "unknown arg: $1 (try --help)" >&2; exit 2 ;;
    esac
done

# Heartbeat: prove this canary ran. Placed BEFORE network so a network
# error still leaves a heartbeat. Mirror of T-1723 pattern.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.fleet-doorbell-mail-canary.heartbeat}"
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch -- "$HEARTBEAT_FILE" 2>/dev/null || true
fi

command -v jq >/dev/null 2>&1 || die "jq not in PATH"
[ -f "$HUBS_FILE" ] || die "hubs file not found: $HUBS_FILE"
[ -x "$SELFTEST" ] || die "selftest verb not executable: $SELFTEST"

# Parse hubs.toml — minimal TOML-aware enough for `[hubs.NAME]` + `address = "..."`.
# (Profiles only need name+address for this canary. secret_file is handled by the
# binary via the running session's identity; selftest creates its own ephemeral
# topic, so per-profile secrets are NOT required here.)
current_name=""
declare -a profile_names=()
declare -a profile_addrs=()

while IFS= read -r raw_line || [ -n "$raw_line" ]; do
    # Strip CR + leading/trailing whitespace + comments.
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
        current_name=""  # consume — only one address per section
    fi
done < "$HUBS_FILE"

total_raw="${#profile_names[@]}"
[ "$total_raw" -gt 0 ] || die "no profiles found in $HUBS_FILE"

# T-1892 hub-identity dedup. Two profiles can list the same physical hub
# under different addresses (canonical: workstation-107-public at
# 192.168.10.107:9100 AND local-test at 127.0.0.1:9100 → same hub bound to
# 0.0.0.0:9100). Without dedup the canary would run the selftest against
# that hub twice, double-counting pass/fail tallies and producing
# misleading G-060 alert deltas. Probe each address; first profile name
# per TLS fingerprint wins.
_self_script="${BASH_SOURCE[0]}"
_self_libdir="$(cd "$(dirname "$_self_script")" && pwd)/lib"
# shellcheck source=/dev/null
. "$_self_libdir/hubs-toml-walk.sh"
_tsv_in=""
for i in "${!profile_names[@]}"; do
    _tsv_in+="${profile_addrs[$i]}"$'\t'"${profile_names[$i]}"$'\n'
done
# T-2225 — in --quiet (cron) mode, suppress the dedup helper's informational
# "skipping duplicate" stderr chatter so a HEALTHY run stays truly silent. The
# cron redirects `--quiet >> log 2>&1`, so any stderr would pollute the log and
# break the "empty log = healthy" contract (and re-fire /canaries) even when the
# fleet is fine. The helper emits no error-level stderr, only this one line.
if [ "$QUIET" = 1 ]; then
    _tsv_out="$(printf '%s' "$_tsv_in" | dedup_addrs_by_fp check-fleet-doorbell-mail-health 2>/dev/null)"
else
    _tsv_out="$(printf '%s' "$_tsv_in" | dedup_addrs_by_fp check-fleet-doorbell-mail-health)"
fi
declare -a _kept_names=()
declare -a _kept_addrs=()
while IFS=$'\t' read -r _kept_addr _kept_name; do
    [ -n "$_kept_addr" ] || continue
    _kept_addrs+=("$_kept_addr")
    _kept_names+=("$_kept_name")
done <<< "$_tsv_out"
profile_addrs=("${_kept_addrs[@]}")
profile_names=("${_kept_names[@]}")

total="${#profile_names[@]}"
[ "$total" -gt 0 ] || die "no profiles found in $HUBS_FILE (after dedup)"

# T-2225 — load the expected-transient profile-name set (file UNION env).
declare -A TRANSIENT_SET=()
if [ -n "${TRANSIENT_FILE:-}" ] && [ -f "$TRANSIENT_FILE" ]; then
    while IFS= read -r t_line || [ -n "$t_line" ]; do
        t_line="${t_line%$'\r'}"; t_line="${t_line%%#*}"
        t_line="${t_line#"${t_line%%[![:space:]]*}"}"; t_line="${t_line%"${t_line##*[![:space:]]}"}"
        [ -z "$t_line" ] && continue
        TRANSIENT_SET["$t_line"]=1
    done < "$TRANSIENT_FILE"
fi
if [ -n "${FLEET_DM_CANARY_TRANSIENT:-}" ]; then
    IFS=',' read -ra _env_transient <<< "$FLEET_DM_CANARY_TRANSIENT"
    for _t in "${_env_transient[@]}"; do
        _t="${_t#"${_t%%[![:space:]]*}"}"; _t="${_t%"${_t##*[![:space:]]}"}"
        [ -z "$_t" ] && continue
        TRANSIENT_SET["$_t"]=1
    done
fi

# Per-profile sweep. Each entry is captured as a JSON object string so we can
# stitch them into the final envelope without spawning jq per profile twice.
results_json="[]"
pass_count=0
fail_count=0
unreachable_count=0
transient_skipped_count=0

# Use a temp file to collect jq-shaped objects line-by-line.
tmp_results="$(mktemp -t fleet-dm-canary.XXXXXX)"
trap 'rm -f "$tmp_results"' EXIT

for i in "${!profile_names[@]}"; do
    name="${profile_names[$i]}"
    addr="${profile_addrs[$i]}"

    is_transient=0
    [ -n "${TRANSIENT_SET[$name]:-}" ] && is_transient=1
    if [ "$is_transient" = 1 ]; then transient_json=true; else transient_json=false; fi

    out="$($TIMEOUT_CMD bash "$SELFTEST" --hub "$addr" --json 2>/dev/null || true)"
    if [ -z "$out" ] || ! printf '%s' "$out" | jq -e . >/dev/null 2>&1; then
        # Selftest exited non-zero AND emitted no parseable JSON — treat as
        # unreachable. Includes timeout(1) exit 124 (whole-sweep wedged on
        # a frozen hub — PL-189) and selftest setup-fail (network drop).
        # T-2225: a declared-transient host being down is EXPECTED — classify
        # transient_skipped (does NOT flip overall_ok) instead of unreachable.
        if [ "$is_transient" = 1 ]; then
            transient_skipped_count=$((transient_skipped_count + 1))
        else
            unreachable_count=$((unreachable_count + 1))
        fi
        jq -n -c \
            --arg name "$name" \
            --arg addr "$addr" \
            --argjson transient "$transient_json" \
            '{name:$name,address:$addr,verdict:"unreachable",elapsed_ms:null,transient:$transient,error:"selftest produced no json"}' \
            >> "$tmp_results"
        continue
    fi

    verdict="$(printf '%s' "$out" | jq -r '.verdict // "unknown"')"
    elapsed="$(printf '%s' "$out" | jq -r '.elapsed_ms // 0')"
    case "$verdict" in
        pass)          pass_count=$((pass_count + 1)) ;;
        # T-2225: setup-fail on a declared-transient host is "asleep", not broken.
        setup-fail)    if [ "$is_transient" = 1 ]; then transient_skipped_count=$((transient_skipped_count + 1)); else unreachable_count=$((unreachable_count + 1)); fi ;;
        *)             fail_count=$((fail_count + 1)) ;;
    esac
    jq -n -c \
        --arg name "$name" \
        --arg addr "$addr" \
        --arg verdict "$verdict" \
        --argjson elapsed "$elapsed" \
        --argjson transient "$transient_json" \
        '{name:$name,address:$addr,verdict:$verdict,elapsed_ms:$elapsed,transient:$transient}' \
        >> "$tmp_results"
done

profiles_arr="$(jq -s -c '.' "$tmp_results")"
overall_ok=true
if [ "$fail_count" -gt 0 ] || [ "$unreachable_count" -gt 0 ]; then
    overall_ok=false
fi

if [ "$FORMAT" = json ]; then
    jq -n -c \
        --argjson profiles "$profiles_arr" \
        --argjson total "$total" \
        --argjson pass "$pass_count" \
        --argjson fail "$fail_count" \
        --argjson unreachable "$unreachable_count" \
        --argjson transient_skipped "$transient_skipped_count" \
        --argjson ok "$overall_ok" \
        '{ok:$ok, summary:{total:$total, pass:$pass, fail:$fail, unreachable:$unreachable, transient_skipped:$transient_skipped}, profiles:$profiles}'
elif [ "$QUIET" = 1 ] && [ "$overall_ok" = true ]; then
    :
else
    echo "Fleet doorbell+mail health: $([ "$overall_ok" = true ] && echo pass || echo DRIFT)"
    echo "  total=$total  pass=$pass_count  fail=$fail_count  unreachable=$unreachable_count  transient_skipped=$transient_skipped_count"
    printf '%s\n' "$profiles_arr" | jq -r '.[] | "  - \(.name)@\(.address): verdict=\(.verdict)\(if .elapsed_ms then " elapsed=\(.elapsed_ms)ms" else "" end)\(if .transient == true then " (transient — skipped)" else "" end)\(if .error then " error=\(.error)" else "" end)"'
fi

if [ "$overall_ok" = true ]; then
    exit 0
else
    exit 1
fi
