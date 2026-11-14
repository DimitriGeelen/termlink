# shellcheck shell=bash
# T-3792: find runuser whatever PATH says. Cron's PATH here is
# /root/.cargo/bin:/usr/local/bin:/usr/bin:/bin, and runuser lives in /usr/sbin,
# so a bare `runuser` was "command not found" and the callers reported it as "no answer".
# Prints the absolute path; returns 1 when there is none.
# Seam: RUNUSER_SEARCH (space-separated candidates after PATH) for fixtures.
resolve_runuser() {
    local c
    if c="$(command -v runuser 2>/dev/null)" && [ -n "$c" ]; then echo "$c"; return 0; fi
    for c in ${RUNUSER_SEARCH:-/usr/sbin/runuser /sbin/runuser}; do
        [ -x "$c" ] && { echo "$c"; return 0; }
    done
    return 1
}
