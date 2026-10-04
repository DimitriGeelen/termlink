#!/usr/bin/env bash
# guard-layer: source
#
# T-3340 — fixtures for substrate-preflight Check 7 (single-hub). Hermetic: candidate
# dirs and "live" pids come from seams; no real hub is read or signalled.
set -uo pipefail
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/substrate-preflight.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
command -v python3 >/dev/null || { echo "TOOLING: python3 missing" >&2; exit 2; }
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

mkhub() { # dir pid nsessions
    mkdir -p "$1/sessions"; echo "$2" > "$1/hub.pid"
    for i in $(seq 1 "$3"); do : > "$1/sessions/s$i.json"; done
}
row() { # dirs alive -> the single-hub JSON row (status|message)
    TERMLINK_PREFLIGHT_TEST_HUB_DIRS="$1" TERMLINK_PREFLIGHT_TEST_ALIVE_PIDS="$2" \
        bash "$SCRIPT" --json 2>/dev/null | python3 -c '
import json,sys
d=json.load(sys.stdin)
r=[c for c in d["checks"] if c.get("name")=="single-hub"]
print((r[0]["status"]+"|"+r[0]["message"]) if r else "MISSING|")'
}

A="$TMP/var-lib"; B="$TMP/tmp-default"; C="$TMP/stale"
mkhub "$A" 101 3; mkhub "$B" 202 2; mkhub "$C" 303 1

out=$(row "$A:$B" "101 202")
case "$out" in warn\|*"2 live termlink hubs"*"pid 101 in $A (3 sessions)"*"pid 202 in $B (2 sessions)"*) ok "two live hubs: WARN naming both dirs, pids and session counts" ;; *) bad "two live: $out" ;; esac

out=$(row "$A:$C" "101")
case "$out" in pass\|*"one live termlink hub"*"pid 101"*) ok "one live + one stale pidfile: PASS, the stale one ignored" ;; *) bad "one live: $out" ;; esac

out=$(row "$A:$A:$B" "101")
case "$out" in pass\|*"one live"*) ok "a dir listed twice is counted once" ;; *) bad "dedupe: $out" ;; esac

out=$(row "$C" "")
case "$out" in pass\|*"no live termlink hub"*) ok "no live hub: PASS (Check 6 owns a down hub)" ;; *) bad "none: $out" ;; esac

out=$(row "$TMP/absent:$B" "202")
case "$out" in pass\|*"pid 202"*) ok "missing dir is skipped, not an error" ;; *) bad "absent dir: $out" ;; esac

echo; echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
