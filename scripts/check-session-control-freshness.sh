#!/usr/bin/env bash
# T-2557 (T-2468 charter-verb completeness) — session-control canary.
#
# TermLink's charter names FOUR core verbs; verb 4 is "control terminal sessions" —
# the founding verb: spawn a PTY-backed session, inject a command, capture its
# output. It has an affirmative on-demand prover (scripts/session-selftest.sh,
# T-2485: SPAWN a tmux-backed scratch session → EXEC `echo <sentinel>` and verify
# the inject→run→capture round-trip → CLEANUP) but — until this canary — ZERO
# passive daily detection. Session spawn/exec could regress and only a manual
# session-selftest.sh run would catch it. The prover is deterministic + local
# (uses `exec --json`, needs no live peer) and self-reaps its scratch session,
# making it ideal canary substrate.
#
# This canary runs the prover and translates its verdict:
#   selftest exit 0 (proven)  -> canary exit 0 (healthy)
#   selftest exit 1 (broken)  -> canary exit 1 (FIRE — verb-4 broken, stage named)
#   selftest exit 2 (tooling) -> canary exit 2 (tooling — hub down / no tmux; NOT a
#                                verb-4 regression, that is /preflight territory)
# Empty log = healthy — the same convention as the other canaries (CLAUDE.md).
#
# Exit codes: 0 healthy · 1 firing (session control broken) · 2 tooling error
set -u

SELFTEST="${SESSION_SELFTEST_BIN:-scripts/session-selftest.sh}"

QUIET=0
FORMAT=human
HUB=""
HEARTBEAT=1
HEARTBEAT_FILE=".context/working/.session-control-canary.heartbeat"

usage() {
    sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: check-session-control-freshness.sh [OPTIONS]
  --hub ADDR           Pass through to session-selftest.sh (default: local hub)
  --json               Emit a JSON envelope
  --quiet              Print only on firing (cron-friendly)
  --no-heartbeat       Skip touching the heartbeat companion
  -h, --help           This help

Test hook: TERMLINK_SESSION_CANARY_TEST_JSON=<file> + TERMLINK_SESSION_CANARY_TEST_RC=<n>
feed a canned session-selftest `--json` verdict + exit code for hub/tmux-independent
testing.

Exit: 0 healthy · 1 firing (verb-4 broken) · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --hub)          HUB="${2:-}"; shift 2 ;;
        --json)         FORMAT=json; shift ;;
        --quiet)        QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        -h|--help)      usage; exit 0 ;;
        *) echo "check-session-control: unknown arg: $1" >&2; exit 2 ;;
    esac
done

# Heartbeat FIRST (before the check) so /canaries can prove the canary ran even on
# a healthy cycle — mirrors the T-2290/T-2295/T-2556 convention.
if [ "$HEARTBEAT" -eq 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null && date -u +%Y-%m-%dT%H:%M:%SZ > "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# Run the prover (test hook short-circuits it for hub/tmux-independence).
if [ -n "${TERMLINK_SESSION_CANARY_TEST_JSON:-}" ]; then
    verdict="$(cat "${TERMLINK_SESSION_CANARY_TEST_JSON}" 2>/dev/null)"
    rc="${TERMLINK_SESSION_CANARY_TEST_RC:-0}"
else
    args=(--json)
    [ -n "$HUB" ] && args+=(--hub "$HUB")
    verdict="$(bash "$SELFTEST" "${args[@]}" 2>/dev/null)"
    rc=$?
fi

broken_stage="$(printf '%s' "$verdict" | jq -r '.broken_stage // "unknown"' 2>/dev/null || echo unknown)"

if [ "$FORMAT" = json ]; then
    # Re-emit a compact canary envelope keyed on the prover's exit code.
    state=$([ "$rc" -eq 0 ] && echo healthy || { [ "$rc" -eq 1 ] && echo firing || echo tooling; })
    printf '{"ok":%s,"state":"%s","selftest_rc":%s,"broken_stage":%s}\n' \
        "$([ "$rc" -eq 0 ] && echo true || echo false)" \
        "$state" "$rc" \
        "$(printf '%s' "$broken_stage" | jq -R . 2>/dev/null || echo '"unknown"')"
    exit "$rc"
fi

case "$rc" in
    0)
        [ "$QUIET" -eq 1 ] || echo "check-session-control: healthy (verb-4 spawn→exec→cleanup proven)"
        exit 0 ;;
    1)
        echo "check-session-control: verb-4 (control terminal sessions) BROKEN at stage '${broken_stage}' (T-2557)"
        echo "  The founding session-control verb failed its spawn→exec→capture round-trip."
        echo "  Reproduce: bash scripts/session-selftest.sh   (names the broken stage)"
        echo "  Then inspect: termlink list-sessions / termlink spawn / termlink exec <s> 'echo hi' --json"
        exit 1 ;;
    *)
        # exit 2 (or any non-0/1): tooling — hub down / no tmux / dep missing. This
        # is /preflight territory, NOT a verb-4 regression; log but do not fire.
        [ "$QUIET" -eq 1 ] || echo "check-session-control: tooling error (selftest rc=$rc — hub down or tmux/dep missing; not a verb-4 regression). Run /preflight." >&2
        exit 2 ;;
esac
