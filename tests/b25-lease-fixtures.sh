#!/usr/bin/env bash
# Fixture suite for B-25 (arc-011 plan unit P-4, T-3694): lease tightening and the waiting-mail
# signal K-34. Runs the hermetic tests that carry each listed E-n and prints one PASS line per
# E-n (or FAIL), and a DEFERRED line, with its reason, for each E-n of the plan row that this
# unit cannot serve in this repository state. Exit 0 only when every E-n in scope passed;
# 1 on a failure; 2 when the tests could not run.
#
#   bash tests/b25-lease-fixtures.sh            # the E-n
#   bash tests/b25-lease-fixtures.sh --mutants  # FP 3.1.b: every mutant must be KILLED
#
# The mutant run works on a clean export of HEAD (git archive, never a worktree) under
# $B25_MUT_DIR (default /var/tmp/b25-mutants), applies one source mutation at a time, runs the
# tests that must catch it, and requires them to FAIL. A mutation that no longer applies to the
# source is a tooling error (2), never a pass.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
export TMPDIR="${TMPDIR:-/var/tmp}" CARGO_TARGET_TMPDIR="${CARGO_TARGET_TMPDIR:-/var/tmp}"

if [ "${1:-}" = "--mutants" ]; then
    root="${B25_MUT_DIR:-/var/tmp/b25-mutants}"
    src="$root/src"
    export CARGO_TARGET_DIR="$root/target"
    mkdir -p "$root" || exit 2
    rc=0
    # mutant <id> <file> <test-cmd> <python-literal-old> <python-literal-new>
    mutant() {
        local id="$1" file="$2" cmd="$3" old="$4" new="$5"
        rm -rf "$src" && mkdir -p "$src" || return 2
        git archive HEAD | tar -x -C "$src" || { echo "TOOLING $id: export failed"; return 2; }
        if ! OLD="$old" NEW="$new" F="$src/$file" python3 -I -c '
import os,sys
p=os.environ["F"]; s=open(p).read(); o=os.environ["OLD"]
if s.count(o)!=1: sys.exit(3)
open(p,"w").write(s.replace(o,os.environ["NEW"],1))'; then
            echo "TOOLING $id: the mutation no longer applies to $file"; return 2
        fi
        local log="$root/$id.log"
        ( cd "$src" && eval "$cmd" ) >"$log" 2>&1
        local trc=$?
        if grep -q "^error\[E\|^error: could not compile" "$log"; then
            echo "TOOLING $id: the mutant does not compile ($log)"; return 2
        fi
        # No test summary and no panic = the tests never ran: tooling, never a survival or a kill.
        if ! grep -q "^test result:" "$log" && ! grep -q "panicked" "$log"; then
            echo "TOOLING $id: the tests did not run ($log)"; return 2
        fi
        if [ "$trc" -ne 0 ] && grep -q "FAILED\|panicked" "$log"; then
            echo "KILLED $id"; return 0
        fi
        echo "SURVIVED $id ($log)"; return 1
    }
    S=crates/termlink-session/src/main_lease.rs
    W=crates/termlink-sidecar/src/mail_waiting.rs
    # cargo takes ONE test-name filter: pass it after `--`, as a single word.
    UNIT="cargo test -p termlink-session --lib -- main_lease::tests::"
    INTEG="cargo test -p termlink --test main_lease_integration -- b25_"
    MW="cargo test -p termlink-sidecar --lib -- mail_waiting::tests::"
    run() { mutant "$@"; local r=$?; [ $r -gt $rc ] && rc=$r; return 0; }
    run M1-unknown-counts-as-human "$S" "${UNIT}e20" \
        '    if origin != Origin::Human {
        return Plan::Ignored; // FCW-52' \
        '    if origin == Origin::Automated {
        return Plan::Ignored; // FCW-52'
    run M2-tool-activity-renews "$S" "${UNIT}e2" \
        'ClaimKind::Tool => return Plan::AnswerOnly { why: "tool activity never renews main" },' \
        'ClaimKind::Tool => {}'
    run M3-absent-ignored-everywhere "$S" "$INTEG" \
        '        if !rep {
            let tag' \
        '        if false {
            let tag'
    run M4-first-origin-field-switches-the-harness "$S" "$INTEG" \
        'let rep = reporting.contains(&req.harness);' \
        'let rep = reporting.contains(&req.harness) || req.origin.is_some();'
    run M5-quiet-period-on-a-provisional-holder "$S" "${UNIT}e23" \
        '        if v.provisional {
            return Plan::OverProvisional' \
        '        if false {
            return Plan::OverProvisional'
    run M6-no-interactive-guard "$S" "${UNIT}e23" \
        'if req.interactive != Some(true) {' \
        'if false {'
    run M7-no-five-minute-check "$S" "${UNIT}e24" \
        'if age > v.timing.quiet_ms + margin {' \
        'if true {'
    run M8-any-touch-counts "$S" "${UNIT}e24_touch" \
        '                    && l.sender.is_some()
                    && l.sender == c.sender =>' \
        ' =>'
    run M9-unreadable-touch-log-read-as-empty "$S" "${UNIT}e162" \
        '    let touches = touches?;' \
        '    let touches = touches.unwrap_or(&[]);'
    run M10-renew-without-the-touch "$S" "$INTEG" \
        'if let Err(e) = self.post_side(&touch_topic(project), &ev).await {' \
        'if let Err(e) = self.post_side(&touch_topic(project), &ev).await.or(Ok::<u64, LeaseError>(0)) {'
    run M11-automated-claim-takes-a-vacancy "$S" "${UNIT}e25_model" \
        '        return Plan::Ignored; // FCW-52' \
        '        if v.holder.is_none() { return Plan::Take { reason: "mutant" }; }
        return Plan::Ignored;'
    run M12-oob-sent-without-a-sink "$W" "${MW}e28_oob" \
        '                Oob::NotAvailable // FCW-62' \
        '                Oob::SentAt(now_ms)'
    run M13-exact-instance-mail-counts "$W" "${MW}e28_exact" \
        'let w: Vec<&Waiting> = mail.iter().filter(|m| m.role).collect();' \
        'let w: Vec<&Waiting> = mail.iter().collect();'
    run M14-oob-step-every-tick "$W" "${MW}e28_oob_sent" \
        'if self.active && oldest_age >= OOB_MS && self.oob_set_by_me.is_none() {' \
        'if self.active && oldest_age >= OOB_MS {'
    [ "$rc" = 0 ] && echo "ALL MUTANTS KILLED"
    exit "$rc"
fi

out="$(mktemp)"
trap 'rm -f "$out" "$out".*' EXIT
cargo test -p termlink-session --lib main_lease >"$out.unit" 2>&1
cargo test -p termlink --test main_lease_integration >"$out.integ" 2>&1
cargo test -p termlink-sidecar --lib mail_waiting >"$out.mw" 2>&1
for f in unit integ mw; do
    grep -q "^test result:" "$out.$f" || { echo "TOOLING: the $f tests did not run"; tail -20 "$out.$f"; exit 2; }
done
cat "$out.unit" "$out.integ" "$out.mw" >"$out"
rc=0
# check <label> <test-name>...: every named test ran and passed (a name absent from the run is a FAIL).
check() {
    local label="$1"; shift
    local ok=1 t
    for t in "$@"; do
        if ! grep -q "^test \(main_lease::tests::\|mail_waiting::tests::\)\?${t} \.\.\. ok" "$out"; then
            echo "  missing or failed: $t"; ok=0
        fi
    done
    if [ "$ok" = 1 ]; then echo "PASS $label"; else echo "FAIL $label"; rc=1; fi
}
echo "== P-4 (T-3694): B-25 lease tightening and K-34 waiting-mail signal"
check "E-20 (only human-origin claims renew, acquire or take over; others recorded ignored: origin)" \
    e20_only_human_origin_counts_in_every_state b25_reporting_harness_provisional_ignored_touch_and_fast_takeover
check "E-21 (tool activity never renews; SessionStart takes a vacant lease provisionally)" \
    e21_tool_never_renews_and_session_start_is_provisional b25_reporting_harness_provisional_ignored_touch_and_fast_takeover
check "E-22 (absent origin counts as built until the operator switches the harness; per harness; NID)" \
    e22_origin_reporting_is_per_harness_and_only_an_operator_record_sets_it b25_e22_absent_origin_counts_as_built_until_the_operator_switches_the_harness
check "E-23 (provisional: taken at once, lapses to VACANT, main while it holds; headless takes nothing)" \
    e23_provisional_holder_never_blocks_and_lapses_to_vacant
check "E-24 (fast takeover after 5 min of holder human silence, never at 4:59, never pinned; Touch by key and claim)" \
    e24_fast_takeover_at_five_minutes_not_before_never_pinned e24_touch_counts_only_from_the_holders_key_and_claim e24_cross_host_margin_until_the_hub_stamps_receive_time
check "E-25 (bounded model check of the lease invariants; the checker sees a non-human take)" \
    e25_model_check_lease_invariants e25_control_the_checker_catches_a_non_human_take
check "E-26 (lapse at 15 min whatever the agent does; resume or take at once)" \
    e26_lapse_without_human_renewal_then_resume_or_take_at_once
check "E-27 (L2: a doorbell line reported automated never renews or takes)" \
    e27_doorbell_line_never_renews_or_takes
check "E-28 (waiting-mail definition: 10 min, not-reading conditions, OOB honest, once per episode/hour, re-raise, clear)" \
    e28_rises_at_ten_minutes_for_every_not_reading_condition e28_a_reading_main_of_this_agent_raises_nothing \
    e28_exact_instance_mail_never_counts e28_oob_not_available_without_a_sink_and_never_sent \
    e28_oob_sent_once_per_episode_and_hour_and_never_without_a_receipt e28_second_receiver_shows_the_holders_outcome \
    e28_reraise_once_per_urgent_message_and_per_generation e28_clears_on_hand_over_and_counts_episodes e28_one_non_main_line_per_generation
check "E-30 (R-68 standing tests re-run with B-25: both end in a visible takeover)" \
    r68_lapse_during_an_open_obligation_ends_in_a_visible_takeover r68_stuck_holder_with_live_reader_ends_in_a_visible_takeover
check "E-31 (W2 report generator on a seeded record set)" \
    e31_report_counts_renewals_by_basis_and_flags_non_human_holder_changes
check "E-84 (R3 prompt-origin parity scenario published and decided by the lease)" \
    e84_prompt_origin_parity_scenario_matches_the_lease
check "E-139 (monitor: waiting > 10 min raises; no OOB 'sent' without the channel)" \
    e28_rises_at_ten_minutes_for_every_not_reading_condition e28_oob_not_available_without_a_sink_and_never_sent
check "E-142 (monitor: one main, takeovers recorded, no counted renewal with origin automated)" \
    e25_model_check_lease_invariants e31_report_counts_renewals_by_basis_and_flags_non_human_holder_changes
check "E-162 (unreadable touch log: no fast takeover; unpostable Touch: unknown; unpostable Ignored: ignored-unrecorded)" \
    e162_unreadable_touch_log_never_allows_fast_takeover b25_e162_unpostable_touch_is_unknown_and_unpostable_ignored_is_unrecorded
check "E-163 (main-lease test seams refused outside a test estate)" \
    seam_outside_estate_is_refused_naming_the_variable seam_outside_estate_is_unknown_naming_it
check "NID (the built lease tests still pass unchanged)" \
    two_sessions_one_main_and_takeover_after_lapse_quiet_and_cooldown operator_pin_overrides_a_live_holder_and_is_audited \
    hub_down_is_unknown_never_main same_session_from_another_process_renews_and_releases_other_sessions_refused
echo "DEFERRED E-28 wiring (status main.mail_waiting, needs-attention entry, non-main line): the sidecar tick lives in daemon.rs, barred to this unit (T-3693)"
echo "DEFERRED E-31 live part (7 days on EP-7b, L7): needs the rollout P-35"
echo "DEFERRED E-153 (receiver-side origin check feeding the claim): the receiver's input counters are in meter.rs/hook.rs, barred to this unit (T-3692/T-3693)"
echo "DEFERRED E-161 (role-mail-only withhold): plan unit P-5 (T-3695) by the plan row"
[ "$rc" = 0 ] && echo "ALL IN-SCOPE PASS"
exit "$rc"
