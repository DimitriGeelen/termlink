#!/usr/bin/env bash
# T-2754 — Shared topic reaper for cron-driven and test-suite provers.
#
# WHY THIS EXISTS
#
# Provers mint a unique topic per run (`agent-conv-selftest-$$-<ts>-$RANDOM` and
# friends) and, before T-2421, had no way to remove it — `channel delete` did not
# exist. `test-agent-conversation-status.sh` said so in a comment. T-2421 shipped
# the verb; the provers were never migrated, so every run leaks one topic forever.
#
# One of those provers runs on a DAILY cron (fleet-doorbell-mail-canary →
# check-fleet-doorbell-mail-health.sh → agent-conversation-selftest.sh), which
# makes it a guard that permanently pollutes the substrate it monitors — the
# T-2709 shape, where a guard's own debris is what teaches an operator to stop
# reading it.
#
# Measured before this landed: 771 topics holding 13,705 records total on the
# local hub — topic-COUNT growth, not record growth. T-2424 swept 851 such topics
# once by hand; T-2426 bounded re-accumulation for five named namespaces in the
# RETENTION dimension only. Neither deletes a registry entry, and neither covers
# the patterns leaking today.
#
# Source via:
#   _self="${BASH_SOURCE[0]}"
#   _libdir="$(cd "$(dirname "$_self")" && pwd)/lib"
#   # shellcheck source=/dev/null
#   . "$_libdir/reap-topic.sh"
#
# Public API:
#   reap_topic <topic-name> [extra termlink args...]
#
# CONTRACT — best-effort, never fatal:
#   Returns 0 ALWAYS. A prover's verdict is about the thing it proves, never about
#   whether cleanup happened to succeed. A failed delete emits one stderr warning
#   and nothing else. This is why callers can safely wire it into `trap ... EXIT`
#   without the trap rewriting their exit code.
#
#   Callers using a trap must still preserve their own status explicitly:
#       trap 'rc=$?; reap_topic "$topic"; exit $rc' EXIT
#   `reap_topic` returning 0 is not sufficient on its own — the trap body's LAST
#   command determines the exit status unless `exit $rc` is explicit.
#
# Opt-out:
#   TERMLINK_KEEP_TEST_TOPICS=1  — skip reaping entirely and say so. For an
#   operator debugging a failed run who needs the topic to still be there.
#
# Env:
#   TERMLINK        — termlink binary (default: "termlink")
#   REAP_TIMEOUT    — per-delete timeout in seconds (default: 20)

# Guard against double-sourcing.
if [ -n "${__REAP_TOPIC_SH_LOADED:-}" ]; then
    return 0 2>/dev/null || true
fi
__REAP_TOPIC_SH_LOADED=1

# Cached result of the `channel delete` capability probe:
#   ""  = not yet probed, "1" = verb available, "0" = verb absent
__REAP_DELETE_SUPPORTED=""

# reap_topic <topic-name> [extra args...]
#
# Deletes a topic minted by a prover. Always returns 0.
reap_topic() {
    local topic="${1:-}"
    shift 2>/dev/null || true

    if [ -z "$topic" ]; then
        # Nothing to do. Not a warning: provers call this from a trap that may
        # fire before the topic variable is ever assigned (e.g. early usage exit).
        return 0
    fi

    if [ "${TERMLINK_KEEP_TEST_TOPICS:-0}" = "1" ]; then
        echo "reap-topic: TERMLINK_KEEP_TEST_TOPICS=1 — retaining '$topic' for inspection" >&2
        return 0
    fi

    local tl="${TERMLINK:-termlink}"

    if ! command -v "$tl" >/dev/null 2>&1; then
        echo "reap-topic: '$tl' not on PATH — cannot reap '$topic' (leaked)" >&2
        return 0
    fi

    # Probe once per process. A binary predating T-2421 has no `channel delete`;
    # that is a known-and-named skip, not an error. Mirrors the same guard in
    # scripts/sweep-test-debris.sh:53.
    if [ -z "$__REAP_DELETE_SUPPORTED" ]; then
        if "$tl" channel delete --help >/dev/null 2>&1; then
            __REAP_DELETE_SUPPORTED=1
        else
            __REAP_DELETE_SUPPORTED=0
        fi
    fi

    if [ "$__REAP_DELETE_SUPPORTED" = "0" ]; then
        echo "reap-topic: this termlink binary has no 'channel delete' (needs >= T-2421 build) — '$topic' leaked" >&2
        return 0
    fi

    local to="${REAP_TIMEOUT:-20}"
    local out
    if out="$(timeout "$to" "$tl" channel delete "$topic" --yes "$@" 2>&1)"; then
        return 0
    fi

    # Best-effort by contract: name what leaked, keep the caller's verdict intact.
    echo "reap-topic: failed to delete '$topic' (leaked): ${out:-<no output>}" >&2
    return 0
}
