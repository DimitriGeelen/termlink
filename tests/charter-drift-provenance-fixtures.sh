#!/usr/bin/env bash
# T-2829 — fixtures for the charter-drift canary's PROVENANCE reporting.
#
# The defect: the canary reads whatever binary `$TERMLINK` resolves to, and said
# nothing about which one. Under cron's PATH that was a Jul-31 /usr/local/bin
# build (0.11.693) predating the P4 deprecations, so it appended a 40-tool FIRING
# entry every day for twenty days while the surface was already clean. Nothing in
# the output distinguished "the surface drifted" from "you asked the wrong binary".
#
# A version floor was rejected as the fix and these fixtures do NOT test one.
# Deprecation is monotonic here (6 deprecated at 0.11.693, 46 at 0.11.1196, none
# un-deprecated), so the 40 tools separating the builds ARE the firing set — the
# staleness signal and the drift verdict are the same names, and no threshold over
# them can tell the two apart. What the check can always do honestly is name what
# it read. That is what is pinned here.
#
# Run: bash tests/charter-drift-provenance-fixtures.sh

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/check-charter-drift-freshness.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL %s\n     %s\n' "$1" "${2:-}"; }

[ -r "$SCRIPT" ] || { echo "cannot read $SCRIPT" >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { echo "jq required" >&2; exit 2; }

echo "== charter-drift provenance fixtures =="

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# A catalog with one off-charter tool LIVE (fires) and one deprecated (must not).
cat > "$TMP/dirty.json" <<'JSON'
{"channel":[
  {"name":"termlink_channel_react","deprecated":false},
  {"name":"termlink_channel_post","deprecated":false},
  {"name":"termlink_channel_star","deprecated":true}
]}
JSON

# A clean catalog: the off-charter tool is deprecated, so nothing fires.
cat > "$TMP/clean.json" <<'JSON'
{"channel":[
  {"name":"termlink_channel_react","deprecated":true},
  {"name":"termlink_channel_post","deprecated":false}
]}
JSON

run_hook() { TERMLINK_CHARTER_DRIFT_TEST_JSON="$1" bash "$SCRIPT" --no-heartbeat "${@:2}" 2>&1; }

# --- 1. firing output names the binary it probed ----------------------------
out="$(run_hook "$TMP/dirty.json")"; rc=$?
if [ "$rc" -eq 1 ]; then
    ok "a live off-charter tool still fires (exit 1)"
else
    bad "dirty catalog must fire" "rc=$rc"
fi
if printf '%s' "$out" | grep -q 'probed:'; then
    ok "firing output carries a 'probed:' provenance line"
else
    bad "firing output must name what it read" "$(printf '%s' "$out" | head -4)"
fi

# --- 2. healthy output names it too -----------------------------------------
# The 20-day miss was a FIRING run, but a clean run from the wrong binary is the
# symmetric hazard: it would report a drifted surface as healthy.
out="$(run_hook "$TMP/clean.json")"; rc=$?
if [ "$rc" -eq 0 ]; then
    ok "a deprecated off-charter tool does not fire (exit 0)"
else
    bad "clean catalog must not fire" "rc=$rc"
fi
if printf '%s' "$out" | grep -q 'read from'; then
    ok "healthy output also carries provenance"
else
    bad "healthy output must name what it read" "$(printf '%s' "$out" | head -4)"
fi

# --- 3. JSON carries both fields, both verdicts ------------------------------
out="$(run_hook "$TMP/dirty.json" --json)"
if printf '%s' "$out" | jq -e 'has("probe_binary") and has("probe_version")' >/dev/null 2>&1; then
    ok "firing JSON carries probe_binary + probe_version"
else
    bad "firing JSON must carry provenance fields" "$out"
fi
out="$(run_hook "$TMP/clean.json" --json)"
if printf '%s' "$out" | jq -e 'has("probe_binary") and has("probe_version")' >/dev/null 2>&1; then
    ok "healthy JSON carries probe_binary + probe_version"
else
    bad "healthy JSON must carry provenance fields" "$out"
fi

# --- 4. the test hook stays binary-independent (PL-213) ----------------------
# Fixtures must not need any termlink on PATH. Point TERMLINK_BIN at something
# that does not exist; with the hook set, the run must still work.
out="$(TERMLINK_BIN=/nonexistent/termlink TERMLINK_CHARTER_DRIFT_TEST_JSON="$TMP/clean.json" \
        bash "$SCRIPT" --no-heartbeat --json 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | jq -e '.ok==true' >/dev/null 2>&1; then
    ok "test hook works with no real binary present (PL-213)"
else
    bad "test hook must not require a binary" "rc=$rc; $out"
fi
if printf '%s' "$out" | jq -e '.probe_binary | test("test hook")' >/dev/null 2>&1; then
    ok "provenance says 'test hook' rather than naming a binary it never ran"
else
    bad "hooked runs must not claim a binary provenance" "$out"
fi

echo
echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
