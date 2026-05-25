#!/usr/bin/env bash
# T-1803 — non-destructive test for the Watchtower foreign-port-holder guard.
#
# Tests the decision helper _watchtower_port_holder_is_ours WITHOUT ever invoking
# do_start (which would touch the live dashboard). Proves:
#   - this project's running Watchtower identifies as ours          (true)
#   - a foreign service (dummy http.server, no /api/_identity)      (false)
#   - the check is read-only: the foreign holder survives it (the guard that uses
#     this helper will therefore never signal a service it can't claim as ours).
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
export PROJECT_ROOT="${PROJECT_ROOT:-$ROOT}"
export FRAMEWORK_ROOT="${FRAMEWORK_ROOT:-$ROOT/.agentic-framework}"

command -v curl >/dev/null 2>&1    || { echo "SKIP: curl not available"; exit 0; }
command -v python3 >/dev/null 2>&1 || { echo "SKIP: python3 not available"; exit 0; }
[ -f "$FRAMEWORK_ROOT/lib/watchtower.sh" ] || { echo "SKIP: lib/watchtower.sh not found"; exit 0; }
# shellcheck disable=SC1091
source "$FRAMEWORK_ROOT/lib/watchtower.sh"

# The hardened helper only exists once the T-1803 change has landed in the
# (vendored) lib. If a pre-T-1803 upstream was re-vendored, skip rather than fail.
type _watchtower_port_holder_is_ours >/dev/null 2>&1 || {
    echo "SKIP: _watchtower_port_holder_is_ours not defined (lib predates T-1803 hardening)"; exit 0; }

fail=0

# --- Positive: this project's running Watchtower is recognized as ours ---
our_port=""
[ -f "$PROJECT_ROOT/.context/working/watchtower.port" ] \
    && our_port="$(tr -d '[:space:]' < "$PROJECT_ROOT/.context/working/watchtower.port")"
if [ -n "$our_port" ] && curl -sf --max-time 2 "http://localhost:${our_port}/api/_identity" >/dev/null 2>&1; then
    if _watchtower_port_holder_is_ours "$our_port"; then
        echo "PASS positive: our Watchtower on :$our_port identifies as ours"
    else
        echo "FAIL positive: our Watchtower on :$our_port NOT recognized as ours"; fail=1
    fi
else
    echo "SKIP positive: no running Watchtower with /api/_identity on our port"
fi

# --- Negative: a foreign service (no /api/_identity) is NOT ours, and survives ---
dummy_port=""
for p in 39517 39518 39519 39521 39523; do
    if ! ss -tlnp 2>/dev/null | grep -q ":${p} "; then dummy_port="$p"; break; fi
done
if [ -z "$dummy_port" ]; then
    echo "SKIP negative: no free port for dummy foreign service"
else
    python3 -m http.server "$dummy_port" --bind 127.0.0.1 >/dev/null 2>&1 &
    dummy_pid=$!
    for _ in 1 2 3 4 5; do ss -tlnp 2>/dev/null | grep -q ":${dummy_port} " && break; sleep 0.5; done

    if _watchtower_port_holder_is_ours "$dummy_port"; then
        echo "FAIL negative: foreign service on :$dummy_port WRONGLY identified as ours"; fail=1
    else
        echo "PASS negative: foreign service on :$dummy_port correctly NOT ours"
    fi

    if kill -0 "$dummy_pid" 2>/dev/null; then
        echo "PASS read-only: foreign holder survived the identity check (never signaled)"
    else
        echo "FAIL read-only: foreign holder died during the identity check"; fail=1
    fi
    kill -KILL "$dummy_pid" 2>/dev/null || true   # our own dummy — safe to clean up
fi

if [ "$fail" = "0" ]; then echo "test-watchtower-guard: ALL PASS"; else echo "test-watchtower-guard: FAILURES"; fi
exit "$fail"
