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
# Classes: MISSING (declared path absent) FIRES (exit 1 — the G-069 class);
# DRIFT (present but content differs from git) is a non-firing WARNING by default,
# fires only under --strict; OK (present + byte-identical).
#
# Exit codes: 0 healthy · 1 firing (missing, or drift under --strict) · 2 tooling error
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
  --strict     Also fire (exit 1) on DRIFT, not just MISSING
  --json       Emit a JSON envelope
  --quiet      Print only on firing (cron-friendly)
  -h, --help   This help

Test hooks: CRON_DRIFT_SRC_DIR=<dir> (git crontab source, default .context/cron),
CRON_DRIFT_INSTALLED_DIR=<dir> (remaps each declared install path's dirname to this
dir, for host-independent fixtures).

Exit: 0 healthy · 1 firing (missing / drift-under-strict) · 2 tooling error
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

missing=(); drifted=(); skipped=(); ok_count=0
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
        drifted+=("$(basename "$f") ↔ $declared")
    else
        ok_count=$((ok_count+1))
    fi
done

miss_n=${#missing[@]}; drift_n=${#drifted[@]}; skip_n=${#skipped[@]}

if [ "$FORMAT" = json ]; then
    jarr() { local first=1; printf '['; for x in "$@"; do [ $first -eq 1 ] || printf ','; printf '%s' "$(printf '%s' "$x" | jq -R .)"; first=0; done; printf ']'; }
    fire=$([ "$miss_n" -gt 0 ] && echo true || { [ "$STRICT" -eq 1 ] && [ "$drift_n" -gt 0 ] && echo true || echo false; })
    printf '{"ok":%s,"missing_count":%s,"drift_count":%s,"ok_count":%s,"skipped_count":%s,"strict":%s,"missing":%s,"drifted":%s}\n' \
        "$([ "$fire" = true ] && echo false || echo true)" \
        "$miss_n" "$drift_n" "$ok_count" "$skip_n" \
        "$([ "$STRICT" -eq 1 ] && echo true || echo false)" \
        "$(jarr "${missing[@]}")" "$(jarr "${drifted[@]}")"
    [ "$fire" = true ] && exit 1 || exit 0
fi

fired=0
if [ "$miss_n" -gt 0 ]; then
    fired=1
    echo "check-cron-install-drift: $miss_n git-tracked crontab(s) NOT installed — SHIPPED BUT DARK (G-069, T-2561):"
    for m in "${missing[@]}"; do echo "  MISSING: $m"; done
    echo "  Remediation: install each with  sudo cp .context/cron/<name>.crontab <declared-path>  (root)."
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
