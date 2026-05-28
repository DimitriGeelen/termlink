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
set -u

FORMAT=human
QUIET=0
HEARTBEAT=1
HUBS_FILE="${HOME}/.termlink/hubs.toml"
SELFTEST="${SELFTEST:-scripts/agent-conversation-selftest.sh}"

die() {
    if [ "$FORMAT" = json ]; then
        printf '{"ok":false,"error":"%s"}\n' "$1"
    else
        echo "fleet-doorbell-mail-health: $1" >&2
    fi
    exit 2
}

usage() {
    sed -n '2,23p' "$0"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json)          FORMAT=json; shift ;;
        --quiet)         QUIET=1; shift ;;
        --no-heartbeat)  HEARTBEAT=0; shift ;;
        --hubs-file)     HUBS_FILE="${2:-}"; shift 2 ;;
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

total="${#profile_names[@]}"
[ "$total" -gt 0 ] || die "no profiles found in $HUBS_FILE"

# Per-profile sweep. Each entry is captured as a JSON object string so we can
# stitch them into the final envelope without spawning jq per profile twice.
results_json="[]"
pass_count=0
fail_count=0
unreachable_count=0

# Use a temp file to collect jq-shaped objects line-by-line.
tmp_results="$(mktemp -t fleet-dm-canary.XXXXXX)"
trap 'rm -f "$tmp_results"' EXIT

for i in "${!profile_names[@]}"; do
    name="${profile_names[$i]}"
    addr="${profile_addrs[$i]}"

    out="$(bash "$SELFTEST" --hub "$addr" --json 2>/dev/null || true)"
    if [ -z "$out" ] || ! printf '%s' "$out" | jq -e . >/dev/null 2>&1; then
        # Selftest exited non-zero AND emitted no parseable JSON — treat as
        # unreachable (network / setup-fail edge case).
        unreachable_count=$((unreachable_count + 1))
        jq -n -c \
            --arg name "$name" \
            --arg addr "$addr" \
            '{name:$name,address:$addr,verdict:"unreachable",elapsed_ms:null,error:"selftest produced no json"}' \
            >> "$tmp_results"
        continue
    fi

    verdict="$(printf '%s' "$out" | jq -r '.verdict // "unknown"')"
    elapsed="$(printf '%s' "$out" | jq -r '.elapsed_ms // 0')"
    case "$verdict" in
        pass)          pass_count=$((pass_count + 1)) ;;
        setup-fail)    unreachable_count=$((unreachable_count + 1)) ;;
        *)             fail_count=$((fail_count + 1)) ;;
    esac
    jq -n -c \
        --arg name "$name" \
        --arg addr "$addr" \
        --arg verdict "$verdict" \
        --argjson elapsed "$elapsed" \
        '{name:$name,address:$addr,verdict:$verdict,elapsed_ms:$elapsed}' \
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
        --argjson ok "$overall_ok" \
        '{ok:$ok, summary:{total:$total, pass:$pass, fail:$fail, unreachable:$unreachable}, profiles:$profiles}'
elif [ "$QUIET" = 1 ] && [ "$overall_ok" = true ]; then
    :
else
    echo "Fleet doorbell+mail health: $([ "$overall_ok" = true ] && echo pass || echo DRIFT)"
    echo "  total=$total  pass=$pass_count  fail=$fail_count  unreachable=$unreachable_count"
    printf '%s\n' "$profiles_arr" | jq -r '.[] | "  - \(.name)@\(.address): verdict=\(.verdict)\(if .elapsed_ms then " elapsed=\(.elapsed_ms)ms" else "" end)\(if .error then " error=\(.error)" else "" end)"'
fi

if [ "$overall_ok" = true ]; then
    exit 0
else
    exit 1
fi
