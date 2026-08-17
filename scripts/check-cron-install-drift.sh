#!/usr/bin/env bash
# T-2561 (T-2468 shipped≠live / G-069) — cron-install-drift check.
#
# A canary is only load-bearing if its crontab is actually installed to
# /etc/cron.d. The canary crontabs live git-tracked under .context/cron/*.crontab
# as the source of truth, but installation to /etc/cron.d is a MANUAL operator step
# — so a freshly-committed canary can sit "shipped but dark" (never fires) with
# nothing detecting it. The meta-canary-aliveness check (T-1723) verifies the
# HEARTBEAT of canaries that ARE running; it structurally cannot see one that was
# never scheduled. This check closes that blindness: for every git-tracked crontab
# it reads the crontab's OWN `# Installed to: <path>` header and verifies that path
# exists and matches. Robust to naming exceptions (e.g. agentic-audit.crontab →
# /etc/cron.d/agentic-audit-termlink) because the path is self-declared, not
# derived from a naming rule.
#
# This is a DEPLOY-TIME / preflight check, NOT itself a cron canary — a
# canary-to-detect-uninstalled-canaries would itself need installing (recursive).
# Run it ad-hoc after committing a new canary, or fold it into /preflight.
#
# Classes:
#   MISSING           declared path absent entirely            → FIRES (the G-069 class)
#   UNINSTALLED_JOBS  present, but the installed file lacks cron JOB lines that git
#                     declares                                 → FIRES (T-2682)
#   DRIFT             present, content differs, but no job line is absent (comment
#                     churn, env tweaks, extra installed jobs)  → WARNING; fires only
#                     under --strict
#   OK                present + byte-identical
#
# T-2682 — why UNINSTALLED_JOBS is its own class. T-2561 shipped with MISSING vs DRIFT
# only, so any content difference read as one non-firing "DRIFT (warning)" line. On the
# origin host that hid two crontabs whose installed copies were missing their
# meta-canary job line entirely (T-2175 substrate-preflight, T-2176 fleet-doorbell-mail)
# — the jobs that detect when the canary itself stops firing had never been scheduled.
# A job that was never scheduled is not a cosmetic difference; it is exactly the
# shipped-but-dark condition this check exists to catch, so it fires regardless of
# --strict. Direction matters: git-declared work absent from the host fires; an EXTRA
# job the operator added locally does not (that is their prerogative).
#
# Job lines are compared with comments, blank lines, and `VAR=value` env assignments
# stripped and internal whitespace collapsed, so a reformat alone is never a false
# positive.
#
# Exit codes: 0 healthy · 1 firing (missing / uninstalled jobs / drift under --strict)
#             · 2 tooling error
set -u

SRC_DIR="${CRON_DRIFT_SRC_DIR:-.context/cron}"
INSTALLED_DIR="${CRON_DRIFT_INSTALLED_DIR:-}"   # test hook: remap install root
QUIET=0
FORMAT=human
STRICT=0

usage() {
    sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: check-cron-install-drift.sh [OPTIONS]
  --strict     Also fire (exit 1) on plain DRIFT. Does NOT affect MISSING or
               UNINSTALLED_JOBS — those always fire.
  --json       Emit a JSON envelope
  --quiet      Print only on firing (cron-friendly)
  -h, --help   This help

Test hooks: CRON_DRIFT_SRC_DIR=<dir> (git crontab source, default .context/cron),
CRON_DRIFT_INSTALLED_DIR=<dir> (remaps each declared install path's dirname to this
dir, for host-independent fixtures).
Fixtures: bash tests/cron-install-drift-fixtures.sh

Exit: 0 healthy · 1 firing (missing / uninstalled jobs / drift-under-strict)
      · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --strict) STRICT=1; shift ;;
        --json)   FORMAT=json; shift ;;
        --quiet)  QUIET=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "check-cron-install-drift: unknown arg: $1" >&2; exit 2 ;;
    esac
done

if [ ! -d "$SRC_DIR" ]; then
    echo "check-cron-install-drift: source dir not found: $SRC_DIR" >&2
    exit 2
fi

# If the real /etc/cron.d does not exist and no test hook remaps it, this host does
# not use cron.d (macOS / dev container) — informational, never firing.
if [ -z "$INSTALLED_DIR" ] && [ ! -d /etc/cron.d ]; then
    [ "$QUIET" -eq 1 ] || echo "check-cron-install-drift: /etc/cron.d absent — host does not use cron.d (informational, not firing)"
    exit 0
fi

# resolve the effective installed path for a declared header path.
resolve_installed() {
    local declared="$1"
    if [ -n "$INSTALLED_DIR" ]; then
        echo "$INSTALLED_DIR/$(basename "$declared")"
    else
        echo "$declared"
    fi
}

# T-2682 — extract executable cron JOB lines from a crontab file: drop blank lines,
# `#`-comments, and `VAR=value` env assignments; collapse internal whitespace so a
# reformat alone is never a false positive. What remains is work that is supposed to
# be scheduled — the only difference class that means "shipped but dark".
job_lines() {
    sed -E 's/[[:space:]]+$//' "$1" 2>/dev/null \
        | grep -vE '^[[:space:]]*(#|$)' \
        | grep -vE '^[[:space:]]*[A-Za-z_][A-Za-z0-9_]*=' \
        | sed -E 's/[[:space:]]+/ /g; s/^ //'
}

# T-2787 — strip a job line's output redirection, leaving schedule + user + command.
# Two lines that agree here are THE SAME SCHEDULED WORK routed differently; the job runs
# either way. This is what separates "this work is not scheduled at all" (UNINSTALLED_JOBS)
# from "this work IS scheduled, but the installed command differs" (JOB_DRIFT). Both fire —
# the distinction is the CLAIM, not the severity. Reporting the second as the first sends
# the operator to re-install crontabs that are already running.
strip_redirects() {
    sed -E 's/[[:space:]]*[0-9]?>>?.*$//'
}

missing=(); drifted=(); skipped=(); ok_count=0
uninstalled=(); uninstalled_detail=()
job_drift=(); job_drift_detail=()
for f in "$SRC_DIR"/*.crontab; do
    [ -e "$f" ] || continue
    declared="$(grep -iE '^#[[:space:]]*Installed to:' "$f" | head -1 | sed -E 's/.*[Ii]nstalled to:[[:space:]]*//; s/[[:space:]].*//')"
    if [ -z "$declared" ]; then
        skipped+=("$(basename "$f") (no 'Installed to:' header)")
        continue
    fi
    inst="$(resolve_installed "$declared")"
    if [ ! -f "$inst" ]; then
        missing+=("$(basename "$f") → $declared")
    elif ! diff -q "$inst" "$f" >/dev/null 2>&1; then
        # T-2682 — split drift by DIRECTION. A git job line absent from the
        # installed file is scheduled work that is not scheduled: shipped-dark,
        # the exact G-069 class this check exists for, so it FIRES. Everything
        # else (comment churn, env tweaks, extra installed jobs) stays a warning.
        # Extra lines present only in the INSTALLED file are deliberately NOT a
        # firing condition — an operator adding a local job is their prerogative.
        gitjobs="$(mktemp)"; instjobs="$(mktemp)"; instbare="$(mktemp)"
        job_lines "$f"    > "$gitjobs"
        job_lines "$inst" > "$instjobs"
        strip_redirects < "$instjobs" > "$instbare"
        absent="$(grep -Fxv -f "$instjobs" "$gitjobs" 2>/dev/null || true)"

        # T-2787 — split the "absent" set once more. A git job line whose
        # redirect-stripped form IS present in the installed file is scheduled work
        # that runs; only its output routing differs (the T-2685 stderr-split shape).
        # Calling that "not scheduled on this host" is a false claim that sends the
        # operator to re-install crontabs that are already firing.
        this_missing=(); this_drift=()
        while IFS= read -r l; do
            [ -n "$l" ] || continue
            bare="$(printf '%s\n' "$l" | strip_redirects)"
            if [ -n "$bare" ] && grep -Fxq "$bare" "$instbare" 2>/dev/null; then
                inst_line="$(grep -F -m1 "$bare" "$instjobs" 2>/dev/null || true)"
                this_drift+=("$(basename "$f")|$declared|$l|$inst_line")
            else
                this_missing+=("$(basename "$f")|$declared|$l")
            fi
        done <<< "$absent"
        rm -f "$gitjobs" "$instjobs" "$instbare"

        if [ "${#this_missing[@]}" -gt 0 ]; then
            uninstalled+=("$(basename "$f") ↔ $declared")
            uninstalled_detail+=("${this_missing[@]}")
        fi
        if [ "${#this_drift[@]}" -gt 0 ]; then
            job_drift+=("$(basename "$f") ↔ $declared")
            job_drift_detail+=("${this_drift[@]}")
        fi
        if [ "${#this_missing[@]}" -eq 0 ] && [ "${#this_drift[@]}" -eq 0 ]; then
            drifted+=("$(basename "$f") ↔ $declared")
        fi
    else
        ok_count=$((ok_count+1))
    fi
done

miss_n=${#missing[@]}; drift_n=${#drifted[@]}; skip_n=${#skipped[@]}
uninst_n=${#uninstalled[@]}; jobdrift_n=${#job_drift[@]}

if [ "$FORMAT" = json ]; then
    jarr() { local first=1; printf '['; for x in "$@"; do [ $first -eq 1 ] || printf ','; printf '%s' "$(printf '%s' "$x" | jq -R .)"; first=0; done; printf ']'; }
    # T-2682: uninstalled job lines fire unconditionally — they are the G-069
    # class, not a cosmetic difference, so --strict is irrelevant to them.
    # T-2787: job-drift fires on the same footing. The installed command differing
    # from git IS a deployment gap (T-2685's redirect is load-bearing); only the
    # CLAIM differs from UNINSTALLED_JOBS, not the severity.
    fire=$({ [ "$miss_n" -gt 0 ] || [ "$uninst_n" -gt 0 ] || [ "$jobdrift_n" -gt 0 ]; } && echo true \
        || { [ "$STRICT" -eq 1 ] && [ "$drift_n" -gt 0 ] && echo true || echo false; })
    printf '{"ok":%s,"missing_count":%s,"uninstalled_jobs_count":%s,"job_drift_count":%s,"drift_count":%s,"ok_count":%s,"skipped_count":%s,"strict":%s,"missing":%s,"uninstalled_jobs":%s,"job_drift":%s,"drifted":%s}\n' \
        "$([ "$fire" = true ] && echo false || echo true)" \
        "$miss_n" "$uninst_n" "$jobdrift_n" "$drift_n" "$ok_count" "$skip_n" \
        "$([ "$STRICT" -eq 1 ] && echo true || echo false)" \
        "$(jarr "${missing[@]}")" "$(jarr "${uninstalled_detail[@]}")" \
        "$(jarr "${job_drift_detail[@]}")" "$(jarr "${drifted[@]}")"
    [ "$fire" = true ] && exit 1 || exit 0
fi

fired=0
if [ "$miss_n" -gt 0 ]; then
    fired=1
    echo "check-cron-install-drift: $miss_n git-tracked crontab(s) NOT installed — SHIPPED BUT DARK (G-069, T-2561):"
    for m in "${missing[@]}"; do echo "  MISSING: $m"; done
    echo "  Remediation: install each with  sudo cp .context/cron/<name>.crontab <declared-path>  (root)."
fi
if [ "$uninst_n" -gt 0 ]; then
    fired=1
    echo "check-cron-install-drift: $uninst_n installed crontab(s) are MISSING JOB LINES that git declares — SHIPPED BUT DARK (G-069, T-2682):"
    for u in "${uninstalled[@]}"; do echo "  UNINSTALLED_JOBS: $u"; done
    echo "  The scheduled work below exists in git but is NOT scheduled on this host:"
    for d in "${uninstalled_detail[@]}"; do
        echo "    ↳ ${d%%|*}: $(printf '%s' "$d" | cut -d'|' -f3-)"
    done
    echo "  Remediation: re-install the affected crontab(s) with"
    echo "    sudo cp .context/cron/<name>.crontab <declared-path>   (root)"
    echo "  This fires regardless of --strict: a job that was never scheduled is not a cosmetic difference."
fi
if [ "$jobdrift_n" -gt 0 ]; then
    fired=1
    echo "check-cron-install-drift: $jobdrift_n installed crontab(s) run a DIFFERENT COMMAND than git declares — UNDEPLOYED CHANGE (T-2787):"
    for j in "${job_drift[@]}"; do echo "  JOB_DRIFT: $j"; done
    echo "  These jobs ARE scheduled and ARE running — the schedule matches, the command does not."
    echo "  (Distinct from UNINSTALLED_JOBS above, which is work that is not scheduled at all.)"
    for d in "${job_drift_detail[@]}"; do
        echo "    ↳ ${d%%|*}:"
        echo "        git:       $(printf '%s' "$d" | cut -d'|' -f3)"
        echo "        installed: $(printf '%s' "$d" | cut -d'|' -f4)"
    done
    echo "  Remediation: re-install the affected crontab(s) with"
    echo "    sudo cp .context/cron/<name>.crontab <declared-path>   (root)"
    echo "  This fires regardless of --strict: a command that differs from git is an undeployed change,"
    echo "  not a cosmetic one — the T-2685 stderr-split is exactly this shape and is load-bearing."
fi
if [ "$drift_n" -gt 0 ]; then
    [ "$STRICT" -eq 1 ] && fired=1
    lvl=$([ "$STRICT" -eq 1 ] && echo "DRIFT (firing under --strict)" || echo "DRIFT (warning)")
    echo "check-cron-install-drift: $drift_n installed crontab(s) differ from git source — $lvl:"
    for d in "${drifted[@]}"; do echo "  DRIFT: $d"; done
    echo "  Remediation: re-install from git source, or reconcile the /etc/cron.d edit back into .context/cron/."
fi
if [ "$skip_n" -gt 0 ] && [ "$QUIET" -ne 1 ]; then
    for s in "${skipped[@]}"; do echo "  skipped: $s"; done
fi
if [ "$fired" -eq 0 ]; then
    [ "$QUIET" -eq 1 ] || echo "check-cron-install-drift: healthy ($ok_count installed + matching, $drift_n drift-warning, $skip_n skipped)"
    exit 0
fi
exit 1
