#!/usr/bin/env bash
# T-1723 / T-2175 — Meta-canary: detect when a watched canary has stopped
# running while the underlying drift it watches for may still be present.
#
# Why this exists: G-058 ran 16 days silently because nothing watched the
# watcher. T-1696 installed the mirror watcher; T-2160 installed the
# substrate-preflight watcher. This script is the meta-watcher for both
# (T-2175 parameterized it). Failure modes it catches: cron entry failed
# to load (parse error, wrong permissions, moved file), canary script
# crashed, log path moved.
#
# Mechanism: the watched canary touches a heartbeat file on every
# invocation. This script stats that file's mtime. If older than the
# threshold (default 48h, twice the daily cron interval), exit 1 with a
# diagnostic. If fresh, exit 0.
#
# Env-parameterized so one script serves both canaries (defaults preserve
# original mirror-canary behavior — backward compatible):
#   HEARTBEAT_FILE     Path to heartbeat file
#                      (default: .context/working/.release-mirror-canary.heartbeat)
#   CANARY_NAME        Human-readable name appearing in diagnostics
#                      (default: "release-mirror canary")
#   CANARY_PROBE_CMD   Command to run on stale to fold in current drift status
#                      (default: bash scripts/check-mirror-freshness.sh --quiet --no-heartbeat)
#                      The probe's rc maps: 0=synced, 1=drift, 2=net-error, *=unknown.
#                      Set empty to skip the drift-fold entirely.
#   CANARY_CRON_PATH   /etc/cron.d path appearing in the diagnostic hint
#                      (default: /etc/cron.d/termlink-release-mirror-canary)
#
# SWEEP MODE (--all, T-2878): walks EVERY cron-scheduled canary in one run instead
# of watching one named canary. Two firing classes — STALE (heartbeat older than
# the threshold) and NO-HEARTBEAT (cron-scheduled but writes no heartbeat, so it
# cannot be watched at all). Canaries git schedules no cron for are excluded, not
# reported dead. Fails closed on an empty corpus. See the block above sweep_all().
#
# Exit codes:
#   0 — canary alive (heartbeat fresh) / all swept canaries alive
#   1 — canary stale (heartbeat older than threshold) — operator action required
#   2 — tooling error (stat failed, heartbeat path absent, empty sweep corpus)
#
# Usage:
#   check-canary-aliveness.sh                  # human-readable, mirror canary
#   check-canary-aliveness.sh --quiet          # only print on staleness (cron-friendly)
#   check-canary-aliveness.sh --max-age-hours 72   # custom threshold
#   check-canary-aliveness.sh --all            # sweep every cron-scheduled canary
#   check-canary-aliveness.sh --all --quiet    # sweep, print only on firing (cron)
#   HEARTBEAT_FILE=.context/working/.substrate-preflight-canary.heartbeat \
#     CANARY_NAME="substrate-preflight canary" \
#     CANARY_PROBE_CMD="bash scripts/substrate-preflight.sh --quiet --no-heartbeat" \
#     CANARY_CRON_PATH=/etc/cron.d/termlink-substrate-preflight-canary \
#     check-canary-aliveness.sh --quiet         # meta-canary for substrate (T-2175)

set -eu

HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.release-mirror-canary.heartbeat}"
CANARY_NAME="${CANARY_NAME:-release-mirror canary}"
CANARY_PROBE_CMD="${CANARY_PROBE_CMD:-bash scripts/check-mirror-freshness.sh --quiet --no-heartbeat}"
CANARY_CRON_PATH="${CANARY_CRON_PATH:-/etc/cron.d/termlink-release-mirror-canary}"
MAX_AGE_HOURS=48
QUIET=0
ALL=0

# Sweep-mode seams (PL-213) — fixtures point these at fixture dirs so the whole
# sweep runs with no live cron and no real canary.
SWEEP_WORKING_DIR="${CANARY_SWEEP_WORKING_DIR:-.context/working}"
SWEEP_CRON_DIR="${CANARY_SWEEP_CRON_DIR:-.context/cron}"

while [ $# -gt 0 ]; do
    case "$1" in
        --all) ALL=1 ;;
        --quiet) QUIET=1 ;;
        --max-age-hours)
            shift
            [ $# -ge 1 ] || { echo "--max-age-hours requires a value" >&2; exit 2; }
            MAX_AGE_HOURS="$1"
            ;;
        --max-age-hours=*) MAX_AGE_HOURS="${1#*=}" ;;
        -h|--help)
            # Through line 46 so the sweep flags and exit codes are actually in
            # --help. The range was 2,25 and every T-2878 addition landed past
            # it, which would have documented --all everywhere except where a
            # user looks for it.
            sed -n '2,46p' "$0"
            exit 0
            ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

case "$MAX_AGE_HOURS" in
    ''|*[!0-9]*) echo "--max-age-hours must be a positive integer, got: $MAX_AGE_HOURS" >&2; exit 2 ;;
esac

# ─────────────────────────────────────────────────────────────────────────────
# SWEEP MODE (--all, T-2878)
#
# The per-canary mode above must be wired as its own cron job line, with
# HEARTBEAT_FILE/CANARY_NAME/CANARY_PROBE_CMD set per canary. Measured when this
# was written: 8 of 20 canary crontabs declared such a job and 12 did not, so
# most canaries had nothing watching them — if their cron stopped, nothing would
# notice, which is the G-058 "16 days silent" failure this script exists for.
#
# The obvious remediation — add 12 more near-identical job lines — is what
# produced the gap in the first place: a per-canary wiring step is a step someone
# forgets, and forgetting is silent. One sweep over every heartbeat cannot be
# forgotten for an individual canary, because it never names them individually.
#
# The per-canary jobs are deliberately left in place. They carry a
# CANARY_PROBE_CMD that folds the watched drift's CURRENT status into the stale
# report, which a generic sweep cannot do; this is a coverage net beneath them,
# not a replacement for them.
sweep_stat_mtime() {
    stat -c %Y "$1" 2>/dev/null || stat -f %m "$1" 2>/dev/null || echo 0
}

# A genuine canary crontab names the log it appends to. Borrowed verbatim in
# spirit from canary-status.sh::is_cron_scheduled, and it is load-bearing here
# for the same reason it is there (T-2826): the source-level static checks
# (alloc-sink, busy-spin, drain-sink, silent-exit) leave a heartbeat behind when
# somebody runs them by hand but have no crontab and no schedule. Without this
# predicate the sweep reports each of them STALE forever, and a sweep that is
# mostly false is a sweep nobody reads.
sweep_is_cron_scheduled() {
    grep -q -- "\.$1\.log" "$SWEEP_CRON_DIR"/*.crontab 2>/dev/null
}

sweep_all() {
    [ -d "$SWEEP_WORKING_DIR" ] || { echo "sweep: working dir absent: $SWEEP_WORKING_DIR" >&2; exit 2; }
    [ -d "$SWEEP_CRON_DIR" ]    || { echo "sweep: cron source dir absent: $SWEEP_CRON_DIR" >&2; exit 2; }

    local names="" f base name
    # Union of both sets. Walking heartbeats ALONE is the trap this mode exists to
    # avoid: a canary that writes no heartbeat is invisible to a heartbeat walk, so
    # the sweep would report "all alive" while that canary sits unwatched — most
    # confident exactly where it is most wrong (T-2680). Logs are walked too, and a
    # canary present as a log with no heartbeat companion is its own firing class.
    for f in "$SWEEP_WORKING_DIR"/.*-canary.heartbeat "$SWEEP_WORKING_DIR"/.*-canary.log; do
        [ -e "$f" ] || continue
        base="${f##*/.}"; name="${base%.heartbeat}"; name="${name%.log}"
        case " $names " in *" $name "*) ;; *) names="$names $name" ;; esac
    done

    local now checked=0 excluded=0 alive=0
    local stale_list="" nohb_list=""
    now=$(date +%s)

    for name in $names; do
        if ! sweep_is_cron_scheduled "$name"; then
            excluded=$((excluded + 1)); continue
        fi
        checked=$((checked + 1))
        local hb="$SWEEP_WORKING_DIR/.$name.heartbeat" hb_mtime age_h
        if [ ! -e "$hb" ]; then
            nohb_list="$nohb_list $name"; continue
        fi
        hb_mtime=$(sweep_stat_mtime "$hb")
        age_h=$(( (now - hb_mtime) / 3600 ))
        if [ "$hb_mtime" = "0" ] || [ $(( now - hb_mtime )) -gt $(( MAX_AGE_HOURS * 3600 )) ]; then
            stale_list="$stale_list $name:$age_h"
        else
            alive=$((alive + 1))
        fi
    done

    # Fail closed on an empty corpus. "0 dead out of 0" is vacuously true and would
    # report green over a discovery path that silently stopped matching — the same
    # zero-census lesson as T-2747, and the precise false assurance this mode exists
    # to remove.
    if [ "$checked" -eq 0 ]; then
        echo "sweep: no cron-scheduled canaries discovered under $SWEEP_WORKING_DIR — refusing to report clean" >&2
        exit 2
    fi

    if [ -z "$stale_list" ] && [ -z "$nohb_list" ]; then
        [ "$QUIET" = 1 ] || echo "Canary sweep: all $checked cron-scheduled canaries alive (threshold ${MAX_AGE_HOURS}h; $excluded unscheduled heartbeat(s) excluded)"
        return 0
    fi

    echo "CANARY SWEEP FIRING: $checked cron-scheduled canaries checked, $alive alive"
    for name in $nohb_list; do
        echo "  [NO-HEARTBEAT] $name — cron-scheduled but writes no heartbeat, so nothing"
        echo "      can tell a dead cron from a healthy one, and canary-status.sh classifies"
        echo "      it FIRING on any non-empty log with no arm that returns it to HEALTHY."
        # The canary NAME carries a trailing "-canary" (it comes from the log
        # filename); the script implementing it does not. Emitting the raw name
        # here produced a hint globbing a path that cannot exist, and a
        # remediation line that matches nothing is worse than none — it reads as
        # authoritative and sends the reader looking for a missing file.
        echo "      Fix: touch a heartbeat at the TOP of scripts/*${name%-canary}*.sh (see T-1723)."
    done
    for entry in $stale_list; do
        echo "  [STALE] ${entry%%:*} — heartbeat ${entry##*:}h old (threshold ${MAX_AGE_HOURS}h)"
        echo "      Likely cause: cron entry failed to load, or the canary script broke."
        echo "      Diagnostic: ls -la /etc/cron.d/ | grep ${entry%%:*}"
    done
    return 1
}

if [ "$ALL" -eq 1 ]; then
    sweep_all
    exit $?
fi

if [ ! -e "$HEARTBEAT_FILE" ]; then
    echo "CANARY HEARTBEAT ABSENT ($CANARY_NAME): $HEARTBEAT_FILE" >&2
    echo "  Either the canary has never run since the heartbeat-touch landed, or the canary script predates it." >&2
    if [ -n "$CANARY_PROBE_CMD" ]; then
        echo "  Manual run to seed it: $CANARY_PROBE_CMD" >&2
    fi
    exit 1
fi

now=$(date +%s)
if hb_mtime=$(stat -c %Y "$HEARTBEAT_FILE" 2>/dev/null); then
    :
elif hb_mtime=$(stat -f %m "$HEARTBEAT_FILE" 2>/dev/null); then
    :
else
    echo "stat failed on $HEARTBEAT_FILE (neither GNU -c nor BSD -f worked)" >&2
    exit 2
fi

age_seconds=$(( now - hb_mtime ))
age_hours=$(( age_seconds / 3600 ))
threshold_seconds=$(( MAX_AGE_HOURS * 3600 ))

if [ "$age_seconds" -le "$threshold_seconds" ]; then
    [ "$QUIET" = 1 ] || echo "Canary alive ($CANARY_NAME): heartbeat is ${age_hours}h old (threshold ${MAX_AGE_HOURS}h)"
    exit 0
fi

# Stale. Try to fold in current drift status so the operator sees both signals at once.
# CANARY_PROBE_CMD may be empty (caller opted out of fold-in).
probe_status=unchecked
if [ -n "$CANARY_PROBE_CMD" ]; then
    if bash -c "$CANARY_PROBE_CMD" >/dev/null 2>&1; then
        probe_status=synced
    else
        rc=$?
        case "$rc" in
            1) probe_status=drift ;;
            2) probe_status=net-error ;;
            *) probe_status="unknown(rc=$rc)" ;;
        esac
    fi
fi

echo "CANARY STALE ($CANARY_NAME): heartbeat is ${age_hours}h old (threshold ${MAX_AGE_HOURS}h)"
echo "  Heartbeat file: $HEARTBEAT_FILE"
echo "  Probe status:   $probe_status"
echo "  Likely cause:   cron entry failed to load, OR the canary script broke."
echo "  Diagnostic:"
echo "    ls -la $CANARY_CRON_PATH"
if [ -n "$CANARY_PROBE_CMD" ]; then
    echo "    $CANARY_PROBE_CMD  # manual run to repopulate heartbeat (drop --no-heartbeat if present)"
fi
echo "    journalctl --since '48 hours ago' -u cron 2>/dev/null | grep -i canary"
exit 1
