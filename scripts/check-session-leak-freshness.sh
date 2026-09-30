#!/usr/bin/env bash
# T-3296 (T-3291 S4, G-019) — Session-leak canary.
#
# Both halves of the T-3291 session leak ran for weeks with nothing firing:
#   * ZOMBIE SESSIONS — ~480 idle `termlink register --shell` processes (detached
#     tmux shells nobody touched since creation, 208 older than a week, ~4 GB).
#     Every one heartbeats every 30 s whether used or not, so every liveness
#     check saw them as LIVE. T-3294 makes a session end with its shell, but a
#     shell left idle at its prompt (the vendored dispatch never exits it) is
#     still a zombie — that part needs upstream (T-3291 S3a) and the reap (S3b).
#   * ORPHAN DATA SOCKETS — 10,144 `<id>.sock.data` with no registration; the
#     sweep deleted only `.sock` + `.json`. T-3293 fixed that and the hub now
#     reaps orphans within 10 min, so an orphan older than an hour here means
#     that reap is not running.
#
# Detection is scripts/lib/session-zombies.py — the SAME detector the runme reap
# uses, so what fires here is exactly what the reap would terminate.
#
# FIRES (exit 1) when zombies > --threshold (default 25) OR any orphan exists.
# Not firing: busy sessions (shell has a child), attached sessions, sessions
# younger than 24 h, non-shell registers (`--self` endpoints), and any register
# that is not a tmux pane (systemd-supervised always-on agents wait at an idle
# shell by design).
#
# Exit: 0 healthy · 1 firing · 2 tooling (fail-closed: unreadable process table
# or detector failure is never a clean result).
# Flags: --json, --quiet (print only when firing), --no-heartbeat, --threshold N.
# Seams (PL-213): SESSION_LEAK_PS_FILE, SESSION_LEAK_TMUX_FILE, SESSION_LEAK_DIRS
# (space-separated sessions dirs), SESSION_LEAK_HEARTBEAT_FILE.
# Operator action on firing: preview the reap with `/opt/termlink/runme.sh
# --dry-run`, then run it; for orphans check the hub is on a T-3293 build
# (`journalctl -u termlink-hub | grep "reaped orphaned"`).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
THRESHOLD=25
JSON=0
QUIET=0
HEARTBEAT=1
HEARTBEAT_FILE="${SESSION_LEAK_HEARTBEAT_FILE:-$ROOT/.context/working/.session-leak-canary.heartbeat}"

while [ $# -gt 0 ]; do
    case "$1" in
        --threshold) THRESHOLD="${2:-}"; shift 2 ;;
        --json) JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --help|-h) sed -n '2,32p' "$0"; exit 0 ;;
        *) echo "check-session-leak-freshness: unknown argument: $1" >&2; exit 2 ;;
    esac
done
case "$THRESHOLD" in ''|*[!0-9]*) echo "check-session-leak-freshness: --threshold needs a number" >&2; exit 2 ;; esac

# T-2691 convention: heartbeat on EXIT, so freshness proves the run finished.
_canary_hb() {
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null && date -u +%Y-%m-%dT%H:%M:%SZ > "$HEARTBEAT_FILE" 2>/dev/null || true
}
if [ "$HEARTBEAT" -eq 1 ]; then trap _canary_hb EXIT; fi

command -v python3 >/dev/null 2>&1 || { echo "check-session-leak-freshness: python3 not found" >&2; exit 2; }

if [ -n "${SESSION_LEAK_DIRS:-}" ]; then
    DIRS="$SESSION_LEAK_DIRS"
else
    DIRS="${TERMLINK_RUNTIME_DIR:-/var/lib/termlink}/sessions /tmp/termlink-$(id -u)/sessions"
fi
args=()
for d in $DIRS; do args+=(--dir "$d"); done
[ -n "${SESSION_LEAK_PS_FILE:-}" ] && args+=(--ps "$SESSION_LEAK_PS_FILE")
[ -n "${SESSION_LEAK_TMUX_FILE:-}" ] && args+=(--tmux "$SESSION_LEAK_TMUX_FILE")

out="$(python3 "$ROOT/scripts/lib/session-zombies.py" "${args[@]}")" || {
    echo "check-session-leak-freshness: detector failed (process table unreadable?)" >&2; exit 2; }

python3 - "$out" "$THRESHOLD" "$JSON" "$QUIET" <<'PYEOF'
import json, sys
d = json.loads(sys.argv[1]); threshold = int(sys.argv[2]); as_json = sys.argv[3] == "1"; quiet = sys.argv[4] == "1"
z = d["zombie"]
orph = {k: v for k, v in d["orphans"].items() if v}
firing = len(z) > threshold or bool(orph)
if as_json:
    d.update({"ok": not firing, "threshold": threshold, "zombie_count": len(z), "orphan_total": sum(orph.values())})
    print(json.dumps(d))
    sys.exit(1 if firing else 0)
if quiet and not firing:
    sys.exit(0)
ts = __import__("datetime").datetime.now(__import__("datetime").timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
if firing:
    print(f"=== {ts} session-leak FIRING ===")
    if len(z) > threshold:
        oldest = sorted(z, key=lambda e: -e["age_s"])[:5]
        print(f"  zombie sessions: {len(z)} (threshold {threshold}) — idle >{d['min_age_s']//3600}h, shell childless, not attached")
        for e in oldest:
            print(f"    pid {e['pid']:>7}  {e['name']:<30} {e['age_s']//86400}d  tmux={e['tmux_session'] or '-'}")
        print("  action: preview `/opt/termlink/runme.sh --dry-run` (zombie reap), then run it")
    for k, v in orph.items():
        print(f"  orphan data sockets: {v} in {k} (no .json, >{d['orphan_age_s']//60} min) — hub reap not running? (T-3293)")
else:
    print(f"session-leak: healthy — {len(z)} zombie(s) (threshold {threshold}), 0 orphan data sockets")
print(f"  scanned: busy={d['busy']} attached={d['attached']} young={d['young']} launcher_alive={d['launcher_alive']} non_shell={d['non_shell']} unmanaged(not tmux, never flagged)={len(d['unmanaged'])} dirs={list(d['orphans'])}")
missing = [k for k, v in d["orphans"].items() if v is None]
if missing:
    print(f"  note: unreadable/absent sessions dir(s), not scanned: {missing}")
sys.exit(1 if firing else 0)
PYEOF
