#!/usr/bin/env bash
# guard-layer: source
#
# T-3334 — fixtures for scripts/check-agent-send-suites-freshness.sh. Hermetic: stub
# suites in a temp dir and a canned hub-precheck rc; no live hub is touched.
set -uo pipefail
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/check-agent-send-suites-freshness.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
mk() { printf '#!/usr/bin/env bash\n%s\n' "$2" > "$TMP/$1"; }
mk green.sh 'echo "PASS A"; exit 0'
mk red.sh   'echo "PASS A"; echo "FAIL E: expected rc=4 (got 3)"; exit 1'
run() { AGENT_SEND_SUITES_DIR="$TMP" AGENT_SEND_CANARY_TEST_HUB_RC="${HUB:-0}" \
        AGENT_SEND_CANARY_HEARTBEAT_FILE="$TMP/hb" AGENT_SEND_SUITES="$SUITES" bash "${CANARY:-$SCRIPT}" "$@"; }

SUITES="green.sh"; out="$(run --quiet)"; rc=$?
[ "$rc" = 0 ] && [ -z "$out" ] && ok "all green: exit 0, quiet prints nothing" || bad "green (rc=$rc out=$out)"
[ -f "$TMP/hb" ] && ok "heartbeat written on exit" || bad "heartbeat"

SUITES="green.sh red.sh"; out="$(run --quiet)"; rc=$?
[ "$rc" = 1 ] && printf '%s' "$out" | grep -q 'FAILING red.sh rc=1: FAIL E: expected rc=4' \
    && ok "a red suite fires (exit 1) naming the suite and its FAIL line" || bad "red (rc=$rc out=$out)"
j="$(run --json)"; [ "$(printf '%s' "$j" | jq -r '.firing')" = "true" ] && ok "--json reports firing" || bad "json: $j"

SUITES="green.sh"; rm -f "$TMP/hb"; HUB=1 run --quiet >/dev/null 2>&1; rc=$?
[ "$rc" = 2 ] && ok "hub unreachable: exit 2 (tooling, not firing)" || bad "hub down rc=$rc"
[ -f "$TMP/hb" ] && ok "heartbeat still written on a tooling exit (the run completed)" || bad "heartbeat on exit 2"

SUITES="green.sh nope.sh"; run --quiet >/dev/null 2>&1; rc=$?
[ "$rc" = 2 ] && ok "missing suite: exit 2, never a clean pass" || bad "missing rc=$rc"

# Mutant: a canary that ignores suite exit codes must be caught by the red case.
sed 's/\[ "\$rc" -ne 0 \] && failed=1/true/' "$SCRIPT" > "$TMP/mut.sh"
if cmp -s "$TMP/mut.sh" "$SCRIPT"; then bad "mutant did not apply"; else
    SUITES="red.sh"; CANARY="$TMP/mut.sh" run --quiet >/dev/null 2>&1; rc=$?
    [ "$rc" = 0 ] && ok "mutant (ignore suite rc) would pass a red suite — the red case above pins it" || bad "mutant rc=$rc"
fi

echo; echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
