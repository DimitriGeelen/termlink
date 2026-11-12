#!/usr/bin/env bash
# Fixture suite for the B-27 follow-ups (arc-011 plan units P-1 T-3692, P-2 T-3693).
# Runs the meter's hermetic tests that carry each listed E-n and prints one PASS line per
# E-n (or FAIL). Exit 0 only when every listed E-n passed; 2 when the tests could not run.
#
#   bash tests/b27-followups-fixtures.sh p1    # P-1: E-156, E-158, QR-2 restart (persisted last_prompt)
#   bash tests/b27-followups-fixtures.sh p2    # P-2: E-156 READY order on (boot_id, mono_ns), clock step
#   bash tests/b27-followups-fixtures.sh       # both
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
export TMPDIR="${TMPDIR:-/var/tmp}" CARGO_TARGET_TMPDIR="${CARGO_TARGET_TMPDIR:-/var/tmp}"
unit="${1:-all}"
out="$(mktemp)"
trap 'rm -f "$out"' EXIT

cargo test -p termlink-sidecar --test meter_tests -- --list >"$out.list" 2>/dev/null || { echo "TOOLING: cannot build or list meter_tests"; exit 2; }
cargo test -p termlink-sidecar --test meter_tests >"$out" 2>&1
grep -q "^test result:" "$out" || { echo "TOOLING: meter_tests did not run"; tail -20 "$out"; exit 2; }
rc=0
# check <label> <test-name-prefix>...: every named test exists and passed.
check() {
    local label="$1"; shift
    local ok=1 t
    for t in "$@"; do
        if ! grep -q "^${t}.*: test" "$out.list"; then echo "  missing test: $t"; ok=0; continue; fi
        if grep -q "^test ${t}.* \.\.\. FAILED" "$out" || ! grep -q "^test ${t}.* \.\.\. ok" "$out"; then
            echo "  failed: $t"; ok=0
        fi
    done
    if [ "$ok" = 1 ]; then echo "PASS $label"; else echo "FAIL $label"; rc=1; fi
}

if [ "$unit" = p1 ] || [ "$unit" = all ]; then
    echo "== P-1 (T-3692): last_prompt persisted with READY"
    check "E-156 (P-1: FCW-3 order, FCW-6 restored READY absent to the gate, FCW-8 paths)" \
        fcw3_ready_ordering fcw8_transcript_growth_and_refusals p1_restart_reads_the_persisted_last_prompt p1_turn_stop_and_later_reports_survive_restarts
    check "E-158 (P-1: FCW-17 no turn stop, FCW-18 ledger unreadable)" \
        e18_client_input_extends_and_fcw17_no_stop e18_agent_conversation_and_fcw18
    check "QR-2 restart (persisted last_prompt read back, other boot / damaged state not used)" \
        p1_restart_reads_the_persisted_last_prompt p1_turn_stop_and_later_reports_survive_restarts p1_other_boot_or_damaged_state_is_not_used
fi
if [ "$unit" = p2 ] || [ "$unit" = all ]; then
    echo "== P-2 (T-3693): READY ordered on (boot_id, mono_ns), receive stamp as fallback"
    check "E-156 (P-2: FCW-3 on (boot_id, mono_ns), receive-stamp fallback, order kept across a restart, FCW-8 paths)" \
        fcw3_ready_ordering p2_reports_without_the_fields_are_ordered_by_receive_stamp p2_order_survives_a_restart fcw8_transcript_growth_and_refusals
    check "clock-step (a backward wall step of 10 min does not freeze READY)" \
        p2_backward_wall_step_does_not_freeze_ready
    check "hooks send boot_id and mono_ns (and fall back for an older sidecar)" \
        p2_hooks_send_boot_id_and_mono_ns p2_hook_falls_back_for_a_sidecar_without_the_fields
fi
rm -f "$out.list"
[ "$rc" = 0 ] && echo "ALL PASS ($unit)"
exit "$rc"
