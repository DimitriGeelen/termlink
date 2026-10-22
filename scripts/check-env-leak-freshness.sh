#!/usr/bin/env bash
# T-3551 (SEV-1 T-3549, G-019) — environment-leak canary.
#
# On 2026-10-22 the host's default tmux server was restarted by an ordinary ssh logout, and
# the next spawn (from another project) seeded it with that project's TASKS_DIR/CONTEXT_DIR.
# Every session started on it afterwards ran with them: 30 of 54 registered sessions were found
# carrying them, by hand, by joining `termlink list --json` with /proc/<pid>/environ. Nothing
# fired. T-3550 builds every spawned environment by construction and `termlink register`
# now refuses a foreign project at start; this canary finds what got in anyway or predates both.
#
# Detection is scripts/lib/env-leak-scan.py. A project variable is FOREIGN when its value lies in
# a different project than the process's own working directory; unknown (temp dirs, processes
# outside any project) is never a finding.
#
# FIRES (exit 1) when any registered session's root process carries a foreign variable, OR the
# default tmux server's global environment holds any project variable at all.
# Exit: 0 healthy · 1 firing · 2 tooling (fail-closed: unreadable session list or process table
# is never a clean result).
# Flags: --json, --quiet (print only when firing), --no-heartbeat.
# Seams (PL-213): ENV_LEAK_LIST_JSON, ENV_LEAK_PROC_DIR, ENV_LEAK_TMUX_ENV_FILE,
# ENV_LEAK_HEARTBEAT_FILE.
# Operator action on firing: the named processes act on another project's tasks. Relaunch each
# from a clean shell (relaunching another project's agents is that project's or the operator's
# call); for a polluted server global, `tmux set-environment -gu <VAR>` stops NEW sessions
# inheriting it, but running ones keep it until relaunched.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JSON=0
QUIET=0
HEARTBEAT=1
HEARTBEAT_FILE="${ENV_LEAK_HEARTBEAT_FILE:-$ROOT/.context/working/.env-leak-canary.heartbeat}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --help|-h) sed -n '2,27p' "$0"; exit 0 ;;
        *) echo "check-env-leak-freshness: unknown argument: $1" >&2; exit 2 ;;
    esac
done

# T-2843 convention: heartbeat on EXIT, so freshness proves the run finished.
_canary_hb() {
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null && date -u +%Y-%m-%dT%H:%M:%SZ > "$HEARTBEAT_FILE" 2>/dev/null || true
}
if [ "$HEARTBEAT" -eq 1 ]; then trap _canary_hb EXIT; fi

command -v python3 >/dev/null 2>&1 || { echo "check-env-leak-freshness: python3 not found" >&2; exit 2; }

args=()
[ -n "${ENV_LEAK_LIST_JSON:-}" ] && args+=(--list-json "$ENV_LEAK_LIST_JSON")
[ -n "${ENV_LEAK_PROC_DIR:-}" ] && args+=(--proc "$ENV_LEAK_PROC_DIR")
[ -n "${ENV_LEAK_TMUX_ENV_FILE+set}" ] && args+=(--tmux-env "$ENV_LEAK_TMUX_ENV_FILE")

out="$(python3 "$ROOT/scripts/lib/env-leak-scan.py" ${args[@]+"${args[@]}"})" || {
    echo "check-env-leak-freshness: detector could not scan (session list or process table unreadable)" >&2; exit 2; }

python3 - "$out" "$JSON" "$QUIET" <<'PYEOF'
import json, sys, datetime
d = json.loads(sys.argv[1]); as_json = sys.argv[2] == "1"; quiet = sys.argv[3] == "1"
firing = bool(d["foreign"] or d["server_global"])
if as_json:
    d["ok"] = not firing
    print(json.dumps(d))
    sys.exit(1 if firing else 0)
if quiet and not firing:
    sys.exit(0)
ts = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
if firing:
    print(f"=== {ts} env-leak FIRING ===")
    by_session = {}
    for f in d["foreign"]:
        by_session.setdefault((f["session"], f["session_pid"]), []).append(f)
    for (name, spid), fs in by_session.items():
        own = fs[0]["own_project"]
        print(f"  session {name} (pid {spid}) works in {own} but carries:")
        for f in fs:
            print(f"    pid {f['pid']}: {f['name']}={f['value']}  (belongs to {f['value_project']})")
    for v in d["server_global"]:
        print(f"  tmux default server GLOBAL env: {v['name']}={v['value']} (every new session inherits it)")
    print("  action: relaunch the named processes from a clean shell; `tmux set-environment -gu <VAR>` for server globals")
else:
    print("env-leak: healthy — no foreign project variables, tmux server global env holds no project")
print(f"  scanned: sessions={d['sessions']} root processes={d['processes_scanned']} unreadable={d['processes_unreadable']}")
sys.exit(1 if firing else 0)
PYEOF
