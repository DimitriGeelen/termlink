#!/usr/bin/env bash
# Fixture suite for B-16 (arc-011 plan unit P-7, T-3697): the doorbell D+ and the 300 s watchdog K-33, the SI-1
# line, strict delivery, back-off, the macOS interim `ps` veto and the session binding for S1 and `command.notify`.
# Runs the hermetic tests that carry each listed E-n and prints one PASS line per E-n (or FAIL), a LIMIT line for
# the stated limit probe (E-14), and a DEFERRED line, with its reason, for each part that needs a later unit, a
# live wave or another host. Exit 0 only when every E-n in scope passed; 1 on a failure; 2 when the tests could
# not run.
#
#   bash tests/b16-doorbell-fixtures.sh            # the E-n
#   bash tests/b16-doorbell-fixtures.sh --mutants  # FP 3.1.b: every mutant must be KILLED
#
# The mutant run works on a clean clone of HEAD (`git clone --shared`, never a worktree, never cd'd into: cargo
# runs with --manifest-path) at $B16_MUT_DIR/src (default .context/working/t3697/src, gitignored), applies one
# source mutation at a time, runs the tests that must catch it, and requires them to FAIL. A mutation that no
# longer applies to the source is a tooling error (2), never a pass.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
unset PROJECT_ROOT # T-3755: cargo builds must not see the framework's PROJECT_ROOT
export TMPDIR="${TMPDIR:-/var/tmp}" CARGO_TARGET_TMPDIR="${CARGO_TARGET_TMPDIR:-/var/tmp}"

if [ "${1:-}" = "--mutants" ]; then
    root="${B16_MUT_DIR:-$PWD/.context/working/t3697}"
    src="$root/src"
    export CARGO_TARGET_DIR="${B16_MUT_TARGET:-/var/tmp/b16-mutants/target}"
    mkdir -p "$root" || exit 2
    rm -rf "$src" && git clone -q --shared --no-checkout . "$src" && git -C "$src" checkout -q --detach "$(git rev-parse HEAD)" \
        || { echo "TOOLING: clone of HEAD failed"; exit 2; }
    rc=0
    only="${B16_MUT_ONLY:-}"
    # mutant <id> <file> <test-cmd> <python-literal-old> <python-literal-new>
    mutant() {
        local id="$1" file="$2" cmd="$3" old="$4" new="$5"
        if [ -n "$only" ] && [ "${id%%-*}" != "$only" ]; then return 0; fi
        git -C "$src" checkout -q -- . && git -C "$src" clean -qfd || { echo "TOOLING $id: reset of the clone failed"; return 2; }
        if ! OLD="$old" NEW="$new" F="$src/$file" python3 -I -c '
import os,sys
p=os.environ["F"]; s=open(p).read(); o=os.environ["OLD"]
if s.count(o)!=1: sys.exit(3)
open(p,"w").write(s.replace(o,os.environ["NEW"],1))'; then
            echo "TOOLING $id: the mutation no longer applies to $file"; return 2
        fi
        local log="$root/$id.log"
        eval "$cmd" >"$log" 2>&1
        local trc=$?
        if grep -q "^error\[E\|^error: could not compile" "$log"; then
            echo "TOOLING $id: the mutant does not compile ($log)"; return 2
        fi
        if ! grep -q "^test result:" "$log" && ! grep -q "panicked" "$log"; then
            echo "TOOLING $id: the tests did not run ($log)"; return 2
        fi
        if [ "$trc" -ne 0 ] && grep -q "FAILED\|panicked" "$log"; then
            echo "KILLED $id"; return 0
        fi
        echo "SURVIVED $id ($log)"; return 1
    }
    D=crates/termlink-sidecar/src/doorbell.rs
    M=crates/termlink-sidecar/src/meter.rs
    H=crates/termlink-session/src/handler.rs
    MP="--manifest-path $src/Cargo.toml"
    TU="cargo test $MP -p termlink-sidecar --test b16_doorbell"
    TL="cargo test $MP -p termlink-sidecar --test b16_live"
    TS="cargo test $MP -p termlink-session --lib -- handler::tests::si1 handler::tests::command_notify_b16"
    run() { mutant "$@"; local r=$?; [ $r -gt $rc ] && rc=$r; return 0; }
    # E-1.5: drop condition (d); read a missing/not-ready READY as ready.
    run M1-drop-conversation-condition "$D" "$TU" \
        '        Conversation::Operator | Conversation::Unknown(_) => return Decision::Hold("conversation"), // FCW-23' \
        '        Conversation::Operator | Conversation::Unknown(_) => p,'
    run M2-missing-ready-read-as-ready "$D" "$TU" \
        '        return Decision::Hold("not-ready"); // FCW-21
    };' \
        '        return Decision::Ring { ids: vec![p[0].id.clone()], convs: vec![], count: 1, ready_seq: 1 };
    };'
    # E-2.5 / E-2.6: each re-check removed.
    run M3-no-kill-switch-recheck "$D" "$TU" \
        '    if !p.switch_on() {' \
        '    if false && !p.switch_on() {'
    run M4-ready-read-only-before-the-veto "$D" "$TU" \
        '        if !ready_ok(p) {
            return held(&mut checks, "ready", &mut row);
        }
        if !conv_compatible' \
        '        if !conv_compatible'
    run M5-no-reread-after-the-veto "$D" "$TU" \
        '    match p.input_activity(session) {
        Ok(fg) if is_harness(h, &fg) => {
            if typing(&fg, p.mono()) {
                return held(&mut checks, "typing", &mut row);
            }
        }
        _ => return held(&mut checks, "foreground", &mut row),
    }
    if via == Via::Doorbell {' \
        '    if via == Via::Doorbell {'
    run M6-veto-ignored "$D" "$TU" \
        '        Veto::Active(sig) => return held(&mut checks, &format!("veto:{sig}"), &mut row), // FCW-30' \
        '        Veto::Active(_) => checks.push(json!("veto:ignored")),'
    run M7-no-conversation-reread "$D" "$TU" \
        '            return held(&mut checks, "conversation", &mut row); // FCW-32' \
        '            checks.push(json!("conversation:ignored"));'
    # E-3.5: no id validation in the line.
    run M8-no-id-validation "$H" "$TU" \
        '        if id.len() != 32 || !id.bytes().all(|b| b.is_ascii_digit() || (b'"'"'a'"'"'..=b'"'"'f'"'"').contains(&b)) {' \
        '        if id.is_empty() {'
    run M9-session-types-bad-params "$H" "$TS" \
        '            Err(why) => return invalid(why),
        };
        let keys = vec![KeyEntry::Text(line), KeyEntry::Key("Enter".to_string())];' \
        '            Err(_) => ids.join(" "),
        };
        let keys = vec![KeyEntry::Text(line), KeyEntry::Key("Enter".to_string())];'
    # E-4.5: a ring counted as delivered.
    run M10-ring-counted-as-delivered "$D" "$TL" \
        'pub const ATTEMPTED: &str = "ATTEMPTED";' \
        'pub const ATTEMPTED: &str = "HANDED_OVER";'
    # E-5.5: re-ring every tick; re-type on an unsubmitted line; no back-off.
    run M11-retype-unsubmitted "$D" "$TU" \
        '            RingState::AwaitSubmit | RingState::Unsubmitted | RingState::Stalled => Err("unsubmitted"),' \
        '            RingState::Stalled => Err("unsubmitted"),
            RingState::AwaitSubmit | RingState::Unsubmitted => Ok(()),'
    run M12-no-backoff "$D" "$TU" \
        '                if tick < self.t0 + offset(self.k + 1) {' \
        '                if false {'
    # E-5.6 control: no STALLED.
    run M13-no-stalled "$D" "$TU" \
        '                self.state = RingState::Stalled;
                self.stalled_since = Some(mono);' \
        '                self.state = RingState::Unsubmitted;'
    # E-5.7: no MAX_ATTEMPTS bound.
    run M14-no-attempts-bound "$D" "$TU" \
        '        .filter(|m| m.attempts < MAX_ATTEMPTS)' \
        '        .filter(|m| m.attempts < u32::MAX)'
    # E-7.5: the watchdog checks READY; no audit row.
    run M15-watchdog-checks-ready "$D" "$TU" \
        '    // 2 READY re-read (doorbell only)
    if via == Via::Doorbell {' \
        '    // 2 READY re-read (doorbell only)
    if true {'
    run M16-no-admin-row "$D" "$TU" \
        '    if let Err(e) = p.append(&row, via == Via::Watchdog) {' \
        '    if let Err(e) = p.append(&row, false) {'
    # E-7.8 control (FCW-198): the wrong session of several.
    run M17-choose-inverted "$D" "$TU" \
        '    (0..cands.len()).max_by(' \
        '    (0..cands.len()).min_by('
    # E-10.5: unknown S2 read as no activity.
    run M18-s2-unknown-is-silence "$D" "$TU" \
        '    if !w.s2_known {' \
        '    if false && !w.s2_known {'
    # E-11.5: a 299 s interval (safety); a fire only once per session (liveness).
    run M19-interval-299s "$D" "$TU" \
        '    if w.now.saturating_sub(since) < V18_MS {' \
        '    if w.now.saturating_sub(since) < V18_MS - 1_000 {'
    run M20-fires-only-once "$D" "$TU" \
        '        return Wd::Counting; // FCW-38
    }
    Wd::Due' \
        '        return Wd::Counting; // FCW-38
    }
    if w.last_fire.is_some() {
        return Wd::Counting;
    }
    Wd::Due'
    # E-12.5: the switch read removed from the watchdog path.
    run M21-switch-not-read-by-watchdog "$D" "$TU" \
        '    if !p.switch_on() {' \
        '    if via == Via::Doorbell && !p.switch_on() {'
    # E-159.5: types without a durable intent.
    run M22-types-without-intent "$D" "$TU" \
        '        return (RingOut::Held("intent-unrecorded".into()), row);' \
        '        row["intent"] = json!("unrecorded");'
    run M23-restart-forgets-the-line "$D" "$TU" \
        '            st.sched.state = if via == Via::Watchdog { RingState::Stalled } else { RingState::Unsubmitted };' \
        '            st.sched.state = RingState::Idle;'
    # E-160.5: unmeasurable treated as quiet; no child check in the ps veto.
    run M24-unmeasurable-is-quiet "$D" "$TU" \
        '        Veto::Unmeasurable(_) if via == Via::Watchdog => return held(&mut checks, "veto-unmeasurable", &mut row), // FCW-41' \
        '        Veto::Unmeasurable(_) if false => return held(&mut checks, "veto-unmeasurable", &mut row),'
    run M25-ps-veto-no-child-check "$D" "$TU" \
        '        if running || !helper {' \
        '        if running && !helper {'
    # E-161.5: an unreadable switch read as on.
    run M26-odd-switch-flag-is-on "$D" "$TL" \
        '    std::fs::symlink_metadata(store.dir().join(SWITCH_FLAG)).is_ok_and(|m| m.is_file())' \
        '    std::fs::symlink_metadata(store.dir().join(SWITCH_FLAG)).is_ok()'
    # E-2.7 control: the foreground matched by pid alone (as good as comm).
    run M27-foreground-by-pid-only "$D" "$TU" \
        '    fg.leader_pid == Some(h.pid) && fg.exe.is_some() && fg.exe == h.exe && fg.script == h.script' \
        '    fg.leader_pid == Some(h.pid)'
    # Session binding: no /proc ancestry; a token of the wrong scope.
    run M28-binding-without-ancestry "$D" "$TL" \
        '    let anc = ancestors(proc_root, harness_pid);' \
        '    let anc: Vec<u32> = vec![];'
    run M29-observe-token "$M" "$TL" \
        'termlink_session::auth::PermissionScope::Notify, &tl, 60)' \
        'termlink_session::auth::PermissionScope::Observe, &tl, 60)'
    [ "$rc" = 0 ] && echo "ALL MUTANTS KILLED"
    exit "$rc"
fi

logs="${B16_LOG_DIR:-$PWD/.context/working/t3697/fixture-logs}"
mkdir -p "$logs" || exit 2
out="$logs/all.log" # kept, so a failure can be read afterwards
cargo test -p termlink-sidecar --test b16_doorbell >"$out.u" 2>&1
cargo test -p termlink-sidecar --test b16_live >"$out.l" 2>&1
cargo test -p termlink-session --lib -- handler::tests::si1 handler::tests::command_notify >"$out.s" 2>&1
for f in u l s; do
    grep -q "^test result:" "$out.$f" || { echo "TOOLING: the $f tests did not run"; tail -20 "$out.$f"; exit 2; }
done
cat "$out.u" "$out.l" "$out.s" >"$out"
rc=0
# check <label> <test-name>...: every named test ran and passed (a name absent from the run is a FAIL).
check() {
    local label="$1"; shift
    local ok=1 t
    for t in "$@"; do
        if ! grep -q "^test \(handler::tests::\)\?${t} \.\.\. ok" "$out"; then
            echo "  missing or failed: $t"; ok=0
        fi
    done
    if [ "$ok" = 1 ]; then echo "PASS $label"; else echo "FAIL $label"; rc=1; fi
}
echo "== P-7 (T-3697): B-16 doorbell D+ and the 300 s watchdog K-33"
check "E-1 (ring gate: 16-row table, one ring per agent per tick, decisions recorded)" e1_ring_gate_sixteen_rows e1_one_session_per_agent_per_tick_and_decisions_recorded
check "E-2 (re-checks in order, each fail closed; re-read after the veto; is_harness binding, never comm)" e2_rechecks_each_fail_closed_and_clean_run_logs_interval e2_6_reread_after_the_veto e2_7_is_harness_binding
check "E-3 (SI-1 line: 10^5 seeded cases match or refuse; the session refuses bad params; typed into local shells exits 127)" e3_si1_line_fuzz e3_line_typed_into_shells_is_command_not_found si1_line_test_vector_and_refusals command_notify_b16_form_validates_count_and_ids
check "E-4 (ATTEMPTED never HANDED_OVER, still owed, HANDOVER_UNCONFIRMED once at 90 s)" e4_ring_is_attempted_then_unconfirmed e4_store_attempted_is_not_handed_over
check "E-5 (back-off 0/4/12/32/52/72/92/112, no re-type, one watchdog line then STALLED, MAX_ATTEMPTS)" e5_6_backoff_schedule_with_submits e5_6_unsubmitted_then_one_watchdog_then_stalled e5_7_max_attempts_bound e5_schedule_table
check "E-6 (L2 part: a replayed line for handed-over mail is counted stray)" e6_stray_replay_is_counted
check "E-7 (watchdog at 300 s independent of READY, admin row, F-1 typing re-check, one fire per agent per tick, coalesced entry)" e7_watchdog_fires_at_300s_independent_of_ready e7_ready_busy_entry_coalesced_per_day e7_6_typing_recheck_after_the_veto_holds_the_watchdog e7_8_one_fire_per_agent_per_tick
check "E-8 (tier 1: kill switch, foreground, veto hold the watchdog with one row each)" e8_watchdog_checks_hold_with_rows
check "E-9 (inflow is not activity; a repeated ready BUSY does not move S2 on the meter)" e9_inflow_is_not_activity e9_6_repeated_ready_busy_does_not_move_s2_on_the_meter
check "E-10 (S2 unknown: no watchdog, status 'watchdog: unavailable')" e10_s2_unknown_watchdog_unavailable
check "E-11 (safety and liveness over 10^4 seeded 2 h traces)" e11_watchdog_bound_and_liveness_property
check "E-12 (tier 1: the switch stops both typed paths within the tick; switch on resumes)" e12_kill_switch_stops_doorbell_and_watchdog e161_kill_switch_reads
check "E-98 (B-16 re-run: an open claim is never rung; the doorbell has no claim or hand-over path)" e98_open_claim_is_not_rung
check "E-121 (tier 1: READY BUSY never rings; the ring path reads no output)" e121_screen_is_never_an_input
check "E-138 (tier 1: no pending mail in a silent unvetoed session waits past 300 s + 1 tick; every fire has a row)" e11_watchdog_bound_and_liveness_property e7_watchdog_fires_at_300s_independent_of_ready
check "E-159 (one session per agent, no intent no keystroke, notify timeout is unknown, restart table)" e159_ring_durability e1_one_session_per_agent_per_tick_and_decisions_recorded
check "E-160 (unmeasurable veto holds with one entry per day; ps veto child/cpu/idle; real ps tree; degraded status; unwritable row types nothing)" e160_unmeasurable_veto_and_macos_ps_veto e160_ps_veto_on_a_real_tree e159_ring_durability
check "E-161 (B-16 part: an unreadable kill switch is off for the typed paths; the pull refuses delivery-off)" e161_kill_switch_reads
check "Session binding for S1 and command.notify (live PTY: bound from /proc, S1 read with a minted notify token, the SI-1 line typed)" live_binding_reads_s1_and_types_the_si1_line
check "Status lines (PD-1): doorbell switch, ring state, watchdog available/unavailable/degraded (macOS, ps veto), H-11 counters" status_lines_switch_and_watchdog e160_unmeasurable_veto_and_macos_ps_veto
if grep -q "^test e14_limit_check_to_keystroke_race \.\.\. ok" "$out"; then
    echo "LIMIT E-14 (a turn starting between the last re-check and the keystrokes receives the line; counted by H-11 (d)): the stated limit, never PASS"
else
    echo "FAIL E-14 (the observed outcome is not the stated limit)"; rc=1
fi
if [ -z "$(command -v sidecar 2>/dev/null)" ]; then
    echo "PASS E-125 (this host: no installed command named sidecar)"
else
    echo "FAIL E-125 (this host has a command named sidecar: $(command -v sidecar))"; rc=1
fi
echo "DEFERRED E-6 L4 part (history-restored line in testdev-real, EP-5): needs the W2 estate, plan unit P-9 (T-3699)"
echo "DEFERRED E-8 tier 2 (bash in the foreground receives zero bytes, EP-4) and E-121 tier 2 (fake harness painting an idle prompt): the EP-4 PTY estate runs with the W2 measurement, P-9 (T-3699)"
echo "DEFERRED E-12 tier 4 (the W2 runme kill-switch drill extended to the typed path, CLI exit 4): P-9 (T-3699)"
echo "DEFERRED E-13 and E-136 (H-11 over 7 days on EP-7, the report and its generator): P-9 (T-3699), after a release with this unit is installed (P-8)"
echo "DEFERRED E-125 fleet part (command -v sidecar on every fleet host, daily) and E-138 production check: daily L7 checks, P-9/P-36; a new canary needs a slot under the guard cap (DP-3)"
echo "DEFERRED E-160 macOS runner (CI macOS job of T-3271, later the M1): the ps veto is proven here on Linux ps against a real tree; the full macOS veto is T-3688 (P-43)"
[ "$rc" = 0 ] && echo "ALL IN-SCOPE PASS"
exit "$rc"
