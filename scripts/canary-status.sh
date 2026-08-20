#!/usr/bin/env bash
# T-2172 — canary-status: unified cron-canary visibility scanner.
#
# Why this exists: TermLink installs N cron canaries (T-2160 substrate-preflight,
# T-1696 release-mirror, T-1723 meta-canary-aliveness, fleet-doorbell-mail, ...)
# each writing to `.context/working/.*-canary.log` with companion `.heartbeat`
# files following the "empty log = healthy" convention. An operator wanting to
# answer "are my canaries firing AND clean?" must know each path, the convention,
# and the expected cadence. That's dormant tooling (PL-168) — this script is the
# trigger that surfaces the surface.
#
# Pattern parity with `/substrate` (T-2096): one-shot situational digest composed
# from many small checks. Substrate-arc framing: completes the substrate safety
# set's visibility tier (CLI/T-2154 preflight → skill/T-2158 → smoke/T-2170 →
# cron/T-2160 → THIS).
#
# Classification taxonomy:
#   HEALTHY        — log size 0 (or log entries all older than the latest
#                    heartbeat) AND heartbeat is fresh (< max-age-hours).
#   FIRING         — log non-empty AND latest log entry mtime >= latest
#                    heartbeat mtime (cron is firing AND finding problems).
#   STALE          — heartbeat older than max-age-hours (cron may have stopped
#                    firing — protection silently degraded).
#   ON_DEMAND      — registered in .context/cron/ondemand-checks.conf as a
#                    deliberately non-cron check (the source-level static checks:
#                    T-2527 alloc-sink, T-2531 drain-sink, T-2666 silent-exit,
#                    T-2672 busy-spin) AND its log is empty. Heartbeat age is not
#                    a health signal for these, so it is not graded — but a
#                    registered check WITH findings still reports FIRING (T-2688).
#                    Not counted as a problem; excluded from --quiet.
#   TOOLING        — the `.log.stderr` companion has content no older than the heartbeat
#                    while the findings log is clean: the check exited 2 (could
#                    not RUN) rather than 1 (ran, found something). T-2696.
#                    Reachable only from HEALTHY — FIRING and STALE both dominate,
#                    so a tooling error can never mask a real finding or a dead
#                    cron. Counted as a problem: a canary that cannot run is dark,
#                    which is worse than the drift it was watching for.
#   NO_HEARTBEAT   — log file present but no .heartbeat companion. Some
#                    canaries don't track heartbeats; classified by log content
#                    alone (empty=HEALTHY, non-empty=FIRING).
#
# Exit codes:
#   0 — all canaries healthy (cron firing AND no entries)
#   1 — at least one canary is FIRING, STALE or TOOLING (operator action required)
#   2 — tooling error (missing dir, jq missing in --json mode, etc.)
#
# Usage:
#   canary-status.sh                     # human-readable summary, all canaries
#   canary-status.sh --json              # machine envelope (jq-friendly)
#   canary-status.sh --quiet             # only render problems (cron-friendly)
#   canary-status.sh --max-age-hours 72  # custom stale threshold (default 48)
#
# Discovery: globs `.context/working/.*-canary.log` and `.canary-aliveness.log`
# (the meta-canary), then pairs each with `.context/working/<stem>.heartbeat`
# and `<log>.stderr` if present. No hard-coded canary list — new canaries appear
# automatically.
#
# The `.log.stderr` companion (T-2696). Canary crontabs originally wired
# `>> <log> 2>&1`, merging the exit-2 tooling-error stream into the log whose
# sole documented meaning is "a real problem was detected". One unreachable-
# network cron run then pinned a canary FIRING permanently, with no documented
# way to clear it — and a canary set that cries wolf stops being read, which is
# how the thing it guards goes unwatched.
#
# The split itself was already live: every installed /etc/cron.d copy routes
# `2>> <log>.stderr`. But that edit was made on the deployed files only and never
# committed, so git still carried the merged form (the T-2689/T-2692 class: a fix
# that runs but is not recoverable), and NOTHING read the .stderr files — a
# write-only sink, the G-063 class. T-2696 committed the wiring back to git and
# made this script the reader. A canary with no .stderr companion classifies
# exactly as before.
#
# See also:
#   /substrate   — runtime-state digest (T-2096)
#   /preflight   — deploy-time correctness (T-2158)
#   /canaries    — this script's slash-skill wrapper (T-2172)

set -eu

WORKING_DIR=".context/working"
MAX_AGE_HOURS=48
JSON=0
QUIET=0

usage() {
    sed -n '2,/^set -eu$/p' "$0" | sed 's/^# \{0,1\}//' | head -n -2
    exit 0
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1 ;;
        --quiet) QUIET=1 ;;
        --max-age-hours)
            shift
            [ $# -ge 1 ] || { echo "canary-status: --max-age-hours requires a value" >&2; exit 2; }
            MAX_AGE_HOURS="$1"
            ;;
        --working-dir)
            shift
            [ $# -ge 1 ] || { echo "canary-status: --working-dir requires a value" >&2; exit 2; }
            WORKING_DIR="$1"
            ;;
        -h|--help) usage ;;
        *) echo "canary-status: unknown flag: $1" >&2; exit 2 ;;
    esac
    shift
done

if [ ! -d "$WORKING_DIR" ]; then
    echo "canary-status: working dir not found: $WORKING_DIR" >&2
    exit 2
fi

# Stale threshold in seconds.
MAX_AGE_SECS=$((MAX_AGE_HOURS * 3600))
NOW=$(date +%s)

# T-2688: on-demand checks — canaries that are deliberately NOT cron-backed, so
# heartbeat age says nothing about their health. Registered in a git-tracked conf
# (override with ONDEMAND_CHECKS_CONF for fixtures). Registration suppresses
# STALENESS ONLY; a registered check whose log carries findings still FIREs.
ONDEMAND_CHECKS_CONF="${ONDEMAND_CHECKS_CONF:-.context/cron/ondemand-checks.conf}"

ONDEMAND_NAMES=""
if [ -r "$ONDEMAND_CHECKS_CONF" ]; then
    ONDEMAND_NAMES=$(sed -e 's/#.*//' -e 's/[[:space:]]//g' "$ONDEMAND_CHECKS_CONF" 2>/dev/null | grep -v '^$' || true)
fi

is_ondemand() {
    [ -n "$ONDEMAND_NAMES" ] || return 1
    printf '%s\n' "$ONDEMAND_NAMES" | grep -qxF "$1"
}

# Discover canary log files. Pattern: any `.context/working/.*-canary.log`,
# plus the meta-canary aliveness log (uses different naming), plus the
# `.log` path SYNTHESIZED from any `.*-canary.heartbeat` so we surface
# healthy, never-fired canaries (T-2178). classify() handles log_size=0 /
# log_mtime=0 cleanly so a synthesized-but-absent .log renders as `log=--`.
# sort -u dedups: if both .log and .heartbeat exist, we list once.
discover_canaries() {
    {
        ls -1 "$WORKING_DIR"/.*-canary.log 2>/dev/null
        ls -1 "$WORKING_DIR"/.canary-aliveness.log 2>/dev/null
        for hb in "$WORKING_DIR"/.*-canary.heartbeat; do
            [ -e "$hb" ] || continue
            printf '%s\n' "${hb%.heartbeat}.log"
        done
    } | sort -u
}

# stat -c %Y wrapper portable to BSD-stat (macOS). Returns 0 if file absent.
file_mtime() {
    if [ -f "$1" ]; then
        stat -c %Y "$1" 2>/dev/null || stat -f %m "$1" 2>/dev/null || echo 0
    else
        echo 0
    fi
}

file_size() {
    if [ -f "$1" ]; then
        stat -c %s "$1" 2>/dev/null || stat -f %z "$1" 2>/dev/null || echo 0
    else
        echo 0
    fi
}

# Compute per-canary classification + metadata. Emits one TSV row per canary
# (avoids quoting hell when passing to the renderer): name TAB status TAB
# log_size TAB log_mtime TAB heartbeat_mtime TAB latest_entry.
classify() {
    local log_path="$1"
    local stem name heartbeat_path
    stem="${log_path%.log}"
    name="${log_path##*/.}"
    name="${name%.log}"
    heartbeat_path="${stem}.heartbeat"

    local log_size log_mtime heartbeat_mtime
    log_size=$(file_size "$log_path")
    log_mtime=$(file_mtime "$log_path")
    heartbeat_mtime=$(file_mtime "$heartbeat_path")

    # T-2696: the `.log.stderr` companion. Canary crontabs route findings
    # (stdout) to the .log and tooling errors (stderr, exit 2) here, instead of
    # merging both with `2>&1`. The suffix is not a new invention — it is what
    # the installed /etc/cron.d copies have been writing all along; this reads
    # what already exists. Absent companion means err_size=0, no TOOLING branch
    # is reachable, and classification is identical to pre-T-2696 behaviour.
    local err_path err_size err_mtime
    err_path="${log_path}.stderr"
    err_size=$(file_size "$err_path")
    err_mtime=$(file_mtime "$err_path")

    local status
    if [ "$heartbeat_mtime" = "0" ]; then
        # No heartbeat companion. Classify by log content.
        if [ "$log_size" = "0" ]; then
            status="HEALTHY"
        else
            status="FIRING"
        fi
    else
        local heartbeat_age=$((NOW - heartbeat_mtime))
        # T-2688: a registered on-demand check is graded on findings only. Its
        # heartbeat is touched when a human runs it, so age is not a health signal
        # and grading it STALE pinned /canaries permanently red. Findings are NOT
        # suppressed — the FIRING branches below are reached exactly as before, so
        # an on-demand check that found something still exits 1.
        if [ "$heartbeat_age" -gt "$MAX_AGE_SECS" ] && is_ondemand "$name" && [ "$log_size" = "0" ]; then
            status="ON_DEMAND"
        elif [ "$heartbeat_age" -gt "$MAX_AGE_SECS" ] && ! is_ondemand "$name"; then
            status="STALE"
        elif [ "$log_size" = "0" ]; then
            status="HEALTHY"
        elif [ "$log_mtime" -gt "$heartbeat_mtime" ]; then
            # Log entries newer than heartbeat: cron fired AND found problems.
            status="FIRING"
        else
            # Log non-empty but no new entries since last heartbeat: prior
            # firings are now resolved (healthy current state, historical
            # entries remain in log).
            status="HEALTHY"
        fi
    fi

    # T-2696: TOOLING — the check could not RUN (exit 2), as opposed to running
    # and finding something (exit 1). Reachable ONLY from HEALTHY: a live finding
    # (FIRING) or a cron that stopped firing at all (STALE) both dominate, so a
    # tooling error can never mask either. The `.log.stderr` must be no older than the
    # heartbeat, mirroring the log rule directly above — an error followed by
    # later clean runs is history, not a current fault.
    if [ "$status" = "HEALTHY" ] && [ "$err_size" != "0" ]; then
        if [ "$heartbeat_mtime" = "0" ] || [ "$err_mtime" -ge "$heartbeat_mtime" ]; then
            status="TOOLING"
        fi
    fi

    # Latest log entry — prefer a signal-bearing line over the literal last
    # line. Canaries that print one multi-line block per cron run (e.g.
    # fleet-doorbell-mail prints a `DRIFT` header + summary counters + one
    # row per hub) have the actionable signal mid-block; `tail -n 1` would
    # return the last per-row entry (typically a passing row), which is
    # operator-misleading when status=FIRING. The heuristic scans the recent
    # tail (last 50 lines) for any line matching fail|drift|stale|warn|
    # error|behind (case-insensitive — covers verdict=fail / verdict=setup-fail,
    # `DRIFT`, mirror canary's "behind origin", etc.) and returns the
    # latest match. Falls back to the first non-empty line of the recent
    # tail when no signal found — typically the per-run header (e.g.
    # `Fleet doorbell+mail health: DRIFT`).
    # T-2696: for TOOLING the actionable text lives in the .log.stderr companion, not
    # the (clean) findings log — read the entry from whichever file carries the
    # signal for this status.
    local entry_src entry_size
    entry_src="$log_path"
    entry_size="$log_size"
    if [ "$status" = "TOOLING" ]; then
        entry_src="$err_path"
        entry_size="$err_size"
    fi

    local latest_entry=""
    if [ "$entry_size" != "0" ]; then
        local recent_tail
        recent_tail=$(tail -n 50 "$entry_src" 2>/dev/null)
        local signal
        signal=$(printf '%s\n' "$recent_tail" | grep -E -i 'fail|drift|stale|warn|error|behind' | tail -n 1 | head -c 120)
        if [ -n "$signal" ]; then
            latest_entry="$signal"
        else
            latest_entry=$(printf '%s\n' "$recent_tail" | grep -v '^$' | head -n 1 | head -c 120)
        fi
    fi

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$name" "$status" "$log_size" "$log_mtime" "$heartbeat_mtime" "$err_size" "$latest_entry"
}

# Build the result set.
RESULTS=""
TOTAL=0
HEALTHY=0
FIRING=0
STALE=0
NO_HB=0
ONDEMAND=0
TOOLING=0

while IFS= read -r log_path; do
    [ -n "$log_path" ] || continue
    row=$(classify "$log_path")
    RESULTS="${RESULTS}${row}"$'\n'
    TOTAL=$((TOTAL + 1))
    status=$(echo "$row" | awk -F'\t' '{print $2}')
    case "$status" in
        HEALTHY) HEALTHY=$((HEALTHY + 1)) ;;
        FIRING) FIRING=$((FIRING + 1)) ;;
        STALE) STALE=$((STALE + 1)) ;;
        NO_HEARTBEAT) NO_HB=$((NO_HB + 1)) ;;
        ON_DEMAND) ONDEMAND=$((ONDEMAND + 1)) ;;
        TOOLING) TOOLING=$((TOOLING + 1)) ;;
    esac
done <<EOF
$(discover_canaries)
EOF

# T-2688: ON_DEMAND is deliberately absent here — a registered ad-hoc check with an
# empty log is not an operator problem, and counting it kept /canaries permanently
# non-zero. Its FIRING path is untouched, so findings still land in PROBLEMS.
# T-2696: TOOLING counts. A canary that cannot RUN is dark, which is strictly
# worse than the drift it was watching for — making it quiet would trade a loud
# wrong answer for a silent one.
PROBLEMS=$((FIRING + STALE + TOOLING))

# JSON rendering.
if [ "$JSON" = "1" ]; then
    printf '{"ok":true,"summary":{"total":%d,"healthy":%d,"firing":%d,"stale":%d,"no_heartbeat":%d,"on_demand":%d,"tooling":%d,"max_age_hours":%d},"canaries":[' \
        "$TOTAL" "$HEALTHY" "$FIRING" "$STALE" "$NO_HB" "$ONDEMAND" "$TOOLING" "$MAX_AGE_HOURS"
    first=1
    while IFS=$'\t' read -r name status log_size log_mtime hb_mtime err_size latest_entry; do
        [ -n "$name" ] || continue
        [ "$first" = "1" ] || printf ','
        first=0
        # JSON-escape the latest_entry (minimal: quotes + backslashes + newlines).
        esc=$(printf '%s' "$latest_entry" | sed 's/\\/\\\\/g; s/"/\\"/g; s/	/\\t/g')
        printf '{"name":"%s","status":"%s","log_size":%s,"log_mtime":%s,"heartbeat_mtime":%s,"err_size":%s,"latest_entry":"%s"}' \
            "$name" "$status" "$log_size" "$log_mtime" "$hb_mtime" "$err_size" "$esc"
    done <<EOF
$RESULTS
EOF
    printf ']}\n'
    [ "$PROBLEMS" = "0" ] && exit 0 || exit 1
fi

# Human rendering.
render_status() {
    case "$1" in
        HEALTHY) printf '\033[0;32m%-12s\033[0m' "HEALTHY" ;;
        FIRING)  printf '\033[0;31m%-12s\033[0m' "FIRING" ;;
        STALE)   printf '\033[0;33m%-12s\033[0m' "STALE" ;;
        # T-2688: dim, not yellow — an on-demand check is informational by design.
        ON_DEMAND) printf '\033[0;36m%-12s\033[0m' "ON_DEMAND" ;;
        # T-2696: magenta — actionable like FIRING, but a different problem
        # (the check could not run), so it must not read as drift at a glance.
        TOOLING) printf '\033[0;35m%-12s\033[0m' "TOOLING" ;;
        *)       printf '%-12s' "$1" ;;
    esac
}

ts_or_dash() {
    if [ "$1" = "0" ]; then
        printf '    --      '
    else
        date -d "@$1" '+%Y-%m-%d %H:%M' 2>/dev/null || printf '    --      '
    fi
}

if [ "$QUIET" = "1" ] && [ "$PROBLEMS" = "0" ]; then
    # Quiet mode and no problems — emit nothing (cron-friendly).
    exit 0
fi

if [ "$TOTAL" = "0" ]; then
    echo "canary-status: no canaries found in $WORKING_DIR"
    echo "  (expected files: .*-canary.log + companion .*-canary.heartbeat)"
    exit 0
fi

if [ "$QUIET" = "1" ]; then
    # Quiet mode WITH problems: render only the FIRING/STALE rows.
    echo "canary-status: $PROBLEMS canary(ies) need attention ($FIRING firing, $STALE stale, $TOOLING tooling, threshold ${MAX_AGE_HOURS}h)"
    while IFS=$'\t' read -r name status log_size log_mtime hb_mtime err_size latest_entry; do
        [ -n "$name" ] || continue
        case "$status" in
            FIRING|STALE|TOOLING) ;;
            *) continue ;;
        esac
        printf '  %s %s\n' "$(render_status "$status")" "$name"
        [ -n "$latest_entry" ] && printf '             ↳ %s\n' "$latest_entry"
    done <<EOF
$RESULTS
EOF
    exit 1
fi

# Full human render.
ONDEMAND_NOTE=""
[ "$ONDEMAND" != "0" ] && ONDEMAND_NOTE=", $ONDEMAND on-demand"
TOOLING_NOTE=""
[ "$TOOLING" != "0" ] && TOOLING_NOTE=", $TOOLING tooling"
echo "canary-status: $TOTAL canary(ies) — $HEALTHY healthy, $FIRING firing, $STALE stale${TOOLING_NOTE}${ONDEMAND_NOTE} (threshold ${MAX_AGE_HOURS}h)"
echo ""
printf '  %-12s %-32s %s\n' "STATUS" "NAME" "LAST FIRED / LATEST ENTRY"
printf '  %-12s %-32s %s\n' "------" "----" "-------------------------"
while IFS=$'\t' read -r name status log_size log_mtime hb_mtime err_size latest_entry; do
    [ -n "$name" ] || continue
    printf '  %s %-32s ' "$(render_status "$status")" "$name"
    # Show most-recent timestamp (heartbeat or log mtime, whichever is newer).
    most_recent=$log_mtime
    [ "$hb_mtime" -gt "$most_recent" ] && most_recent=$hb_mtime
    if [ "$most_recent" != "0" ]; then
        printf 'hb=%s log=%s\n' "$(ts_or_dash "$hb_mtime")" "$(ts_or_dash "$log_mtime")"
    else
        echo ""
    fi
    [ -n "$latest_entry" ] && printf '               ↳ %s\n' "$latest_entry"
done <<EOF
$RESULTS
EOF

# Actionable hints for problems.
if [ "$PROBLEMS" != "0" ]; then
    echo ""
    echo "Action needed:"
    if [ "$FIRING" != "0" ]; then
        echo "  FIRING — a canary is detecting a real problem. Read the log:"
        echo "    cat $WORKING_DIR/.<name>-canary.log"
        echo "  Then fix the underlying drift (rotation, mirror sync, etc.) per the relevant runbook."
    fi
    if [ "$TOOLING" != "0" ]; then
        echo "  TOOLING — the check could not RUN (exit 2). This is NOT drift: the canary"
        echo "            never got far enough to have an opinion. Read the error:"
        echo "    cat $WORKING_DIR/.<name>-canary.log.stderr"
        echo "  Re-run it ad-hoc to see whether the fault persists:"
        echo "    bash scripts/<canary-script>.sh"
        echo "  Exit 0 means the error was transient (network blip, hub restarting) and the"
        echo "  companion can be cleared:  : > $WORKING_DIR/.<name>-canary.log.stderr"
        echo "  A fault that persists means the canary is DARK — fix it before trusting the"
        echo "  empty log of the thing it was watching."
    fi
    if [ "$STALE" != "0" ]; then
        echo "  STALE  — a canary cron hasn't fired in >${MAX_AGE_HOURS}h. Check that cron is loaded:"
        echo "    sudo cat /etc/cron.d/<canary-name>"
        echo "    sudo systemctl status cron"
        echo "  Then verify the script runs manually:"
        echo "    bash scripts/<canary-script>.sh --quiet"
    fi
fi

[ "$PROBLEMS" = "0" ] && exit 0 || exit 1
