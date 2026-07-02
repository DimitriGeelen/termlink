#!/usr/bin/env bash
# T-2316 (arc-004 WP1) — pure-helper unit test for the push-waker filter/dedup.
# Sources be-reachable-pushwaker.sh in library mode (no main run) and exercises
# pushwaker_extract_payload / pushwaker_decide / pushwaker_dedup_ok.
set -u

SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
BE_REACHABLE_PUSHWAKER_LIB=1 . "${SELF_DIR}/be-reachable-pushwaker.sh"

fail=0
check() { # desc expected actual
    if [ "$2" = "$3" ]; then
        echo "ok: $1"
    else
        echo "FAIL: $1 — expected '$2' got '$3'"; fail=1
    fi
}
check_exit() { # desc expected_rc actual_rc
    if [ "$2" = "$3" ]; then echo "ok: $1"; else echo "FAIL: $1 — expected rc $2 got $3"; fail=1; fi
}

# --- pushwaker_extract_payload ---
p="$(pushwaker_extract_payload '[push] inbox.queued seq=3: {"addressee_session_id":"bob","message_offset":9}')"
check "extract strips push prefix" '{"addressee_session_id":"bob","message_offset":9}' "$p"

# --- pushwaker_decide ---
d="$(pushwaker_decide '{"addressee_session_id":"bob","message_offset":7}' bob)"
check "frame matches self -> RING <offset>" "RING 7" "$d"

d="$(pushwaker_decide '{"addressee_session_id":"alice","message_offset":7}' bob)"
check "frame for other addressee -> SKIP" "SKIP other:alice" "$d"

d="$(pushwaker_decide '{"message_offset":7}' bob)"
check "missing addressee -> SKIP no-addressee" "SKIP no-addressee" "$d"

d="$(pushwaker_decide '{"addressee_session_id":"bob"}' bob)"
check "missing offset -> SKIP no-offset" "SKIP no-offset" "$d"

# --- pushwaker_dedup_ok (duplicate offset -> skip) ---
pushwaker_dedup_ok 100 "" 120; check_exit "first sighting rings (no last)" 0 $?
pushwaker_dedup_ok 100 90 120; check_exit "duplicate offset within ttl skips" 1 $?
pushwaker_dedup_ok 300 90 120; check_exit "same offset after ttl rings again" 0 $?

if [ "$fail" -eq 0 ]; then
    echo "RESULT: PASS"
else
    echo "RESULT: FAIL"
    exit 1
fi
