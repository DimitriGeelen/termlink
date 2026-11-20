#!/usr/bin/env bash
# T-3708 (arc-011 P-24 slice a, B-14): the unit's fixture suite. One PASS line per evidence item E-n of slice (a)
# (plan exit: "fixture suite for the unit exits 0 with a PASS line per listed E-n"); the mutant list is
# crates/termlink-session/tests/mutants/run-t3708.py (45/45 killed, docs/reports/T-3708-mutants.md).
# E-60 and E-61 belong to slice (b), T-3832.
#
# Run from the repo root:  bash tests/t3708-evidence.sh
# Exit 0 only if every E-n passes; a test that cannot run (no test hub binary) is a FAIL, never a skip.
set -u
cd "$(dirname "$0")/.." || exit 2
RT="$(mktemp -d -t t3708-evidence-rt.XXXXXX)"
trap 'rm -rf "$RT"' EXIT
fails=0

# run <E-n> <what> <cargo test args...>
run() {
    local en="$1" what="$2"; shift 2
    local out
    if out="$(env -u PROJECT_ROOT TERMLINK_RUNTIME_DIR="$RT" cargo test -p termlink-sidecar "$@" 2>&1)" \
        && ! grep -qE 'running 0 tests|^test result: .* 0 passed' <<<"$out"; then
        echo "PASS $en — $what"
    else
        echo "FAIL $en — $what"
        grep -E 'panicked|FAILED|error' <<<"$out" | head -5 | sed 's/^/     /'
        fails=$((fails+1))
    fi
}

lib() { echo --lib -- --exact "$@"; }

# shellcheck disable=SC2046
run E-19 "a frozen field reads unknown past its bound, never its last value" $(lib fields::tests::e19_frozen_field_turns_unknown_after_its_bound)
# shellcheck disable=SC2046
run E-55 "fields stamped; never-measured reads unknown" $(lib fields::tests::e55_never_measured_reads_unknown_not_measured_since_start)
run E-55 "status schema, restart = unknown, additive keys, switch (API socket)" --test t3708_status
run E-55 "fields fresh from a real pass; p95 <= 5 s hub up / stopped; silent hub not waited on (L4)" --test t3708_live -- t3708_e55 t3708_status_never_waits
# shellcheck disable=SC2046
run E-56 "refused confirmations with a fresh heartbeat read NOT CLEAR" $(lib fields::tests::e56_refused_confirmations_with_fresh_heartbeat_read_not_clear)
# shellcheck disable=SC2046
run E-56 "a healthy receiver reads CLEAR after the hand-over" $(lib fields::tests::e56_healthy_receiver_reads_clear_after_the_hand_over)
# shellcheck disable=SC2046
run E-57 "idle 48 h with nothing pending is not bad" $(lib fields::tests::e57_idle_48h_with_nothing_pending_is_not_bad)
# shellcheck disable=SC2046
run E-57 "bad exactly after READY + 2 ticks" $(lib fields::tests::e57_bad_exactly_after_ready_plus_two_ticks)
# shellcheck disable=SC2046
run E-57 "bad 2 ticks after a hook call that missed it, across a restart" $(lib fields::tests::e57_bad_two_ticks_after_a_hook_call_that_missed_it_and_across_a_restart)
# shellcheck disable=SC2046
run E-58 "strength flags never claim what a root or shared host lacks" $(lib host::tests::root_host_reads_absent_never_present)
run E-58 "strength flags match ps -o user= on this host (L4)" --test t3708_live -- t3708_e58
# shellcheck disable=SC2046
run E-59 "skew flagged at 3 s, delays unusable at 45 s, NTP unchecked without tools" $(lib host::tests::skew_flags_at_the_stated_thresholds)
# shellcheck disable=SC2046
run E-59 "a wall jump moves neither freshness nor ages" $(lib fields::tests::e59_wall_jump_moves_neither_freshness_nor_ages)
run E-59 "deadlines across +/-1 h jumps on the real clock path (receiver hand-over included)" --test t3708_clock
run E-134 "every status call leaves a bounded latency line (with the 200-call run)" --test t3708_status

if [ "$fails" = 0 ]; then echo "ALL PASS (T-3708 slice a)"; exit 0; fi
echo "$fails FAILED"; exit 1
