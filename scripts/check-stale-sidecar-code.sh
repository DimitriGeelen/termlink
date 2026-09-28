#!/usr/bin/env bash
# T-3205 — detect a LIVE notify-sidecar executing code that is no longer on disk.
#
# DELIBERATELY NOT MARKED `# guard-layer: source`. That marker means "safe to run
# anywhere: no live hub, no network, no host state" — and this reads live processes.
# Run from the guard-layer runner in CI it would find zero sidecars and report a clean
# bill about a host that has none, which is the vacuous-green shape T-2747/T-3105 warn
# about. It is a RUNTIME canary and belongs to cron
# (.context/cron/stale-sidecar-code-canary.crontab). The FIXTURES are hermetic and do
# carry the marker.
#
# WHY THIS EXISTS
# ---------------
# A long-lived detached process keeps running the code it loaded. Editing the script
# changes nothing for it. `notify-sidecar-supervisor.sh` deliberately never restarts a
# live sidecar — it only starts missing ones — so nothing in the system notices.
#
# Measured, T-3204: a fix was committed, tested, pushed and then sat DARK in production
# for hours while every surface reported healthy — process alive, heartbeat fresh, canary
# quiet, guard layer green. The three sidecars had been running since Sep 22 against code
# written Sep 28. Nothing distinguishes "running" from "running the current code".
#
# T-2405 built exactly this detector for push-wakers and scoped it there. That made the
# gap MORE expensive rather than less: the existing canary's green reads as coverage of a
# class it covers one member of. This is the sibling for sidecars (PL-392).
#
# DETECTION ONLY — it never restarts anything. A restart posts receipts visible to peers
# and can inject into live session prompts; T-3204's restart needed explicit operator
# approval for precisely that. A checker that auto-restarted would convert a visible
# staleness into a silent one, which is the trade this exists to reverse.
#
# THE PRIMITIVES ARE T-2405'S, NOT COPIES
# ---------------------------------------
# code_mtime / proc_start_mtime / is_stale are EXTRACTED from
# check-stale-waker-code-freshness.sh at runtime and eval'd. Two hand-copies of a subtle
# mtime comparison drift, and the copy that drifts is the one that quietly stops catching
# things. If extraction fails we exit 2 — never fall back to a private reimplementation,
# because a detector silently running its own logic is the drift this guards against.
#
# Exit: 0 all current · 1 one or more stale · 2 tooling (fail-closed)
set -uo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SIBLING="${SIDECAR_STALE_SIBLING:-$SELF_DIR/check-stale-waker-code-freshness.sh}"
SIDECAR_SCRIPT="${SIDECAR_STALE_SCRIPT:-$SELF_DIR/notify-sidecar.sh}"
HEARTBEAT="${SIDECAR_STALE_HEARTBEAT:-$SELF_DIR/../.context/working/.stale-sidecar-code-canary.heartbeat}"

JSON=0; QUIET=0; NO_HEARTBEAT=0
while [ $# -gt 0 ]; do
    case "$1" in
        --json)          JSON=1; shift ;;
        --quiet)         QUIET=1; shift ;;
        --no-heartbeat)  NO_HEARTBEAT=1; shift ;;
        -h|--help)
            sed -n '2,32p' "${BASH_SOURCE[0]}"; exit 0 ;;
        *) echo "check-stale-sidecar-code: unknown arg: $1" >&2; exit 2 ;;
    esac
done

die() { echo "check-stale-sidecar-code: $*" >&2; exit 2; }

# --- AC2: reuse, do not reimplement -----------------------------------------------
[ -r "$SIBLING" ] || die "cannot read sibling detector $SIBLING (T-2405) — refusing to reimplement its primitives"

# Extract ONE function at a time. A single awk with three ranges looks equivalent and is
# not: the ranges match independently, so overlapping regions print more than once and
# the result eval'd to `is_stale() { is_stale() { is_stale() {` — nested definitions with
# the wrong return value. It eval'd cleanly and `declare -F` found the name, so an
# existence check passed while the logic was garbage; the live run then reported two
# freshly-restarted sidecars as stale. Caught before shipping, and the reason the
# behavioural self-test below exists rather than just a "does the name exist" check.
for fn in code_mtime proc_start_mtime is_stale; do
    _src="$(awk -v f="$fn" '$0 ~ "^"f"\\(\\) \\{", /^\}/' "$SIBLING" 2>/dev/null)"
    [ -n "$_src" ] || die "primitive '$fn' not found in $SIBLING — it was renamed or refactored; fix the extraction rather than copying the logic"
    eval "$_src" 2>/dev/null || die "could not evaluate primitive '$fn' from $SIBLING"
    declare -F "$fn" >/dev/null 2>&1 || die "primitive '$fn' did not define after eval"
done

# BEHAVIOURAL self-test of the borrowed primitives. Existence is not correctness — the
# bug above proves it. If T-2405's semantics change under us, or extraction mangles them
# again, this exits 2 rather than silently classifying every sidecar wrongly.
# Contract asserted: older-than-code is stale; newer is not; empty (dead pid) is not.
is_stale 100 200   || die "borrowed is_stale failed self-test: 100 older than 200 must be stale"
is_stale 300 200   && die "borrowed is_stale failed self-test: 300 newer than 200 must NOT be stale"
is_stale ""  200   && die "borrowed is_stale failed self-test: empty proc mtime must NOT be stale"
:

[ -r "$SIDECAR_SCRIPT" ] || die "cannot read $SIDECAR_SCRIPT — no staleness reference"
CM="$(code_mtime "$SIDECAR_SCRIPT")"
[ "${CM:-0}" -gt 0 ] 2>/dev/null || die "could not stat $SIDECAR_SCRIPT for mtime"

# --- discovery ---------------------------------------------------------------------
# Test seam: lines of "<pid>\t<proc_start_mtime>\t<cmdline>". When set, /proc is not
# consulted at all, so the suite runs with no live sidecar and no root.
#
# The live anchor uses notify-sidecar[.]sh deliberately. `pgrep -f` scans every cmdline
# INCLUDING this script's own, and the plain pattern matches itself — measured in T-3204,
# where it counted 4 processes when 3 were running. A detector that matches its own
# checker reports a number unrelated to the thing being checked.
procs=""
if [ -n "${SIDECAR_STALE_TEST_PROCS:-}" ]; then
    [ -r "$SIDECAR_STALE_TEST_PROCS" ] || die "test seam file unreadable: $SIDECAR_STALE_TEST_PROCS"
    procs="$(cat "$SIDECAR_STALE_TEST_PROCS")"
else
    command -v pgrep >/dev/null 2>&1 || die "pgrep not available"
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        pid="${line%% *}"
        cmd="${line#* }"
        pm="$(proc_start_mtime "$pid")"
        printf -v _row '%s\t%s\t%s' "$pid" "$pm" "$cmd"
        procs="${procs}${_row}"$'\n'
    done < <(pgrep -fa 'notify-sidecar[.]sh --agent-id' 2>/dev/null)
fi

stale=(); current=(); skipped=0
while IFS=$'\t' read -r pid pm cmd; do
    [ -n "${pid:-}" ] || continue
    # AC4 — pid-recycle guard (T-2239 pattern). A pid can be reused between the scan and
    # the stat; if the cmdline no longer names our script, it is somebody else's process
    # and reporting it as a stale sidecar would be a confident wrong answer.
    case "$cmd" in
        *notify-sidecar.sh*) : ;;
        *) skipped=$((skipped+1)); continue ;;
    esac
    # A pid that vanished mid-scan has no start mtime. That is the not-running class,
    # not the old-code class — is_stale already treats empty as not-stale, and we count
    # it as skipped rather than inventing a verdict about a process that is gone.
    if [ -z "${pm:-}" ]; then skipped=$((skipped+1)); continue; fi
    agent="$(printf '%s' "$cmd" | sed -n 's/.*--agent-id[ =]\([^ ]*\).*/\1/p')"
    [ -n "$agent" ] || agent="(unknown)"
    if is_stale "$pm" "$CM"; then
        stale+=("$pid|$agent|$pm")
    else
        current+=("$pid|$agent|$pm")
    fi
done <<< "$procs"

n_stale=${#stale[@]}; n_current=${#current[@]}
rc=0; [ "$n_stale" -gt 0 ] && rc=1

if [ "$NO_HEARTBEAT" -eq 0 ]; then
    mkdir -p "$(dirname "$HEARTBEAT")" 2>/dev/null || true
    date +%s > "$HEARTBEAT" 2>/dev/null || true
fi

if [ "$JSON" -eq 1 ]; then
    printf '{"ok":%s,"code_mtime":%s,"script":"%s","stale_count":%s,"current_count":%s,"skipped":%s,"stale":[' \
        "$([ "$rc" -eq 0 ] && echo true || echo false)" "$CM" "$SIDECAR_SCRIPT" "$n_stale" "$n_current" "$skipped"
    first=1
    for e in ${stale[@]+"${stale[@]}"}; do
        IFS='|' read -r p a m <<< "$e"
        [ "$first" -eq 1 ] || printf ','
        printf '{"pid":%s,"agent_id":"%s","proc_start_mtime":%s}' "$p" "$a" "$m"
        first=0
    done
    printf ']}\n'
    exit "$rc"
fi

if [ "$rc" -eq 0 ]; then
    # Zero sidecars is NOT the same claim as "all sidecars are current", and saying
    # "healthy" for it is how a check earns a green it did not measure. Non-firing
    # either way — a host may legitimately run none — but it must read differently.
    if [ "$n_current" -eq 0 ] && [ "$n_stale" -eq 0 ]; then
        [ "$QUIET" -eq 1 ] || {
            echo "stale-sidecar-code: NOT EVALUATED — no notify-sidecar processes found."
            echo "  Nothing was compared. This is not a clean bill; it is an empty candidate set."
            echo "  If sidecars are expected here, that absence is itself the problem:"
            echo "    bash scripts/notify-sidecar-supervisor.sh"
            [ "$skipped" -gt 0 ] && echo "  ($skipped process(es) skipped: pid recycled or exited mid-scan)"
        }
        exit 0
    fi
    [ "$QUIET" -eq 1 ] || {
        echo "stale-sidecar-code: healthy — $n_current sidecar(s) running current code, 0 stale"
        echo "  reference: $SIDECAR_SCRIPT (code_mtime=$CM)"
        [ "$skipped" -gt 0 ] && echo "  $skipped process(es) skipped (pid recycled or exited mid-scan)"
        echo "  Scope: compares process start time against script mtime. It does NOT verify the"
        echo "  running code is CORRECT, only that it is not older than what is on disk."
    }
    exit 0
fi

echo "stale-sidecar-code canary: FIRING — $n_stale sidecar(s) running code older than $SIDECAR_SCRIPT"
for e in ${stale[@]+"${stale[@]}"}; do
    IFS='|' read -r p a m <<< "$e"
    echo "  [stale] agent=$a pid=$p started=$(date -d "@$m" '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo "$m")"
    echo "      script mtime=$(date -d "@$CM" '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo "$CM") — this process cannot"
    echo "      be executing the current code. A fix that looks shipped is dark here."
done
echo ""
echo "  Remediation is an OPERATOR action, deliberately not automated: restarting a"
echo "  sidecar posts receipts visible to peers and can inject into live prompts."
echo "    pkill -TERM -f 'notify-sidecar[.]sh --agent-id'"
echo "    bash scripts/notify-sidecar-supervisor.sh     # respawns on current code"
echo "  Then confirm the RUNNING code carries the change via an observable only the new"
echo "  code can produce — a fresh start time alone proves a restart, not which code it"
echo "  loaded (PL-392)."
exit 1
