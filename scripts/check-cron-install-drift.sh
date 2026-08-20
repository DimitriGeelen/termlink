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
# DRIFT (present but content differs from git) FIRES unless acknowledged;
# ACKNOWLEDGED (drift allowlisted with a reason) reported, not firing;
# OK (present + byte-identical).
#
# T-2697 (building on T-2690's honest-wording fix below). Drift was a non-firing
# warning, on the reasoning that a host-local edit should not paint the check
# permanently red. T-2690 corrected the summary WORD but deliberately kept the
# exit code. The remaining cost: exit 0 is what automation and a skimming human
# both read, and the state it was reporting was 21 of 24 installed crontabs
# diverged from source — carrying a real fix nobody had committed. A warning
# nobody must act on is indistinguishable from no warning once it scrolls past.
#
# The four source-level static checks (T-2527 alloc-sink, T-2531 drain-sink,
# T-2666 silent-exit, T-2672 busy-spin) had already settled the same tension the
# other way: fire by default, and acknowledge a confirmed-safe instance in an
# allowlist with a cited reason. This check now follows that convention.
# `--lenient` restores the pre-T-2697 behaviour; `--strict` is kept as an accepted
# alias of the new default so existing invocations keep working.
#
# Exit codes: 0 healthy · 1 firing (missing, or unacknowledged drift) · 2 tooling error
set -u

SRC_DIR="${CRON_DRIFT_SRC_DIR:-.context/cron}"
INSTALLED_DIR="${CRON_DRIFT_INSTALLED_DIR:-}"   # test hook: remap install root
ALLOWLIST="${CRON_DRIFT_ALLOWLIST:-.context/working/.cron-drift-allowlist}"
QUIET=0
FORMAT=human
LENIENT=0

usage() {
    sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: check-cron-install-drift.sh [OPTIONS]
  --lenient    Do NOT fire on DRIFT (pre-T-2697 behaviour); MISSING still fires
  --strict     Accepted alias of the default (kept for back-compat)
  --json       Emit a JSON envelope
  --quiet      Print only on firing (cron-friendly)
  -h, --help   This help

Allowlist: .context/working/.cron-drift-allowlist — one crontab basename per line
for a deliberate host-local variation, with the reason on a `#` comment. Same shape
as the alloc-sink / drain-sink / silent-exit / busy-spin allowlists. Acknowledged
entries are still reported and counted; they just do not fire. A MISSING crontab is
never acknowledgeable — a dark canary is not a variation.

Test hooks: CRON_DRIFT_SRC_DIR=<dir> (git crontab source, default .context/cron),
CRON_DRIFT_INSTALLED_DIR=<dir> (remaps each declared install path's dirname to this
dir, for host-independent fixtures), CRON_DRIFT_ALLOWLIST=<file>.

Exit: 0 healthy · 1 firing (missing / unacknowledged drift) · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --lenient) LENIENT=1; shift ;;
        # T-2697: --strict was the flag that made DRIFT fire; that is now the
        # default, so it is a no-op. Kept accepted rather than rejected — it is in
        # the docs and in muscle memory, and turning a documentation lag into an
        # unknown-arg failure helps nobody.
        --strict) shift ;;
        --json)   FORMAT=json; shift ;;
        --quiet)  QUIET=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "check-cron-install-drift: unknown arg: $1" >&2; exit 2 ;;
    esac
done

# Allowlisted basenames (deliberate host-local variation). Comments and blank
# lines ignored; an inline `#` reason after the name is stripped.
ACK_NAMES=""
if [ -r "$ALLOWLIST" ]; then
    ACK_NAMES="$(sed -e 's/#.*//' -e 's/[[:space:]]*$//' -e 's/^[[:space:]]*//' "$ALLOWLIST" 2>/dev/null | grep -v '^$' || true)"
fi

is_acknowledged() {
    [ -n "$ACK_NAMES" ] || return 1
    printf '%s\n' "$ACK_NAMES" | grep -qxF "$1"
}

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

missing=(); drifted=(); acknowledged=(); skipped=(); ok_count=0
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
        if is_acknowledged "$(basename "$f")"; then
            acknowledged+=("$(basename "$f") ↔ $declared")
        else
            drifted+=("$(basename "$f") ↔ $declared")
        fi
    else
        ok_count=$((ok_count+1))
    fi
done

miss_n=${#missing[@]}; drift_n=${#drifted[@]}; skip_n=${#skipped[@]}
ack_n=${#acknowledged[@]}

if [ "$FORMAT" = json ]; then
    jarr() { local first=1; printf '['; for x in "$@"; do [ $first -eq 1 ] || printf ','; printf '%s' "$(printf '%s' "$x" | jq -R .)"; first=0; done; printf ']'; }
    fire=$([ "$miss_n" -gt 0 ] && echo true || { [ "$LENIENT" -eq 0 ] && [ "$drift_n" -gt 0 ] && echo true || echo false; })
    printf '{"ok":%s,"missing_count":%s,"drift_count":%s,"acknowledged_count":%s,"ok_count":%s,"skipped_count":%s,"lenient":%s,"missing":%s,"drifted":%s,"acknowledged":%s}\n' \
        "$([ "$fire" = true ] && echo false || echo true)" \
        "$miss_n" "$drift_n" "$ack_n" "$ok_count" "$skip_n" \
        "$([ "$LENIENT" -eq 1 ] && echo true || echo false)" \
        "$(jarr "${missing[@]}")" "$(jarr "${drifted[@]}")" "$(jarr "${acknowledged[@]}")"
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
    [ "$LENIENT" -eq 0 ] && fired=1
    lvl=$([ "$LENIENT" -eq 1 ] && echo "DRIFT (warning — --lenient)" || echo "DRIFT (firing)")
    echo "check-cron-install-drift: $drift_n installed crontab(s) differ from git source — $lvl:"
    for d in "${drifted[@]}"; do echo "  DRIFT: $d"; done
    echo "  The installed copy is what actually runs; the git copy is what gets reviewed."
    echo "  Remediation: re-install from git source, or reconcile the /etc/cron.d edit back"
    echo "  into .context/cron/ — whichever is correct. If the difference is a deliberate"
    echo "  host-local variation, acknowledge it with a reason:"
    echo "    echo '<name>.crontab  # why this host differs' >> $ALLOWLIST"
fi
# Acknowledged drift is reported even when nothing is firing: an allowlist that has
# quietly grown should stay readable, which is the failure mode an allowlist invites.
if [ "$ack_n" -gt 0 ] && [ "$QUIET" -ne 1 ]; then
    echo "check-cron-install-drift: $ack_n acknowledged host-local variation(s) (not firing):"
    for a in "${acknowledged[@]}"; do echo "  ACKNOWLEDGED: $a"; done
fi
if [ "$skip_n" -gt 0 ] && [ "$QUIET" -ne 1 ]; then
    for s in "${skipped[@]}"; do echo "  skipped: $s"; done
fi
if [ "$fired" -eq 0 ]; then
    # Word the summary honestly (T-2690). T-2697 then made unacknowledged drift
    # fire outright, so this branch is now reached only under --lenient or with a
    # clean tree — but the wording rule still holds and still matters: "healthy" is
    # claimed only when nothing is missing and nothing drifts unacknowledged. It
    # was once printed while 21 installed crontabs had their canaries' stderr
    # rerouted to a `.log.stderr` sink that nothing read.
    if [ "$drift_n" -gt 0 ]; then
        [ "$QUIET" -eq 1 ] || echo "check-cron-install-drift: DRIFT ($ok_count installed + matching, $drift_n drifting, $ack_n acknowledged, $skip_n skipped) — NOT firing (--lenient); installed crontabs differ from the git source of truth, reconcile before trusting them"
    else
        [ "$QUIET" -eq 1 ] || echo "check-cron-install-drift: healthy ($ok_count installed + matching, $ack_n acknowledged, $skip_n skipped)"
    fi
    exit 0
fi
exit 1
