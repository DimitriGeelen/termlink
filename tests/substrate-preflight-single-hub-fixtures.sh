#!/usr/bin/env bash
# guard-layer: source
#
# T-3340 — fixtures for substrate-preflight Check 7 (single-hub). Hermetic: candidate
# dirs and "live" pids come from seams; no real hub is read or signalled.
# T-3657 — undeclared second hub is FAIL; declared (<dir>/second-hub.declared) stays
# WARN; the single-binary row FAILs on two undeclared versions (binaries from a seam).
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
BINS_DEFAULT="/opt/a/termlink=1.0.1"
row() { # dirs alive [check-name] -> the JSON row (status|message)
    TERMLINK_PREFLIGHT_TEST_HUB_DIRS="$1" TERMLINK_PREFLIGHT_TEST_ALIVE_PIDS="$2" \
    TERMLINK_PREFLIGHT_TEST_BINARIES="${BINS:-$BINS_DEFAULT}" \
    TERMLINK_DECLARED_BINARIES_FILE="${DECL:-$TMP/no-decl}" \
        bash "$SCRIPT" --json --no-heartbeat 2>/dev/null | python3 -c '
import json,sys
d=json.load(sys.stdin)
r=[c for c in d["checks"] if c.get("name")==sys.argv[1]]
print((r[0]["status"]+"|"+r[0]["message"]) if r else "MISSING|")' "${3:-single-hub}"
}
rc_of() { # dirs alive -> preflight exit code
    TERMLINK_PREFLIGHT_TEST_HUB_DIRS="$1" TERMLINK_PREFLIGHT_TEST_ALIVE_PIDS="$2" \
    TERMLINK_PREFLIGHT_TEST_BINARIES="${BINS:-$BINS_DEFAULT}" \
    TERMLINK_DECLARED_BINARIES_FILE="${DECL:-$TMP/no-decl}" \
        bash "$SCRIPT" --json --no-heartbeat >/dev/null 2>&1; echo $?
}

A="$TMP/var-lib"; B="$TMP/tmp-default"; C="$TMP/stale"
mkhub "$A" 101 3; mkhub "$B" 202 2; mkhub "$C" 303 1

out=$(row "$A:$B" "101 202")
case "$out" in fail\|*"2 live termlink hubs"*"2 undeclared"*"pid 101 in $A (3 sessions)"*"pid 202 in $B (2 sessions)"*) ok "two live undeclared hubs: FAIL naming both dirs, pids and session counts" ;; *) bad "two live: $out" ;; esac
[ "$(rc_of "$A:$B" "101 202")" = 2 ] && ok "undeclared second hub: preflight exits 2 (FAIL)" || bad "undeclared rc"

: > "$B/second-hub.declared"
out=$(row "$A:$B" "101 202")
case "$out" in warn\|*"2 live termlink hubs"*"(2 sessions, declared)"*) ok "declared second hub: WARN, not FAIL, and marked declared" ;; *) bad "declared: $out" ;; esac
out=$(TERMLINK_PREFLIGHT_TEST_HUB_DIRS="$A:$B" TERMLINK_PREFLIGHT_TEST_ALIVE_PIDS="101 202" TERMLINK_PREFLIGHT_TEST_BINARIES="$BINS_DEFAULT" \
    bash "$SCRIPT" --json --no-heartbeat 2>/dev/null | python3 -c '
import json,sys
r=[c for c in json.load(sys.stdin)["checks"] if c.get("name")=="single-hub"]
print(r[0]["severity"] if r else "MISSING")')
[ "$out" = medium ] && ok "declared second hub: medium severity, so it cannot raise exit 2" || bad "declared severity: $out"
rm -f "$B/second-hub.declared"

out=$(row "$A:$C" "101")
case "$out" in pass\|*"one live termlink hub"*"pid 101"*) ok "one live + one stale pidfile: PASS, the stale one ignored" ;; *) bad "one live: $out" ;; esac

out=$(row "$A:$A:$B" "101")
case "$out" in pass\|*"one live"*) ok "a dir listed twice is counted once" ;; *) bad "dedupe: $out" ;; esac

out=$(row "$C" "")
case "$out" in pass\|*"no live termlink hub"*) ok "no live hub: PASS (Check 6 owns a down hub)" ;; *) bad "none: $out" ;; esac

out=$(row "$TMP/absent:$B" "202")
case "$out" in pass\|*"pid 202"*) ok "missing dir is skipped, not an error" ;; *) bad "absent dir: $out" ;; esac

out=$(row "$A" "101")
case "$out" in pass\|*"one live termlink hub"*) ok "single hub: PASS" ;; *) bad "single: $out" ;; esac

# ---- single-binary row (T-3657) ----
out=$(BINS="/opt/a/termlink=1.0.1 /opt/b/termlink=1.0.1" row "$A" "101" single-binary)
case "$out" in pass\|*"one termlink version"*) ok "two binaries, same version: PASS" ;; *) bad "same version: $out" ;; esac

out=$(BINS="/opt/a/termlink=1.0.1 /opt/b/termlink=0.9.7" row "$A" "101" single-binary)
case "$out" in fail\|*"2 different termlink versions"*"/opt/b/termlink 0.9.7"*) ok "two versions undeclared: FAIL naming both" ;; *) bad "two versions: $out" ;; esac

printf '/opt/b/termlink\n' > "$TMP/decl"
out=$(DECL="$TMP/decl" BINS="/opt/a/termlink=1.0.1 /opt/b/termlink=0.9.7" row "$A" "101" single-binary)
case "$out" in pass\|*"/opt/b/termlink 0.9.7 (declared)"*) ok "second version declared: PASS, marked declared" ;; *) bad "declared bin: $out" ;; esac

echo; echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
