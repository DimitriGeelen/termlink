# T-3437 — the runtime dir a RUNNING hub actually uses, resolved from the hub's
# own environment exactly as the binary resolves it, and confirmed by its pidfile.
#
# Sourced by scripts (and embedded verbatim into fleet-deploy-binary.sh's remote
# runner). Defines one function:
#
#   hub_runtime_dir <pid>   prints the confirmed dir and returns 0; on any doubt
#                           prints the reason on stderr and returns 1.
#
# The binary's order (crates/termlink-session/src/discovery.rs runtime_dir()):
#   1. $TERMLINK_RUNTIME_DIR   2. $XDG_RUNTIME_DIR/termlink
#   3. $TMPDIR/termlink-<uid>  4. /tmp/termlink-<uid>
# read from /proc/<pid>/environ (the hub's environment, not the caller's), with
# <uid> the hub process's own uid. The dir counts only when its hub.pid names
# <pid>: a relaunch into any other dir regenerates the hub's cert and secret
# (PL-021; .132 on 2026-10-13, when only step 1 was read and the relaunch fell
# back to ~/.termlink/runtime).
#
# Seams (fixtures only): HUB_PROC_ROOT (default /proc) — a fake /proc tree;
# HUB_DIR_ROOT (default empty) — prefix for reading <dir>/hub.pid, so a fixture
# never reads a real /tmp/termlink-<uid>. The printed dir is never prefixed.
hub_runtime_dir() {
    local pid="$1" proc="${HUB_PROC_ROOT:-/proc}" env uid dir v got
    [ -n "$pid" ] && [ -d "$proc/$pid" ] || { echo "hub_runtime_dir: no process $pid" >&2; return 1; }
    env="$proc/$pid/environ"
    [ -r "$env" ] || { echo "hub_runtime_dir: cannot read $env" >&2; return 1; }
    _hrd_env() { tr '\0' '\n' < "$env" 2>/dev/null | sed -n "s/^$1=//p" | head -1; }
    uid=$(stat -c %u "$proc/$pid" 2>/dev/null)
    if v=$(_hrd_env TERMLINK_RUNTIME_DIR) && [ -n "$v" ]; then dir="$v"
    elif v=$(_hrd_env XDG_RUNTIME_DIR) && [ -n "$v" ]; then dir="$v/termlink"
    elif v=$(_hrd_env TMPDIR) && [ -n "$v" ]; then dir="$v/termlink-$uid"
    elif [ -n "$uid" ]; then dir="/tmp/termlink-$uid"
    else echo "hub_runtime_dir: cannot tell the uid of process $pid" >&2; return 1
    fi
    got=$(tr -d '[:space:]' < "${HUB_DIR_ROOT:-}$dir/hub.pid" 2>/dev/null)
    if [ "$got" != "$pid" ]; then
        echo "hub_runtime_dir: resolved $dir, but its hub.pid names '${got:-nothing}', not $pid" >&2
        return 1
    fi
    printf '%s\n' "$dir"
}
