#!/usr/bin/env bash
# tests/pickup-canary-selffilter-fixtures.sh — T-2691 regression fixtures.
#
# Pins scripts/check-framework-pickup-freshness.sh against canned topic NDJSON:
#
#   1. own-only topic          -> healthy, exit 0 (the bug: used to fire)
#   2. inbound present         -> fires, exit 1, counts ONLY the inbound ones
#   3. unknown attribution     -> fires (fail-safe: cannot prove it is ours)
#   4. mixed                   -> firing count excludes own; own count reported
#   5. suppression is loud     -> the own-count line is always printed
#   6. --ack advances past own -> marker reaches the true max offset
#   7. filter is load-bearing  -> overriding SELF_PROJECT changes what fires
#   8. attribution fallbacks   -> source_project and agent_id attribute like from_project
#
# Assertion 5 is the important one. The first implementation of the suppression notice
# contained double quotes IN A COMMENT; because that python program is embedded in a
# double-quoted shell string, the quote closed the string and silently truncated the
# program. The counts still looked correct (that code ran before the truncation point)
# while the tail of the report vanished. If someone reintroduces a double quote into
# that block, assertion 5 fails.
#
# Host-independent (PL-213): uses FW_PICKUP_TEST_NDJSON, never contacts a hub.
#
# Usage: bash tests/pickup-canary-selffilter-fixtures.sh
# Exit:  0 = all pass, 1 = a fixture regressed.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/check-framework-pickup-freshness.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  PASS  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1" >&2; [ -n "${2:-}" ] && printf '          %s\n' "$2" >&2; }

[ -r "$SCRIPT" ] || { echo "pickup-canary-selffilter-fixtures: cannot read $SCRIPT" >&2; exit 2; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

MARKER="$SCRATCH/seen-offset"
HB="$SCRATCH/heartbeat"

# Emit one NDJSON envelope line.
env_line() { # offset, metadata-json, payload
    printf '{"offset":%d,"msg_type":"note","metadata":%s,"payload":"%s"}\n' "$1" "$2" "$3"
}

SELF='{"from_project":"010-termlink"}'
PEER='{"from_project":"050-email-archive"}'
NOMETA='null'

# --- fixture files ----------------------------------------------------------
OWN_ONLY="$SCRATCH/own-only.ndjson"
: > "$OWN_ONLY"
env_line 0 "$SELF" "our filing one"  >> "$OWN_ONLY"
env_line 1 "$SELF" "our filing two"  >> "$OWN_ONLY"

INBOUND="$SCRATCH/inbound.ndjson"
: > "$INBOUND"
env_line 0 "$SELF" "our filing"      >> "$INBOUND"
env_line 1 "$PEER" "peer filing"     >> "$INBOUND"

UNKNOWN="$SCRATCH/unknown.ndjson"
: > "$UNKNOWN"
env_line 0 "$NOMETA" "unattributed"  >> "$UNKNOWN"

MIXED="$SCRATCH/mixed.ndjson"
: > "$MIXED"
env_line 0 "$SELF"   "ours a"        >> "$MIXED"
env_line 1 "$PEER"   "peer a"        >> "$MIXED"
env_line 2 "$SELF"   "ours b"        >> "$MIXED"
env_line 3 "$NOMETA" "unattributed"  >> "$MIXED"
env_line 4 "$SELF"   "ours c"        >> "$MIXED"

FALLBACK="$SCRATCH/fallback.ndjson"
: > "$FALLBACK"
env_line 0 '{"source_project":"010-termlink"}' "ours via source_project" >> "$FALLBACK"
env_line 1 '{"agent_id":"010-termlink"}'       "ours via agent_id"       >> "$FALLBACK"

echo "T-2691 pickup-canary self-filter fixtures"
echo

run() { # ndjson-file, extra args...
    local f="$1"; shift
    rm -f "$MARKER"
    FW_PICKUP_TEST_NDJSON="$f" \
    FW_PICKUP_CANARY_MARKER="$MARKER" \
    HEARTBEAT_FILE="$HB" \
    bash "$SCRIPT" "$@" 2>&1
}

# --- 1. own-only topic reads healthy ----------------------------------------
out=$(run "$OWN_ONLY"); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "own-only topic is healthy (exit 0)"
else
    bad "own-only topic is healthy" "exit $rc; out: $out"
fi

# --- 2. inbound fires, counting only the inbound one ------------------------
out=$(run "$INBOUND"); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "1 unprocessed filing"; then
    ok "inbound filing fires, own filing not counted"
else
    bad "inbound filing fires, own filing not counted" "exit $rc; out: $out"
fi

# --- 3. unknown attribution still fires (fail-safe) -------------------------
out=$(run "$UNKNOWN"); rc=$?
if [ "$rc" -eq 1 ]; then
    ok "unknown attribution still fires (fail-safe direction)"
else
    bad "unknown attribution still fires" "exit $rc; out: $out"
fi

# --- 4. mixed: firing count excludes own ------------------------------------
out=$(run "$MIXED"); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "2 unprocessed filing"; then
    ok "mixed topic fires on 2 (1 peer + 1 unattributed), excludes 3 own"
else
    bad "mixed topic firing count excludes own" "exit $rc; out: $out"
fi

# --- 5. suppression is LOUD (shell-string truncation guard) -----------------
if printf '%s' "$out" | grep -q "3 own filing(s) from 010-termlink not counted"; then
    ok "suppression count is reported (double-quote truncation guard)"
else
    bad "suppression count is reported" "the own-count line is missing; out: $out"
fi

# --- 6. JSON envelope carries own_count -------------------------------------
jout=$(run "$MIXED" --json)
if printf '%s' "$jout" | grep -q '"own_count": 3' \
   && printf '%s' "$jout" | grep -q '"self_project": "010-termlink"'; then
    ok "--json carries own_count and self_project"
else
    bad "--json carries own_count and self_project" "got: $jout"
fi

# --- 7. --ack advances past own filings -------------------------------------
run "$MIXED" --ack >/dev/null 2>&1
acked=$(cat "$MARKER" 2>/dev/null)
if [ "$acked" = "4" ]; then
    ok "--ack advances to true max offset (4), including own filings"
else
    bad "--ack advances to true max offset" "marker=$acked, expected 4"
fi

# --- 8. filter is load-bearing ----------------------------------------------
rm -f "$MARKER"
out=$(FW_PICKUP_TEST_NDJSON="$MIXED" FW_PICKUP_CANARY_MARKER="$MARKER" \
      HEARTBEAT_FILE="$HB" FW_PICKUP_SELF_PROJECT=nonesuch bash "$SCRIPT" 2>&1); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "5 unprocessed filing"; then
    ok "filter is load-bearing (no self match -> all 5 fire)"
else
    bad "filter is load-bearing" "exit $rc; out: $out"
fi

# --- 9. attribution fallbacks -----------------------------------------------
out=$(run "$FALLBACK"); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "source_project and agent_id attribute like from_project"
else
    bad "source_project and agent_id attribute like from_project" "exit $rc; out: $out"
fi

# --- 10. no double quotes in the embedded python block ----------------------
# Structural guard for the same failure assertion 5 catches behaviourally.
# Drop the first line (the `PARSED="$(... python3 -c "` opener, whose quotes are the
# string delimiters themselves) and the last (the closing `" 2>/dev/null )"`). What
# remains is the python program body, which must be double-quote-free.
blk=$(sed -n '/^PARSED=/,/^" 2>\/dev\/null )"/p' "$SCRIPT" | sed '1d;$d')
if printf '%s' "$blk" | grep -q '"'; then
    bad "embedded python block contains no double quote" \
        "a double quote closes the shell string and silently truncates the program"
else
    ok "embedded python block contains no double quote"
fi

echo
echo "pickup-canary-selffilter-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
