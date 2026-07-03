#!/usr/bin/env bash
# T-2328 (PL-237): unit tests for be-reachable.sh's push-wake dormancy WARN.
#
# Sources be-reachable.sh with BE_REACHABLE_LIB=1 (dispatcher suppressed) and
# asserts the pure helper `pushwake_dormancy_warn`:
#   - empty pty_session  -> emits a WARN naming "DORMANT" to STDERR
#   - bound pty_session   -> emits nothing (no false alarm on the happy path)
#   - the WARN goes to STDERR, not STDOUT (must not corrupt the success summary)
set -u

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
BE_REACHABLE_LIB=1 source "${SELF_DIR}/be-reachable.sh"

fails=0
ok()   { echo "ok: $1"; }
fail() { echo "FAIL: $1"; fails=$((fails+1)); }

# 1. Dormant path: empty pty_session -> WARN to stderr mentioning DORMANT.
err="$(pushwake_dormancy_warn "" 2>&1 >/dev/null)"
if echo "$err" | grep -qi "dormant"; then
    ok "empty pty_session -> WARN names DORMANT"
else
    fail "empty pty_session should WARN 'DORMANT' (got: '${err}')"
fi

# 2. Dormant WARN goes to STDERR, not STDOUT.
out="$(pushwake_dormancy_warn "" 2>/dev/null)"
if [ -z "$out" ]; then
    ok "empty pty_session -> nothing on STDOUT (WARN is stderr-only)"
else
    fail "dormancy WARN leaked to STDOUT: '${out}'"
fi

# 3. Happy path: bound pty_session -> no output at all (stdout+stderr empty).
both="$(pushwake_dormancy_warn "mypty" 2>&1)"
if [ -z "$both" ]; then
    ok "bound pty_session -> no dormancy WARN"
else
    fail "bound pty_session should be silent (got: '${both}')"
fi

# 4. Helper always returns 0 (non-fatal; must never block start).
pushwake_dormancy_warn "" >/dev/null 2>&1
rc_empty=$?
pushwake_dormancy_warn "x" >/dev/null 2>&1
rc_bound=$?
if [ "$rc_empty" -eq 0 ] && [ "$rc_bound" -eq 0 ]; then
    ok "helper returns 0 in both cases (never blocks start)"
else
    fail "helper must return 0 (empty=$rc_empty bound=$rc_bound)"
fi

echo "---"
if [ "$fails" -eq 0 ]; then
    echo "RESULT: PASS"
    exit 0
else
    echo "RESULT: FAIL ($fails)"
    exit 1
fi
