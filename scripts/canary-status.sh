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
#   NO_HEARTBEAT   — log file present but no .heartbeat companion. Some
#                    canaries don't track heartbeats; classified by log content
#                    alone (empty=HEALTHY, non-empty=FIRING).
#
# Exit codes:
#   0 — all canaries healthy (cron firing AND no entries)
#   1 — at least one canary is FIRING or STALE (operator action required)
#   2 — tooling error (missing dir, jq missing in --json mode, etc.)
#
# Usage:
#   canary-status.sh                     # human-readable summary, all canaries
#   canary-status.sh --json              # machine envelope (jq-friendly)
#   canary-status.sh --quiet             # only render FIRING/STALE (cron-friendly)
#   canary-status.sh --max-age-hours 72  # custom stale threshold (default 48)
#
# Worktree resolution (T-2763): canary logs/heartbeats are written by host cron,
# which runs from the MAIN checkout. Inside a linked git worktree this script now
# resolves to the main checkout's `.context/working/` instead of the worktree's
# own (empty) copy — otherwise a canary genuinely FIRING reads HEALTHY here, a
# false all-clear. Detection is `git rev-parse --git-dir` vs `--git-common-dir`
# (equal in a plain checkout, different in a worktree), never a path string match.
# If the main checkout's dir is unreadable the script exits 2 rather than falling
# back. `--working-dir` always wins. Every non-silent output path names the
# directory actually read and the resolution mode, so "which tree did this look
# at?" is never a guess. Seams: CANARY_STATUS_TEST_GIT_DIR /
# CANARY_STATUS_TEST_GIT_COMMON_DIR.
#
# Discovery: globs `.context/working/.*-canary.log` and `.canary-aliveness.log`
# (the meta-canary), then pairs each with `.context/working/<stem>.heartbeat`
# if present. No hard-coded canary list — new canaries appear automatically.
#
# See also:
#   /substrate   — runtime-state digest (T-2096)
#   /preflight   — deploy-time correctness (T-2158)
#   /canaries    — this script's slash-skill wrapper (T-2172)

set -eu

WORKING_DIR=".context/working"
WORKING_DIR_EXPLICIT=0
MAX_AGE_HOURS=48
JSON=0
QUIET=0
# Set by resolve_canary_dir(): local | worktree->main | explicit
RESOLUTION="local"

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
            WORKING_DIR_EXPLICIT=1
            ;;
        -h|--help) usage ;;
        *) echo "canary-status: unknown flag: $1" >&2; exit 2 ;;
    esac
    shift
done

# ── Worktree resolution (T-2763) ──────────────────────────────────────────────
# Canary logs + heartbeats are written by HOST CRON, which runs from the MAIN
# checkout. Inside a linked git worktree `.context/working/` is a different,
# gitignored directory that cron never touches — so every log reads empty and
# every heartbeat reads ancient. That produces two wrong answers, and the second
# is the dangerous one:
#
#   (a) STALE noise   — heartbeats look ancient though cron is firing fine.
#   (b) FALSE ALL-CLEAR — a canary genuinely FIRING in the main checkout reads
#                         HEALTHY here, because this directory's log is empty.
#
# (b) is why this cannot be left as cosmetic drift: "empty log = healthy" is a
# one-bit channel, and reading the wrong file inverts it silently. `fw audit`
# already knows about worktrees ("Cron drift checks skipped — linked worktree");
# this script did not.
#
# Detection is git's OWN answer: --git-dir and --git-common-dir are equal in a
# plain checkout and differ in a linked worktree. Deliberately NOT a string match
# on `.claude/worktrees/` — that naming convention is a habit, not a contract, and
# `git worktree add` anywhere else must still resolve correctly.
#
# An explicit --working-dir always wins (that is the operator saying "I mean THIS
# directory"), and is also the seam the fixtures use.

# Echo an absolute path for a rev-parse dir selector, or return 1. Prefers
# --path-format=absolute (git >= 2.31); falls back to absolutising by hand so an
# older git degrades to a CORRECT answer rather than to the false all-clear.
_git_dir_abs() {
    local v
    if v=$(git rev-parse --path-format=absolute "$1" 2>/dev/null) && [ -n "$v" ]; then
        printf '%s' "$v"
        return 0
    fi
    v=$(git rev-parse "$1" 2>/dev/null) || return 1
    [ -n "$v" ] || return 1
    case "$v" in
        /*) printf '%s' "$v" ;;
        *)  printf '%s/%s' "$(pwd -P)" "$v" ;;
    esac
}

resolve_canary_dir() {
    if [ "$WORKING_DIR_EXPLICIT" = "1" ]; then
        RESOLUTION="explicit"
        return 0
    fi

    local gd gc main_root main_dir
    gd=$(_git_dir_abs --git-dir 2>/dev/null || true)
    gc=$(_git_dir_abs --git-common-dir 2>/dev/null || true)

    # Test seams (PL-213): feed the two git answers directly so the fixtures
    # exercise the REAL worktree decision and the REAL main-root derivation,
    # rather than bypassing them with a pre-computed directory.
    [ -n "${CANARY_STATUS_TEST_GIT_DIR:-}" ] && gd="$CANARY_STATUS_TEST_GIT_DIR"
    [ -n "${CANARY_STATUS_TEST_GIT_COMMON_DIR:-}" ] && gc="$CANARY_STATUS_TEST_GIT_COMMON_DIR"

    # Not a repo, or the two agree → plain checkout. Nothing to redirect.
    [ -n "$gd" ] && [ -n "$gc" ] || return 0
    [ "$gd" != "$gc" ] || return 0

    # Linked worktree. The main checkout is the parent of the common git dir.
    main_root=$(dirname "$gc")
    main_dir="$main_root/$WORKING_DIR"

    if [ -d "$main_dir" ] && [ -r "$main_dir" ]; then
        WORKING_DIR="$main_dir"
        RESOLUTION="worktree->main"
        return 0
    fi

    # REFUSE. Falling back to this worktree's own empty directory is exactly the
    # false all-clear this block exists to remove, so it is a tooling error (2),
    # never a healthy (0).
    echo "canary-status: running inside a linked git worktree, but the MAIN checkout's canary dir is unreadable:" >&2
    echo "  $main_dir" >&2
    echo "  Cron writes canary logs and heartbeats into the main checkout only. This worktree's" >&2
    echo "  copy is empty, so reporting on it would be a FALSE ALL-CLEAR, not a clean bill." >&2
    echo "  Refusing to fall back (T-2763). Either run from $main_root, or pass" >&2
    echo "  --working-dir <path> if you genuinely want this worktree's own copy." >&2
    exit 2
}

resolve_canary_dir

if [ ! -d "$WORKING_DIR" ]; then
    echo "canary-status: working dir not found: $WORKING_DIR" >&2
    exit 2
fi

# Stale threshold in seconds.
MAX_AGE_SECS=$((MAX_AGE_HOURS * 3600))
NOW=$(date +%s)

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
        if [ "$heartbeat_age" -gt "$MAX_AGE_SECS" ]; then
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
    local latest_entry=""
    if [ "$log_size" != "0" ]; then
        local recent_tail
        recent_tail=$(tail -n 50 "$log_path" 2>/dev/null)
        local signal
        signal=$(printf '%s\n' "$recent_tail" | grep -E -i 'fail|drift|stale|warn|error|behind' | tail -n 1 | head -c 120)
        if [ -n "$signal" ]; then
            latest_entry="$signal"
        else
            latest_entry=$(printf '%s\n' "$recent_tail" | grep -v '^$' | head -n 1 | head -c 120)
        fi
    fi

    printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$name" "$status" "$log_size" "$log_mtime" "$heartbeat_mtime" "$latest_entry"
}

# Build the result set.
RESULTS=""
TOTAL=0
HEALTHY=0
FIRING=0
STALE=0
NO_HB=0

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
    esac
done <<EOF
$(discover_canaries)
EOF

PROBLEMS=$((FIRING + STALE))

# JSON rendering.
if [ "$JSON" = "1" ]; then
    printf '{"ok":true,"summary":{"total":%d,"healthy":%d,"firing":%d,"stale":%d,"no_heartbeat":%d,"max_age_hours":%d},"canary_dir":"%s","resolution":"%s","canaries":[' \
        "$TOTAL" "$HEALTHY" "$FIRING" "$STALE" "$NO_HB" "$MAX_AGE_HOURS" \
        "$(printf '%s' "$WORKING_DIR" | sed 's/\\/\\\\/g; s/"/\\"/g')" "$RESOLUTION"
    first=1
    while IFS=$'\t' read -r name status log_size log_mtime hb_mtime latest_entry; do
        [ -n "$name" ] || continue
        [ "$first" = "1" ] || printf ','
        first=0
        # JSON-escape the latest_entry (minimal: quotes + backslashes + newlines).
        esc=$(printf '%s' "$latest_entry" | sed 's/\\/\\\\/g; s/"/\\"/g; s/	/\\t/g')
        printf '{"name":"%s","status":"%s","log_size":%s,"log_mtime":%s,"heartbeat_mtime":%s,"latest_entry":"%s"}' \
            "$name" "$status" "$log_size" "$log_mtime" "$hb_mtime" "$esc"
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
    echo "canary-status: $PROBLEMS canary(ies) need attention ($FIRING firing, $STALE stale, threshold ${MAX_AGE_HOURS}h)"
    echo "  read: $WORKING_DIR [$RESOLUTION]"
    while IFS=$'\t' read -r name status log_size log_mtime hb_mtime latest_entry; do
        [ -n "$name" ] || continue
        case "$status" in
            FIRING|STALE) ;;
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
echo "canary-status: $TOTAL canary(ies) — $HEALTHY healthy, $FIRING firing, $STALE stale (threshold ${MAX_AGE_HOURS}h)"
echo "  read: $WORKING_DIR [$RESOLUTION]"
echo ""
printf '  %-12s %-32s %s\n' "STATUS" "NAME" "LAST FIRED / LATEST ENTRY"
printf '  %-12s %-32s %s\n' "------" "----" "-------------------------"
while IFS=$'\t' read -r name status log_size log_mtime hb_mtime latest_entry; do
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
    if [ "$STALE" != "0" ]; then
        echo "  STALE  — a canary cron hasn't fired in >${MAX_AGE_HOURS}h. Check that cron is loaded:"
        echo "    sudo cat /etc/cron.d/<canary-name>"
        echo "    sudo systemctl status cron"
        echo "  Then verify the script runs manually:"
        echo "    bash scripts/<canary-script>.sh --quiet"
    fi
fi

[ "$PROBLEMS" = "0" ] && exit 0 || exit 1
