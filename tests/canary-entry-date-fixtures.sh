#!/usr/bin/env bash
# T-3002 (C-39): every firing --quiet cron entry is dated.
#
# Five canaries appended undated entries to their cron logs, so a reader could not
# tell a blip from a chronic condition (the value review found 5 of 7 firing logs
# with no per-entry date). They now open each firing --quiet emission with the
# same `=== <UTC ISO> ===` frame substrate-preflight.sh already used.
#
# Each canary is driven through its own documented test seam — no hub, no network,
# no real ~/.termlink state, and --no-heartbeat where supported so a fixture run can
# never refresh a real cron heartbeat. Two properties per canary:
#   1. firing + --quiet  → output opens with a dated frame line
#   2. healthy + --quiet → output is EMPTY (empty-log = healthy must survive)
# The doorbell-mail canary has no offline seam for a firing sweep, so its frame is
# pinned structurally (the line exists and is gated on QUIET) rather than executed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
S="$REPO_ROOT/scripts"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $1 — $2"; }
DATE_RE='^=== [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z ===$'

dated()  { local name="$1" out="$2"
    if head -1 <<< "$out" | grep -qE "$DATE_RE"; then ok "$name: firing --quiet entry opens with a dated frame"
    else bad "$name: firing --quiet entry opens with a dated frame" "first line: $(head -1 <<< "$out")"; fi; }
silent() { local name="$1" out="$2"
    if [ -z "$out" ]; then ok "$name: healthy --quiet prints nothing"
    else bad "$name: healthy --quiet prints nothing" "got: $(head -1 <<< "$out")"; fi; }

echo "canary-entry-date fixtures (T-3002):"

# --- stuck-claims ---
echo '{"ok":true,"topic_count":3,"stuck_count":1,"shown":1,"only_stuck":true,"topics":[{"ok":true,"topic":"work-q","stuck":true,"active_count":1,"expired_count":0,"oldest_active_age_ms":120000}]}' > "$TMP/sc-fire.json"
echo '{"ok":true,"topic_count":3,"stuck_count":0,"shown":0,"only_stuck":true,"topics":[]}' > "$TMP/sc-ok.json"
dated  stuck-claims "$(TERMLINK_STUCK_CLAIMS_TEST_JSON="$TMP/sc-fire.json" bash "$S/check-stuck-claims-freshness.sh" --no-heartbeat --quiet 2>/dev/null)"
silent stuck-claims "$(TERMLINK_STUCK_CLAIMS_TEST_JSON="$TMP/sc-ok.json"  bash "$S/check-stuck-claims-freshness.sh" --no-heartbeat --quiet 2>/dev/null)"
# --json is never framed: a date line would make it unparseable.
# Capture first: the canary exits 1 when firing, which pipefail would report as a jq failure.
SCJ="$(TERMLINK_STUCK_CLAIMS_TEST_JSON="$TMP/sc-fire.json" bash "$S/check-stuck-claims-freshness.sh" --no-heartbeat --quiet --json 2>/dev/null)"
if jq -e . >/dev/null 2>&1 <<< "$SCJ"; then
    ok "stuck-claims: --quiet --json stays parseable (no frame)"
else bad "stuck-claims: --quiet --json stays parseable (no frame)" "jq rejected the output"; fi

# --- hook-counter-integrity ---
printf 'a=1\nb=2\n' > "$TMP/hc-clean"; printf 'a=1\na=2\n' > "$TMP/hc-dup"
dated  hook-counter "$(bash "$S/check-hook-counter-integrity.sh" --counter "$TMP/hc-dup"   --quiet --no-heartbeat 2>&1)"
silent hook-counter "$(bash "$S/check-hook-counter-integrity.sh" --counter "$TMP/hc-clean" --quiet --no-heartbeat 2>&1)"

# --- waker-liveness (class b: a state file whose pushwaker pid is dead) ---
echo '{"ok":true,"listeners":[]}' > "$TMP/wl.json"
mkdir -p "$TMP/wl-fire" "$TMP/wl-ok"
echo '{"agent_id":"fx","pushwaker_pid":999999,"pty_session":"fx-pty"}' > "$TMP/wl-fire/be-reachable-fx.state"
dated  waker-liveness "$(TERMLINK_WAKER_TEST_JSON="$TMP/wl.json" TERMLINK_WAKER_STATE_DIR="$TMP/wl-fire" bash "$S/check-waker-liveness-freshness.sh" --quiet --no-heartbeat 2>/dev/null)"
silent waker-liveness "$(TERMLINK_WAKER_TEST_JSON="$TMP/wl.json" TERMLINK_WAKER_STATE_DIR="$TMP/wl-ok"   bash "$S/check-waker-liveness-freshness.sh" --quiet --no-heartbeat 2>/dev/null)"

# --- stale-waker-code (a live waker pid started before the waker script's mtime) ---
PW="$TMP/pw.sh"; echo '# waker' > "$PW"; touch -d '+1 hour' "$PW"
mkdir -p "$TMP/sw-fire" "$TMP/sw-ok"
printf '{"agent_id":"fx","pushwaker_pid":%s}\n' "$$" > "$TMP/sw-fire/be-reachable-fx.state"
dated  stale-waker-code "$(STALE_WAKER_STATE_DIR="$TMP/sw-fire" STALE_WAKER_PW_SCRIPT="$PW" bash "$S/check-stale-waker-code-freshness.sh" --quiet --no-heartbeat 2>/dev/null)"
silent stale-waker-code "$(STALE_WAKER_STATE_DIR="$TMP/sw-ok"   STALE_WAKER_PW_SCRIPT="$PW" bash "$S/check-stale-waker-code-freshness.sh" --quiet --no-heartbeat 2>/dev/null)"

# --- fleet-doorbell-mail: structural pin (no offline firing seam) ---
if grep -qE "^\s*\[ \"\\\$QUIET\" = 1 \] && printf '=== %s ===" "$S/check-fleet-doorbell-mail-health.sh"; then
    ok "doorbell-mail: dated frame present and gated on --quiet"
else bad "doorbell-mail: dated frame present and gated on --quiet" "frame line not found"; fi

echo ""
echo "canary-entry-date fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
