#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-canary-log-hygiene.sh (T-2685, G-019 prevention for the T-2683 F2 class)
#
# THE DEFECT THIS GUARDS
#
# Every canary in this repo implements the same three-way exit contract:
#
#     exit 0  healthy · exit 1  FIRING (a real finding) · exit 2  tooling error
#
# and the exit-2 class is deliberate. T-2557 states why, for the session-control
# canary, in words that apply to all of them:
#
#     "This split keeps a firing log meaningful: it fills ONLY when session control
#      genuinely broke, never on a transient hub-down."
#
# The crontab used to throw that split away. Every canary job line was written as:
#
#     … bash scripts/check-<x>.sh --quiet >> .context/working/.<x>-canary.log 2>&1
#                                                                            ^^^^^
#
# `2>&1` merges stderr into the findings log. A check that COULD NOT RUN then writes
# into the log whose entire documented meaning is "the thing you are watching is
# broken". T-2683 found this on 19 of 19 canary job lines, with a live instance:
# `.release-mirror-canary.log` contained `error: origin HEAD empty` while the script
# itself exited 0 with "GitHub mirror: synced". Per CLAUDE.md that non-empty log
# directs an operator to rotate a GitHub token — real work, wrong diagnosis, no
# underlying fault.
#
# The compounding harm is worse than one false positive. "Empty log = healthy" is a
# ONE-BIT channel. Once a tooling error dirties the log, a subsequent GENUINE finding
# appends to an already-non-empty file and changes nothing an operator can see. The
# canary built to prevent a 16-day silent mirror failure (G-058) was itself in a state
# where a real mirror failure was indistinguishable from the noise already in its log.
#
# THE CORRECT IDIOM, which this check enforces:
#
#     … bash scripts/check-<x>.sh --quiet >> <log> 2>> <log>.stderr
#
# stdout (findings) and stderr (tooling) land in separate files. Nothing is lost —
# the diagnostic stream is preserved, just not conflated with the signal. The
# `.stderr` suffix deliberately does NOT match `/canaries`' `.*-canary.log`
# discovery glob (canary-status.sh:97), so a tooling error can never be re-read as
# a finding by the operator's own tooling.
#
# `2>/dev/null` is ALSO rejected. It fixes the false-positive half and creates a
# worse problem: a canary that cannot run now fails completely silently, which is
# Directive #2's "no silent failures" violated in the monitoring layer itself.
# Discarding the evidence is not hygiene.
#
# SCOPE. Only *findings* logs are in scope — the `.context/working/.*.log` files the
# canary convention treats as one-bit signals. An operator appending stderr to a
# scratch or debug log is not this defect and must not fire.
#
# Exit codes: 0 clean · 1 a job line merges or discards stderr on a findings log
#             · 2 tooling error (fail-closed)
set -uo pipefail

SRC_DIR="${CANARY_HYGIENE_SRC_DIR:-.context/cron}"
QUIET=0
FORMAT=human

usage() {
    cat <<'EOF'
check-canary-log-hygiene.sh — canary cron job lines must not merge or discard the
tooling-error stream into a findings log.

Correct:   … check-x.sh --quiet >> <log> 2>> <log>.stderr
Rejected:  … >> <log> 2>&1          (tooling error reads as a finding)
           … >> <log> 2>/dev/null   (tooling error vanishes silently)

Usage: check-canary-log-hygiene.sh [OPTIONS]
  --json          Emit {ok, firing[], checked, crontabs}
  --quiet         Print only on firing (cron-friendly)
  --no-heartbeat  Accepted for guard-layer parity; this check writes no heartbeat
  -h, --help      This help

Test seam: CANARY_HYGIENE_SRC_DIR=<dir> (crontab source, default .context/cron).
Fixtures:  bash tests/canary-log-hygiene-fixtures.sh

Exit: 0 clean · 1 a merging/discarding job line · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "check-canary-log-hygiene: unknown arg: $1" >&2; exit 2 ;;
    esac
done

if [ ! -d "$SRC_DIR" ]; then
    echo "check-canary-log-hygiene: source dir not found: $SRC_DIR" >&2
    exit 2
fi

firing=(); checked=0; crontabs=0

for f in "$SRC_DIR"/*.crontab; do
    [ -e "$f" ] || continue
    crontabs=$((crontabs+1))
    base="$(basename "$f")"
    # Executable cron job lines only: drop comments, blanks, and VAR=value env lines.
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        # Only lines that append to a findings log under the working dir are in scope.
        case "$line" in
            *">>"*".context/working/"*.log*) ;;
            *) continue ;;
        esac
        checked=$((checked+1))
        why=""
        case "$line" in
            *"2>&1"*)      why="merges stderr into the findings log (2>&1) — a tooling error reads as a finding" ;;
            *"2>/dev/null"*) why="discards stderr (2>/dev/null) — a canary that cannot run fails silently" ;;
        esac
        [ -n "$why" ] && firing+=("$base|$line|$why")
    done <<EOF
$(grep -vE '^[[:space:]]*(#|$)' "$f" 2>/dev/null | grep -vE '^[[:space:]]*[A-Za-z_][A-Za-z0-9_]*=')
EOF
done

n=${#firing[@]}

if [ "$FORMAT" = json ]; then
    printf '{"ok":%s,"firing":[' "$([ "$n" -eq 0 ] && echo true || echo false)"
    i=0
    while [ "$i" -lt "$n" ]; do
        [ "$i" -eq 0 ] || printf ','
        e="${firing[$i]}"
        printf '{"crontab":%s,"line":%s,"why":%s}' \
            "$(printf '%s' "${e%%|*}" | jq -R .)" \
            "$(printf '%s' "$e" | cut -d'|' -f2 | jq -R .)" \
            "$(printf '%s' "$e" | cut -d'|' -f3- | jq -R .)"
        i=$((i+1))
    done
    printf '],"checked":%s,"crontabs":%s}\n' "$checked" "$crontabs"
    [ "$n" -eq 0 ] && exit 0 || exit 1
fi

if [ "$n" -eq 0 ]; then
    [ "$QUIET" -eq 1 ] || echo "check-canary-log-hygiene: clean — $checked findings-log job line(s) across $crontabs crontab(s) keep stdout and stderr separate."
    exit 0
fi

echo "check-canary-log-hygiene: FIRING — $n cron job line(s) conflate the tooling-error stream with the findings log:"
for e in "${firing[@]}"; do
    echo "  ↳ ${e%%|*}: $(printf '%s' "$e" | cut -d'|' -f3-)"
    echo "      $(printf '%s' "$e" | cut -d'|' -f2)"
done
cat <<'EOF'
  Why this matters: "empty log = healthy" is a one-bit channel. A tooling error
  written into it is indistinguishable from a real finding, AND permanently
  destroys the signal — a genuine finding afterwards appends to an already-dirty
  log and changes nothing the operator can see.
  Fix: route the two streams to two files —
      … >> .context/working/.<name>-canary.log 2>> .context/working/.<name>-canary.log.stderr
  Do NOT use 2>/dev/null: that trades a false positive for a silent failure.
EOF
exit 1
