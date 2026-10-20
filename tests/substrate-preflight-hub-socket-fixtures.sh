#!/usr/bin/env bash
# guard-layer: source
#
# T-3495 — fixtures for substrate-preflight Check 8 (hub-socket). Hermetic: the
# `hub status --json` output comes from a seam; no real hub is contacted.
set -uo pipefail
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/substrate-preflight.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
command -v python3 >/dev/null || { echo "TOOLING: python3 missing" >&2; exit 2; }
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

row() { # canned hub-status json -> the hub-socket JSON row (status|message)
    printf '%s' "$1" > "$TMP/status.json"
    TERMLINK_PREFLIGHT_TEST_HUB_STATUS_JSON="$TMP/status.json" \
    TERMLINK_PREFLIGHT_TEST_HUB_DIRS="$TMP/none" \
        bash "$SCRIPT" --json 2>/dev/null | python3 -c '
import json,sys
d=json.load(sys.stdin)
r=[c for c in d["checks"] if c.get("name")=="hub-socket"]
print((r[0]["status"]+"|"+r[0]["message"]) if r else "MISSING|")'
}

out=$(row '{"ok":false,"status":"running","pid":7,"socket":"/x/hub.sock","socket_ok":false,"socket_error":"/x/hub.sock: Connection refused (os error 111)"}')
case "$out" in fail\|*"does not answer"*"Connection refused"*) ok "socket_ok:false: FAIL naming the probe error" ;; *) bad "dead socket: $out" ;; esac

out=$(row '{"ok":true,"status":"running","pid":7,"socket":"/x/hub.sock","socket_ok":true}')
case "$out" in pass\|*"answers on its unix socket"*) ok "socket_ok:true: PASS" ;; *) bad "live socket: $out" ;; esac

out=$(row '{"ok":true,"status":"not_running"}')
case "$out" in MISSING\|) ok "hub not running: no row (Check 6 owns a down hub)" ;; *) bad "not running: $out" ;; esac

out=$(row '{"ok":true,"status":"running","pid":7}')
case "$out" in MISSING\|) ok "pre-T-3495 binary (no socket_ok): no row, never a false pass" ;; *) bad "old binary: $out" ;; esac

echo; echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
