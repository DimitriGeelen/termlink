#!/usr/bin/env bash
# Fixture suite for arc-011 plan unit P-13 (T-3701, build item B-3; V-3 retention ceilings).
# Runs the hermetic tests that carry each listed E-n and prints one PASS line per E-n (or FAIL).
# Exit 0 only when every listed E-n passed; 1 on a failure; 2 when the tests could not run.
#
#   bash tests/t3701-b3-fixtures.sh            # E-67, E-169, T-3382 bound (the working tree)
#   bash tests/t3701-b3-fixtures.sh mutants    # FP 3.1.b: every mutant must be KILLED
#
# Mutants run on a `git archive HEAD` export under .context/working/t3701/mut (gitignored),
# never on the shared working tree (other workers build crates/ concurrently). Each mutant
# edits ONE line in the export, runs the test(s) named for it, and must turn them red.
# A mutant whose pattern is not found, or whose export does not compile, is TOOLING (exit 2):
# "could not look" is never reported as killed.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
export TMPDIR="${TMPDIR:-/var/tmp}" CARGO_TARGET_TMPDIR="${CARGO_TARGET_TMPDIR:-/var/tmp}"
mode="${1:-fixtures}"
work=.context/working/t3701
mkdir -p "$work" || exit 2

HUB_LIB=(-p termlink-hub --lib mail_guard::tests)
BUS_IT=(-p termlink-bus --test t3701_removal_guard)
BUS_LIB=(-p termlink-bus --lib removal_guard::tests)

# run_tests <outfile> <cargo args...>: run, keep output; 2 when the suite did not run at all.
run_tests() {
    local out="$1"; shift
    cargo test "$@" >"$out" 2>&1
    grep -q "^test result:" "$out" || return 2
    return 0
}

if [ "$mode" = fixtures ]; then
    out="$work/fx.out"; : >"$out"
    for args in "HUB_LIB" "BUS_IT" "BUS_LIB"; do
        declare -n a="$args"
        run_tests "$out.part" "${a[@]}" || { echo "TOOLING: cannot run cargo test ${a[*]}"; tail -20 "$out.part"; exit 2; }
        cat "$out.part" >>"$out"
    done
    rc=0
    # check <label> <test-name>...: every named test ran and passed.
    check() {
        local label="$1"; shift
        local ok=1 t
        for t in "$@"; do
            if grep -q "^test \(.*::\)\?${t} \.\.\. ok" "$out"; then continue; fi
            if grep -q "^test \(.*::\)\?${t} \.\.\. FAILED" "$out"; then echo "  failed: $t"; else echo "  missing: $t"; fi
            ok=0
        done
        if [ "$ok" = 1 ]; then echo "PASS $label"; else echo "FAIL $label"; rc=1; fi
    }
    echo "== P-13 (T-3701): B-3 admin log, sweep/trim refusal on open mail, V-3 ceilings"
    check "E-67 (admin row per admin RPC, kept >= 30 days; sweep/set-retention/trim refused across an open offset; V-3 final first, open last resort with needs-attention; control mutant removes open mail)" \
        e67_every_admin_rpc_gets_one_intent_and_one_result_row \
        e67_admin_rows_survive_rotation_for_30_days \
        e67_sweep_trim_set_retention_refused_across_the_open_offset_with_reason \
        e67_finality_is_read_from_the_stage_records \
        e67_finality_survives_the_trim_of_its_own_replied_record \
        e67_ceiling_final_first_then_open_last_resort_raises_needs_attention \
        e67_control_mutant_without_the_open_offset_check_removes_open_mail \
        e67_sweep_across_an_open_offset_is_refused_and_removes_nothing \
        e67_control_sweep_without_the_guard_removes_the_open_message \
        e67_trim_across_an_open_offset_is_refused \
        e67_control_trim_without_the_guard_removes_the_open_message \
        e67_set_retention_that_would_remove_an_open_message_is_refused \
        e67_ceiling_removes_final_first_and_keeps_open_while_a_final_remains \
        e67_ceiling_removes_open_only_as_last_resort_with_needs_attention \
        e67_hub_ceiling_takes_the_oldest_final_across_topics \
        e67_ceilings_off_is_the_rollback \
        e67_unguarded_topics_are_unchanged \
        periodic_sweep_keeps_open_mail_and_removes_final \
        preview_matches_each_policy \
        ceiling_plan_takes_finals_first_and_open_only_as_last_resort
    check "E-169 (finality unreadable -> sweep/trim/set-retention refused, ceiling removes nothing; admin RPC refused and never acts when its row cannot be written)" \
        e169_finality_unreadable_is_refused \
        e169_finality_unreadable_refuses_sweep_trim_and_set_retention \
        e169_ceiling_with_unreadable_finality_removes_nothing \
        e169_admin_rpc_whose_log_row_cannot_be_written_is_refused_and_never_acts
    check "T-3382 (forever inbox topics bounded by V-3 on post)" \
        t3382_forever_inbox_is_bounded_on_post e67_ceiling_fires_on_post_for_a_forever_inbox
    [ "$rc" = 0 ] && echo "ALL PASS (P-13)"
    exit "$rc"
fi

if [ "$mode" != mutants ]; then echo "usage: $0 [fixtures|mutants]"; exit 2; fi

# ---- mutants (FP 3.1.b) ----------------------------------------------------
mut="$work/mut"
src="$mut/src"
rm -rf "$src"; mkdir -p "$src" || exit 2
git archive HEAD | tar -x -C "$src" || { echo "TOOLING: cannot export HEAD"; exit 2; }
export CARGO_TARGET_DIR="$PWD/$mut/target"
( cd "$src" && cargo test -p termlink-hub -p termlink-bus --no-run >"$PWD/../build.out" 2>&1 ) \
    || { echo "TOOLING: the HEAD export does not build"; tail -20 "$mut/build.out"; exit 2; }

rc=0
# mutant <id> <file> <from> <to> <cargo test args...>
mutant() {
    local id="$1" file="$2" from="$3" to="$4"; shift 4
    local f="$src/$file"
    cp "$f" "$f.orig"
    if ! python3 -I - "$f" "$from" "$to" <<'PY'
import sys
p, a, b = sys.argv[1:4]
s = open(p).read()
if s.count(a) != 1:
    sys.exit(1)
open(p, "w").write(s.replace(a, b))
PY
    then
        echo "TOOLING $id: pattern not found exactly once in $file"; mv "$f.orig" "$f"; rc=2; return
    fi
    local out="$mut/$id.out"
    ( cd "$src" && cargo test "$@" >"$PWD/../$id.out" 2>&1 )
    local trc=$?
    mv "$f.orig" "$f"
    if grep -q "^error\(\[E[0-9]*\]\)\?:" "$out" && ! grep -q "^test result:" "$out"; then
        echo "TOOLING $id: mutant does not compile"; [ "$rc" = 0 ] && rc=2; return
    fi
    if [ "$trc" -ne 0 ] && grep -q "FAILED" "$out"; then
        echo "KILLED $id"
    else
        echo "SURVIVED $id  <-- the tests did not notice"; rc=1
    fi
}

G=crates/termlink-bus/src/removal_guard.rs
L=crates/termlink-bus/src/lib.rs
H=crates/termlink-hub/src/mail_guard.rs
A=crates/termlink-hub/src/rpc_audit.rs
S=crates/termlink-hub/src/server.rs

mutant M1-no-open-offset-check "$G" "            if !fin {" "            if false && !fin {" \
    -p termlink-bus --test t3701_removal_guard e67_sweep_across_an_open_offset
mutant M2-ceiling-ignores-finality "$G" "        if fin {" "        if true {" \
    -p termlink-bus --test t3701_removal_guard e67_ceiling_removes_final_first
mutant M3-unreadable-record-reads-final "$H" 'Err(why) => Err(format!("{topic} offset {off}: {why}")),' 'Err(_why) => Ok(true),' \
    -p termlink-hub --lib mail_guard::tests::e169_finality_unreadable
mutant M4-unreadable-hub-record-ignored "$H" "        self.refresh(bus, &mut cache)?;" "        let _ = self.refresh(bus, &mut cache);" \
    -p termlink-hub --lib mail_guard::tests::e169_finality_unreadable
mutant M5-admin-acts-without-its-row "$S" "    let row = match crate::rpc_audit::admin_intent(&req.method, caller, &req.params) {" \
    "    let row = match crate::rpc_audit::admin_intent(&req.method, caller, &req.params).or_else(|_| Ok::<String, std::io::Error>(String::new())) {" \
    -p termlink-hub --lib mail_guard::tests::e169_admin_rpc
mutant M6-rotation-drops-young-admin-rows "$A" ".is_none_or(|ts| u128::from(ts) >= cutoff)" ".is_none_or(|ts| u128::from(ts) >= now_ms)" \
    -p termlink-hub --lib mail_guard::tests::e67_admin_rows_survive
mutant M7-forever-inbox-not-bounded "$L" "        if self.is_guarded(topic) {
            return self.apply_mail_ceiling(topic);" "        if false {
            return self.apply_mail_ceiling(topic);" \
    -p termlink-bus --test t3701_removal_guard e67_ceiling_fires_on_post_for_a_forever_inbox
mutant M8-stage-records-not-followed "$H" "        STAGE_TYPE => payload_field" "        STAGE_TYPE if false => payload_field" \
    -p termlink-hub --lib mail_guard::tests::e67_finality_is_read_from_the_stage_records
mutant M9-sweep-not-an-admin-rpc "$A" "    termlink_protocol::control::method::CHANNEL_SWEEP,
" "" \
    -p termlink-hub --lib mail_guard::tests::e67_every_admin_rpc
mutant M10-periodic-sweep-cuts-open-mail "$G" ".filter(|r| r.2)" ".filter(|_r| true)" \
    -p termlink-bus --test t3701_removal_guard periodic_sweep_keeps_open_mail
mutant M11-set-retention-unchecked "$L" "            self.check_all_final(g.as_ref(), topic, &offsets)?;
        }
        self.meta.set_topic_retention" "            let _ = (g, offsets);
        }
        self.meta.set_topic_retention" \
    -p termlink-bus --test t3701_removal_guard e67_set_retention_that_would_remove

case "$rc" in
    0) echo "ALL MUTANTS KILLED (P-13)";;
    1) echo "MUTANT SURVIVED (P-13)";;
    *) echo "TOOLING: some mutants could not be judged";;
esac
exit "$rc"
