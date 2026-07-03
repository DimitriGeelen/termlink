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

# --- dm rail decision (T-2324, arc-004 S2) ---
# The dm rail reuses pushwaker_decide with the SELF-FP as the match arg (the
# non-sender half of dm:<a>:<b>, a 16-hex identity fingerprint). Same helper,
# so the three outcomes below prove the dm-rail wiring: ring on self-addressed,
# skip other-addressed, and (self-fp unset ⇒ empty match) skip everything.
SELF_FP="80d5a3a1f60d3741"
OTHER_FP="011eaa3f94456938"
d="$(pushwaker_decide "{\"addressee_session_id\":\"$SELF_FP\",\"message_offset\":42}" "$SELF_FP")"
check "dm.queued addressed to self-fp -> RING <offset>" "RING 42" "$d"

d="$(pushwaker_decide "{\"addressee_session_id\":\"$OTHER_FP\",\"message_offset\":42}" "$SELF_FP")"
check "dm.queued for another fp -> SKIP" "SKIP other:$OTHER_FP" "$d"

# self-fp unset (empty match) — a real addressee never equals "" so the rail
# rings nothing; this mirrors run_waker leaving the dm rail disabled when
# --self-fp is absent (back-compat, inbox-rail only).
d="$(pushwaker_decide "{\"addressee_session_id\":\"$SELF_FP\",\"message_offset\":42}" "")"
check "dm rail with empty self-fp -> SKIP (never rings)" "SKIP other:$SELF_FP" "$d"

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
