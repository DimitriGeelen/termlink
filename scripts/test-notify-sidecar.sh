#!/usr/bin/env bash
# T-2294 (arc-003 reliable-comms, V3a) — tests for the deterministic notify
# sidecar + self-check pair (notify-sidecar.sh + notify-check.sh).
#
# The core mechanism is hub-INDEPENDENT: the sidecar's mail probe is driven by
# the TERMLINK_NOTIFY_TEST_UNREAD hook (mirrors TERMLINK_GROWTH_TEST_JSON), so
# every verdict path — MAIL / CLEAR / DEAF-stale / DEAF-missing — is proven
# without a live hub. That is the point of the design: the self-check must be
# trustworthy when the hub is unreachable.
#
# Covers:
#   T1  sidecar --help → 0
#   T2  sidecar unknown arg → 2
#   T3  sidecar missing --agent-id → 2
#   T4  sidecar --interval 1 → 2 (below min 5)
#   T5  check --help → 0
#   T6  check unknown arg → 2
#   T7  check missing --agent-id → 2
#   T8  MAIL: sidecar --once unread=3 → check exit 10 + "MAIL"
#   T9  CLEAR: sidecar --once unread=0 → check exit 0 + "CLEAR"
#   T10 DEAF (stale): backdated heartbeat → check exit 3 + "DEAF"
#   T11 DEAF (missing): no heartbeat → check exit 3 + "DEAF"
#   T12 check --json emits parseable {"state":...} with matching exit
#   T13 sidecar writes flag with pending= and a numeric heartbeat
#   T14 heartbeat is ALWAYS written even when unread=0 (alive ≠ deaf)
set -u

SIDECAR="${SIDECAR:-scripts/notify-sidecar.sh}"
CHECK="${CHECK:-scripts/notify-check.sh}"

PASS=0; FAIL=0; SKIP=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

ND="$(mktemp -d)/notify"

# -------- usage / arg-guard tests --------
echo "T1: sidecar --help → 0"
out="$(bash "$SIDECAR" --help 2>/dev/null)"; rc=$?
{ [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "Usage:"; } && pass "T1 exit=$rc" || fail "T1 exit=$rc"

echo "T2: sidecar unknown arg → 2"
bash "$SIDECAR" --bogus >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T2 exit=$rc" || fail "T2 expected 2 got $rc"

echo "T3: sidecar missing --agent-id → 2"
bash "$SIDECAR" --once >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T3 exit=$rc" || fail "T3 expected 2 got $rc"

echo "T4: sidecar --interval 1 → 2"
bash "$SIDECAR" --agent-id x --interval 1 --once >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T4 exit=$rc" || fail "T4 expected 2 got $rc"

echo "T5: check --help → 0"
out="$(bash "$CHECK" --help 2>/dev/null)"; rc=$?
{ [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "Usage:"; } && pass "T5 exit=$rc" || fail "T5 exit=$rc"

echo "T6: check unknown arg → 2"
bash "$CHECK" --bogus >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T6 exit=$rc" || fail "T6 expected 2 got $rc"

echo "T7: check missing --agent-id → 2"
bash "$CHECK" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T7 exit=$rc" || fail "T7 expected 2 got $rc"

# -------- MAIL verdict --------
echo "T8: MAIL — sidecar unread=3 → check exit 10"
TERMLINK_NOTIFY_TEST_UNREAD=3 TERMLINK_NOTIFY_TEST_LATEST_TOPIC="dm:aa:bb" \
    bash "$SIDECAR" --agent-id A8 --notify-dir "$ND" --once >/dev/null 2>&1
out="$(bash "$CHECK" --agent-id A8 --notify-dir "$ND" 2>/dev/null)"; rc=$?
{ [ "$rc" -eq 10 ] && printf '%s' "$out" | grep -qF "MAIL"; } && pass "T8 exit=$rc ($out)" || fail "T8 exit=$rc out=$out"

# -------- CLEAR verdict --------
echo "T9: CLEAR — sidecar unread=0 → check exit 0"
TERMLINK_NOTIFY_TEST_UNREAD=0 bash "$SIDECAR" --agent-id A9 --notify-dir "$ND" --once >/dev/null 2>&1
out="$(bash "$CHECK" --agent-id A9 --notify-dir "$ND" 2>/dev/null)"; rc=$?
{ [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "CLEAR"; } && pass "T9 exit=$rc ($out)" || fail "T9 exit=$rc out=$out"

# -------- DEAF (stale heartbeat) --------
echo "T10: DEAF (stale) — backdated heartbeat → check exit 3"
TERMLINK_NOTIFY_TEST_UNREAD=0 bash "$SIDECAR" --agent-id A10 --notify-dir "$ND" --once >/dev/null 2>&1
echo "$(( ($(date +%s) - 999) * 1000 ))" > "$ND/A10.heartbeat"
out="$(bash "$CHECK" --agent-id A10 --notify-dir "$ND" --deaf-after 45 2>/dev/null)"; rc=$?
{ [ "$rc" -eq 3 ] && printf '%s' "$out" | grep -qF "DEAF"; } && pass "T10 exit=$rc ($out)" || fail "T10 exit=$rc out=$out"

# -------- DEAF (missing heartbeat) --------
echo "T11: DEAF (missing) — no sidecar ever ran → check exit 3"
out="$(bash "$CHECK" --agent-id never-ran --notify-dir "$ND" 2>/dev/null)"; rc=$?
{ [ "$rc" -eq 3 ] && printf '%s' "$out" | grep -qF "DEAF"; } && pass "T11 exit=$rc" || fail "T11 exit=$rc out=$out"

# -------- JSON verdict --------
echo "T12: check --json emits {\"state\":...} matching exit"
TERMLINK_NOTIFY_TEST_UNREAD=2 bash "$SIDECAR" --agent-id A12 --notify-dir "$ND" --once >/dev/null 2>&1
out="$(bash "$CHECK" --agent-id A12 --notify-dir "$ND" --json 2>/dev/null)"; rc=$?
state="$(printf '%s' "$out" | jq -r '.state // empty' 2>/dev/null)"
{ [ "$rc" -eq 10 ] && [ "$state" = "mail" ]; } && pass "T12 json state=$state exit=$rc" || fail "T12 state=$state exit=$rc out=$out"

# -------- flag format --------
echo "T13: sidecar flag has pending= and numeric heartbeat"
TERMLINK_NOTIFY_TEST_UNREAD=5 bash "$SIDECAR" --agent-id A13 --notify-dir "$ND" --once >/dev/null 2>&1
p="$(grep -E '^pending=' "$ND/A13.flag" 2>/dev/null | cut -d= -f2)"
hb="$(cat "$ND/A13.heartbeat" 2>/dev/null)"
{ [ "$p" = "5" ] && printf '%s' "$hb" | grep -qE '^[0-9]+$'; } && pass "T13 pending=$p hb=$hb" || fail "T13 pending=$p hb=$hb"

# -------- heartbeat always written (alive ≠ deaf) even with zero mail --------
echo "T14: heartbeat written even when unread=0"
TERMLINK_NOTIFY_TEST_UNREAD=0 bash "$SIDECAR" --agent-id A14 --notify-dir "$ND" --once >/dev/null 2>&1
[ -s "$ND/A14.heartbeat" ] && pass "T14 heartbeat present with 0 mail" || fail "T14 heartbeat missing"

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
