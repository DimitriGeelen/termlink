#!/usr/bin/env bash
# T-2826 — fixtures for canary-status.sh's FIRING classification.
#
# The load-bearing one is SAME-SECOND. That is the case the bug was made of:
# `-gt` reported a firing canary as HEALTHY whenever its log and heartbeat
# landed in the same second, which is the common case because every canary
# touches its heartbeat at the top of the run (T-1723) and the crontab appends
# the log when it finishes.
#
# The other cases are here to stop the fix over-correcting. A canary whose log
# predates its heartbeat is genuinely healthy — it found nothing this run, so it
# wrote nothing, and the old entries are history. If that leg ever fails, the
# fix has turned every canary that ever fired into a permanent red light.
#
# Run: bash tests/canary-status-firing-fixtures.sh

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/canary-status.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL %s\n     %s\n' "$1" "${2:-}"; }

[ -r "$SCRIPT" ] || { echo "cannot read $SCRIPT" >&2; exit 2; }

echo "== canary-status firing fixtures =="

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
W="$TMP/.context/working"
mkdir -p "$W"

# canary-status discovers by globbing the working dir; it resolves that relative
# to the repo root, so run it from the fixture tree.
run() { (cd "$TMP" && bash "$SCRIPT" "$@" 2>&1); }

mk() {  # mk <name> <log-content> <log-epoch> <heartbeat-epoch>
    local n="$1" body="$2" lt="$3" ht="$4"
    printf '%s\n' "$body" > "$W/.$n-canary.log"
    : > "$W/.$n-canary.heartbeat"
    touch -d "@$ht" "$W/.$n-canary.heartbeat"
    touch -d "@$lt" "$W/.$n-canary.log"
}

NOW=$(date +%s)

# --- 1. SAME SECOND -> FIRING (the whole point) -----------------------------
rm -f "$W"/.*-canary.* 2>/dev/null
mk samesec "check-x: FIRING — something is wrong" "$NOW" "$NOW"
out=$(run); rc=$?
if printf '%s' "$out" | grep -q 'FIRING'; then
    ok "log mtime EQUAL to heartbeat is classified FIRING"
else
    bad "same-second must be FIRING" "$(printf '%s' "$out" | head -6)"
fi
if [ "$rc" -eq 1 ]; then
    ok "a firing canary makes the exit code 1"
else
    bad "firing canary should exit 1" "rc=$rc"
fi
if printf '%s' "$out" | grep -qE '1 firing|firing: *1'; then
    ok "the summary counts it"
else
    bad "summary should count 1 firing" "$(printf '%s' "$out" | head -3)"
fi

# --- 2. log NEWER than heartbeat -> FIRING (unchanged behaviour) -------------
rm -f "$W"/.*-canary.* 2>/dev/null
mk newer "check-y: FIRING — problem" "$NOW" "$((NOW - 30))"
out=$(run); rc=$?
if printf '%s' "$out" | grep -q 'FIRING' && [ "$rc" -eq 1 ]; then
    ok "log newer than heartbeat is still FIRING"
else
    bad "log-newer must remain FIRING" "rc=$rc"
fi

# --- 3. log OLDER than heartbeat -> HEALTHY (the over-correction guard) ------
# A canary that fired last week and has been quiet since must NOT be red now.
rm -f "$W"/.*-canary.* 2>/dev/null
mk older "check-z: FIRING — this was last week" "$((NOW - 604800))" "$NOW"
out=$(run); rc=$?
if printf '%s' "$out" | grep -q 'HEALTHY' && [ "$rc" -eq 0 ]; then
    ok "historical entries (log older than heartbeat) stay HEALTHY"
else
    bad "log-older must stay HEALTHY — the fix must not make old firings permanent" \
        "rc=$rc; $(printf '%s' "$out" | head -4)"
fi

# --- 4. empty log -> HEALTHY ------------------------------------------------
rm -f "$W"/.*-canary.* 2>/dev/null
: > "$W/.empty-canary.log"
: > "$W/.empty-canary.heartbeat"
touch -d "@$NOW" "$W/.empty-canary.log" "$W/.empty-canary.heartbeat"
out=$(run); rc=$?
if [ "$rc" -eq 0 ] && ! printf '%s' "$out" | grep -q 'FIRING'; then
    ok "empty log is HEALTHY even at the same second (empty-log = healthy convention)"
else
    bad "empty log must stay HEALTHY" "rc=$rc; $(printf '%s' "$out" | head -4)"
fi

echo
echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
