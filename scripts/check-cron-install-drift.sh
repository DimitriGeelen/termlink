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
# T-2821: --lenient suppresses DRIFT firing even when --strict asked for it. It
# never suppresses MISSING or UNINSTALLED_JOBS.
LENIENT=0

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
        --strict)  STRICT=1; shift ;;
        --lenient) LENIENT=1; shift ;;
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

missing=(); drifted=(); skipped=(); ok_count=0
uninstalled=(); uninstalled_detail=(); acknowledged=()

# T-2821 allowlist. A crontab basename listed here is a DELIBERATE host-local
# variation, so its plain DRIFT is reported but never fires. It cannot suppress
# MISSING or UNINSTALLED_JOBS — a dark canary is not a variation.
ALLOWLIST="${CRON_DRIFT_ALLOWLIST:-.context/working/.cron-drift-allowlist}"
ACK_NAMES=""
if [ -r "$ALLOWLIST" ]; then
    ACK_NAMES="$(sed -e 's/#.*//' -e 's/[[:space:]]*$//' -e 's/^[[:space:]]*//' "$ALLOWLIST" 2>/dev/null | grep -v '^$' || true)"
fi
is_acknowledged() {
    [ -n "$ACK_NAMES" ] || return 1
    printf '%s\n' "$ACK_NAMES" | grep -qxF "$1"
}
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
        gitjobs="$(mktemp)"; instjobs="$(mktemp)"
        job_lines "$f"    > "$gitjobs"
        job_lines "$inst" > "$instjobs"
        absent="$(grep -Fxv -f "$instjobs" "$gitjobs" 2>/dev/null || true)"
        rm -f "$gitjobs" "$instjobs"
        if [ -n "$absent" ]; then
            uninstalled+=("$(basename "$f") ↔ $declared")
            while IFS= read -r l; do
                [ -n "$l" ] && uninstalled_detail+=("$(basename "$f")|$declared|$l")
            done <<< "$absent"
        elif is_acknowledged "$(basename "$f")"; then
            acknowledged+=("$(basename "$f") ↔ $declared")
        else
            drifted+=("$(basename "$f") ↔ $declared")
        fi
    else
        ok_count=$((ok_count+1))
    fi
done

miss_n=${#missing[@]}; drift_n=${#drifted[@]}; skip_n=${#skipped[@]}
uninst_n=${#uninstalled[@]}; ack_n=${#acknowledged[@]}

if [ "$FORMAT" = json ]; then
    jarr() { local first=1; printf '['; for x in "$@"; do [ $first -eq 1 ] || printf ','; printf '%s' "$(printf '%s' "$x" | jq -R .)"; first=0; done; printf ']'; }
    # T-2682: uninstalled job lines fire unconditionally — they are the G-069
    # class, not a cosmetic difference, so --strict is irrelevant to them.
    fire=$({ [ "$miss_n" -gt 0 ] || [ "$uninst_n" -gt 0 ]; } && echo true \
        || { [ "$STRICT" -eq 1 ] && [ "$LENIENT" -eq 0 ] && [ "$drift_n" -gt 0 ] && echo true || echo false; })
    printf '{"ok":%s,"missing_count":%s,"uninstalled_jobs_count":%s,"drift_count":%s,"acknowledged_count":%s,"ok_count":%s,"skipped_count":%s,"strict":%s,"lenient":%s,"missing":%s,"uninstalled_jobs":%s,"drifted":%s,"acknowledged":%s}\n' \
        "$([ "$fire" = true ] && echo false || echo true)" \
        "$miss_n" "$uninst_n" "$drift_n" "$ack_n" "$ok_count" "$skip_n" \
        "$([ "$STRICT" -eq 1 ] && echo true || echo false)" \
        "$([ "$LENIENT" -eq 1 ] && echo true || echo false)" \
        "$(jarr "${missing[@]}")" "$(jarr "${uninstalled_detail[@]}")" "$(jarr "${drifted[@]}")" "$(jarr "${acknowledged[@]}")"
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
if [ "$ack_n" -gt 0 ]; then
    # Acknowledged drift is reported even when nothing fires: an allowlist that has
    # quietly grown should stay readable — that is the failure mode an allowlist invites.
    echo "check-cron-install-drift: $ack_n installed crontab(s) differ from git source but are ACKNOWLEDGED in $ALLOWLIST:"
    for a in "${acknowledged[@]}"; do echo "  ACKNOWLEDGED: $a"; done
fi
if [ "$drift_n" -gt 0 ]; then
    [ "$STRICT" -eq 1 ] && [ "$LENIENT" -eq 0 ] && fired=1
    if [ "$STRICT" -eq 1 ] && [ "$LENIENT" -eq 1 ]; then
        lvl="DRIFT (warning — --lenient)"
    elif [ "$STRICT" -eq 1 ]; then
        lvl="DRIFT (firing under --strict)"
    else
        lvl="DRIFT (warning)"
    fi
    echo "check-cron-install-drift: $drift_n installed crontab(s) differ from git source — $lvl:"
    for d in "${drifted[@]}"; do echo "  DRIFT: $d"; done
    echo "  Remediation: re-install from git source, or reconcile the /etc/cron.d edit back into .context/cron/."
fi
if [ "$skip_n" -gt 0 ] && [ "$QUIET" -ne 1 ]; then
    for s in "${skipped[@]}"; do echo "  skipped: $s"; done
fi
if [ "$fired" -eq 0 ]; then
    # T-2815/T-2821 wording rule, preserved through the T-2830 merge: only say
    # "healthy" when there is genuinely nothing outstanding. Summarising a tree
    # that carries known drift as "healthy" is the defect T-2815 was filed for —
    # exit 0 plus the word "healthy" is what both automation and a skimming human
    # read, and it made 21 diverged crontabs invisible for as long as they were
    # only ever a warning. Not firing is a policy choice; calling it healthy is a
    # false statement.
    if [ "$QUIET" -ne 1 ]; then
        if [ "$drift_n" -gt 0 ]; then
            echo "check-cron-install-drift: not firing, but NOT clean — $drift_n crontab(s) drift from git source ($ok_count matching, $ack_n acknowledged, $skip_n skipped)"
        else
            echo "check-cron-install-drift: healthy ($ok_count installed + matching, $ack_n acknowledged, $skip_n skipped)"
        fi
    fi
    exit 0
fi
exit 1
