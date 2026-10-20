#!/usr/bin/env bash
# reap-zombie-sessions.sh — T-3491: terminate idle zombie `termlink register --shell`
# sessions. Moved out of runme.sh action 7 (T-3297, T-3291 S3b) so a daily cron can run
# the same code: the session-leak canary fired four times between runme runs (2026-10-05
# .. 2026-10-20) because nothing else reaped them.
#
# Targets come from scripts/lib/session-zombies.py, the SAME detector as the daily
# session-leak canary, so this terminates exactly what the canary flags: a tmux pane,
# older than 24 h, shell with no child, session not attached, launcher not alive.
# Systemd-supervised agents are never targets.
#
# Safety: every target is listed BEFORE any is touched; right before each signal
# /proc/<pid>/cmdline is re-read and must still be `termlink register --name <that
# name>` (pid reuse guard) — a mismatch is skipped and reported. SIGTERM only, never
# SIGKILL: a post-T-3293 register removes its own files on SIGTERM and its tmux pane
# closes; an older binary dies by default and the hub sweep removes what it leaves. A
# target still alive after the bound is FAILED, not escalated.
#
# Usage: reap-zombie-sessions.sh [--dry-run] [--quiet]
#   --quiet  print nothing when there is nothing to reap (cron)
# Exit: 0 reaped (or nothing to reap) · 1 a target survived SIGTERM, or the detector
#       could not run / its output was unreadable (fail-closed: nothing reaped).
# Seams (fixtures only; runme's names, kept so its fixtures cover this script):
#   RUNME_ZOMBIE_DETECT (command printing detector JSON), RUNME_ZOMBIE_PROC (proc root),
#   RUNME_ZOMBIE_KILL (signal command), RUNME_ZOMBIE_WAIT_SECS.
set -uo pipefail

DRY_RUN=0 QUIET=0
for a in "$@"; do
    case "$a" in
        --dry-run) DRY_RUN=1 ;;
        --quiet) QUIET=1 ;;
        -h|--help) sed -n '2,27p' "$0"; exit 0 ;;
        *) echo "reap-zombie-sessions: unknown argument: $a" >&2; exit 1 ;;
    esac
done

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ZOMBIE_DETECT="${RUNME_ZOMBIE_DETECT:-python3 $ROOT/scripts/lib/session-zombies.py}"
ZOMBIE_PROC="${RUNME_ZOMBIE_PROC:-/proc}"
ZOMBIE_KILL="${RUNME_ZOMBIE_KILL:-kill -TERM}"
ZOMBIE_WAIT="${RUNME_ZOMBIE_WAIT_SECS:-10}"

say() { echo "$@"; }

zombie_still_is() {  # <pid> <name> -> 0 if the pid is still that register session
    local cl
    # T-3301: the redirect sits inside a group whose stderr is discarded — a bare
    # `< file 2>/dev/null` reports the failed open BEFORE the 2> applies, which
    # printed one bash error per exited session into the operator's log.
    cl="$( { tr '\0' ' ' < "$ZOMBIE_PROC/$1/cmdline"; } 2>/dev/null )" || return 1
    case "$cl" in *termlink*" register "*"--name $2 "*) return 0 ;; esac
    return 1
}

if ! json="$($ZOMBIE_DETECT 2>/dev/null)"; then
    say "  FAILED  zombie detector could not run — nothing reaped"; exit 1
fi
list="$(printf '%s' "$json" | python3 -c '
import json,sys
for z in json.load(sys.stdin).get("zombie",[]):
    print(z["pid"], z["name"], z["age_s"]//86400, z.get("tmux_session") or "-")')" || {
    say "  FAILED  zombie detector output unreadable — nothing reaped"; exit 1; }
n="$(printf '%s' "$list" | grep -c . || true)"
if [ "$n" = "0" ]; then
    [ "$QUIET" = 1 ] || say "  skip    no zombie sessions (idle >24h, detached, shell childless)"
    exit 0
fi
say "  targets $n zombie session(s) — listed before any is touched:"
printf '%s\n' "$list" | while read -r pid name days tm; do
    say "            pid $pid  $name  ${days}d  tmux=$tm"
done
if [ "$DRY_RUN" = "1" ]; then
    say "  [DRY]   would SIGTERM these $n session(s) after re-verifying each pid"; exit 0
fi
gone=0 skipped=0 stuck=""
while read -r pid name days tm; do
    if ! zombie_still_is "$pid" "$name"; then
        skipped=$((skipped+1)); say "  skip    pid $pid is no longer register '$name' — not signalled"; continue
    fi
    $ZOMBIE_KILL "$pid" 2>/dev/null || true
    for _ in $(seq 1 "$ZOMBIE_WAIT"); do
        zombie_still_is "$pid" "$name" || break
        sleep 1
    done
    if zombie_still_is "$pid" "$name"; then stuck="$stuck $pid($name)"; else gone=$((gone+1)); fi
done <<< "$list"
if [ -z "$stuck" ]; then
    note=""; [ "$skipped" -gt 0 ] && note="; $skipped skipped (pid no longer that session)"
    say "  OK      reaped and verified: $gone session(s) exited on SIGTERM$note"; exit 0
fi
say "  FAILED  still running after SIGTERM (not escalated to SIGKILL):$stuck — $gone exited"
exit 1
