# T-3497 — the TERMLINK_* settings of a RUNNING hub, so a relaunch keeps them.
#
# Sourced by scripts (and embedded verbatim into fleet-deploy-binary.sh's remote
# runner). Defines one function:
#
#   hub_termlink_env <pid>  reads /proc/<pid>/environ and sets two arrays:
#     HUB_TL_ENV    every TERMLINK_<NAME>=<value> entry of the hub, verbatim
#     HUB_ENV_ARGS  arguments for env(1): "-u <var>" for each TERMLINK_* var of
#                   the CALLER (so the exec session's own settings never leak
#                   into the hub), then the HUB_TL_ENV entries
#   Returns 1 (reason on stderr) when the environment cannot be read: a relaunch
#   without it silently drops env-configured settings, so the caller must refuse.
#
# Entries are split on NUL and passed as separate argv words, never through a
# shell, so a value with spaces, quotes or newlines arrives byte-for-byte. Only
# TERMLINK_* is copied: the rest of the old environment (PATH, HOME, the
# watchdog's own variables) stays the relauncher's.
#
# Origin: ring20-dashboard (.121, 2026-10-20) lost TERMLINK_SWEEP_INTERVAL_SECS,
# set by its watchdog, on a swap-restart that carried only the runtime dir.
#
# Seam (fixtures only): HUB_PROC_ROOT (default /proc) — a fake /proc tree.
hub_termlink_env() {
    local pid="$1" env kv v
    local -a all=()
    HUB_TL_ENV=(); HUB_ENV_ARGS=()
    env="${HUB_PROC_ROOT:-/proc}/$pid/environ"
    if [ -z "$pid" ] || [ ! -f "$env" ] || ! mapfile -d '' all < "$env" 2>/dev/null; then
        echo "hub_termlink_env: cannot read $env" >&2; return 1
    fi
    for kv in "${all[@]}"; do
        [[ "${kv%%=*}" =~ ^TERMLINK_[A-Za-z0-9_]+$ ]] && [ "$kv" != "${kv%%=*}" ] && HUB_TL_ENV+=("$kv")
    done
    for v in $(compgen -e); do
        case "$v" in TERMLINK_*) HUB_ENV_ARGS+=(-u "$v") ;; esac
    done
    HUB_ENV_ARGS+=("${HUB_TL_ENV[@]}")
}
