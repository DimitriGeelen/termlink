#!/usr/bin/env bash
# T-3425 — Root-directory mode canary (boot + hourly).
#
# On 2026-10-12 `/` lost its execute bits (0755 -> 0644, an agent's
# `tmux -S /proc/<pid>/cwd`). Root kept working, so nothing looked wrong until the
# next boot: every service that drops privileges failed, D-Bus first, and the
# host never reached the login screen. Nothing fired for the ~10 minutes between the
# change and the reboot (docs/reports/T-3425-root-chmod-incident.md).
#
# Checks that `/` is a directory with mode 755 owned by root. It ALERTS and never
# repairs: who changed `/` matters, and a silent self-heal would hide that.
#   exit 0 healthy · 1 FIRING (prints the finding; also logger -p auth.crit) · 2 tooling
# Flags: --quiet (print only when firing), --no-heartbeat.
# Seams: ROOT_MODE_PATH (default /), ROOT_MODE_HEARTBEAT_FILE.
set -u
QUIET=0; HEARTBEAT=1
for a in "$@"; do
    case "$a" in
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        *) echo "usage: $0 [--quiet] [--no-heartbeat]" >&2; exit 2 ;;
    esac
done
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HB="${ROOT_MODE_HEARTBEAT_FILE:-$ROOT/.context/working/.root-mode-canary.heartbeat}"
[ "$HEARTBEAT" = 1 ] && trap 'touch "$HB" 2>/dev/null || true' EXIT

P="${ROOT_MODE_PATH:-/}"
if ! info=$(stat -c '%a %U %F' "$P" 2>/dev/null); then
    echo "root-mode canary: cannot stat $P" >&2; exit 2
fi
read -r mode owner kind_rest <<<"$info"
if [ "$mode" = 755 ] && [ "$owner" = root ] && [ "$kind_rest" = directory ]; then
    [ "$QUIET" = 1 ] || echo "root-mode canary: $P is 755 root directory (healthy)"
    exit 0
fi
msg="root-mode canary FIRING: $P is mode $mode owner $owner ($kind_rest), expected 755 root directory. Non-root services cannot start; fix: chmod 755 $P (see docs/reports/T-3425-root-chmod-incident.md)"
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $msg"
command -v logger >/dev/null && logger -p auth.crit -t root-mode-canary "$msg" 2>/dev/null
exit 1
