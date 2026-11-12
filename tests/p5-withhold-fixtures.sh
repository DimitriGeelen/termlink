#!/usr/bin/env bash
# Fixture suite for P-5 (arc-011 plan unit, T-3695): a non-main session is withheld ROLE mail
# only (ARC4 7.1.a, CR-4.11; 06 AL-12 5.12.c). Runs the hermetic tests that carry E-161 (the
# B-25 withhold part) and the unit's acceptance cases and prints one PASS line per case (or
# FAIL), and a DEFERRED line, with its reason, for what this unit cannot serve. Exit 0 only
# when every case in scope passed; 1 on a failure; 2 when the tests could not run.
#
#   bash tests/p5-withhold-fixtures.sh            # the cases
#   bash tests/p5-withhold-fixtures.sh --mutants  # FP 3.1.b: every mutant must be KILLED
#
# The mutant run works on a clean clone of HEAD (`git clone --shared`, never a worktree, never
# cd'd into: cargo runs with --manifest-path) at $P5_MUT_DIR/src (default
# .context/working/t3695/src, gitignored), applies one source mutation at a time, runs the
# tests that must catch it, and requires them to FAIL. A mutation that no longer applies is a
# tooling error (2), never a pass. M1 is the plan's negative control: the built all-mail
# withhold must fail the exact-instance case.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
export TMPDIR="${TMPDIR:-/var/tmp}" CARGO_TARGET_TMPDIR="${CARGO_TARGET_TMPDIR:-/var/tmp}"

if [ "${1:-}" = "--mutants" ]; then
    root="${P5_MUT_DIR:-$PWD/.context/working/t3695}"
    src="$root/src"
    export CARGO_TARGET_DIR="${P5_MUT_TARGET:-/var/tmp/p5-mutants/target}"
    mkdir -p "$root" || exit 2
    rm -rf "$src" && git clone -q --shared --no-checkout . "$src" && git -C "$src" checkout -q --detach "$(git rev-parse HEAD)" \
        || { echo "TOOLING: clone of HEAD failed"; exit 2; }
    rc=0
    # mutant <id> <file> <test-cmd> <old> <new>
    mutant() {
        local id="$1" file="$2" cmd="$3" old="$4" new="$5"
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
    A=crates/termlink-sidecar/src/api.rs
    D=crates/termlink-sidecar/src/deliver.rs
    T="cargo test --manifest-path $src/Cargo.toml -p termlink-sidecar --test p5_withhold"
    run() { mutant "$@"; local r=$?; [ $r -gt $rc ] && rc=$r; return 0; }
    run M1-built-all-mail-withhold "$A" "$T -- e161_non_main" \
        '        if !d.withhold_all() {' \
        '        if false {'
    run M2-role-mail-to-non-main "$D" "$T -- e161_non_main" \
        'cands.retain(|c| self.bound_session(&c.conv).as_deref() == Some(session));' \
        'cands.retain(|_| true);'
    run M3-any-binding-counts "$D" "$T -- p5_mail_bound_to_another" \
        'cands.retain(|c| self.bound_session(&c.conv).as_deref() == Some(session));' \
        'cands.retain(|c| self.bound_session(&c.conv).is_some());'
    run M4-line-not-once "$A" "$T -- e161_non_main" \
        'let notice = l.tell_once(session, generation);' \
        'let notice = true;'
    run M5-rollback-flag-ignored "$D" "$T -- p5_withhold_all" \
        '            Ok(_) => true,
            Err(e) => e.kind() != std::io::ErrorKind::NotFound,' \
        '            Ok(_) => false,
            Err(e) => e.kind() != std::io::ErrorKind::NotFound,'
    run M6-bound-mail-counted-as-role "$D" "$T -- p5_withhold_all_flag_restores" \
        'let role = all || self.bound_session(&c.conv).is_none();' \
        'let role = true;'
    run M7-held-mail-silent-under-rollback "$D" "$T -- p5_withhold_all_flag_restores" \
        'let role = all || self.bound_session(&c.conv).is_none();' \
        'let role = self.bound_session(&c.conv).is_none();'
    run M8-no-bind-at-hand-over "$D" "$T -- p5_binding" \
        'if let Err(e) = self.bind(&it.conv, &rec.session) {' \
        'if let Err(e) = Ok::<bool, String>(false) {'
    run M9-no-bind-at-send "$A" "$T -- p5_a_send" \
        '        && let Err(e) = d.bind(conv, sess)' \
        '        && let Err(e) = Ok::<bool, String>(false).map(|_| (d, conv, sess))'
    run M10-rebinding-last-wins "$D" "$T -- p5_binding" \
        'if self.store.read_state(&name).map_err(|e| e.to_string())?.is_some() {' \
        'if false {'
    run M11-claim-op-binds "$D" "$T -- p5_binding" \
        'if session == CLAIM_OP_SESSION || NOT_MAIL.contains(&conv) {' \
        'if NOT_MAIL.contains(&conv) {'
    run M12-delivery-off-reads-not-main "$A" "$T -- e161_delivery_off" \
        'Ok(Pulled::Nothing { why: "none-pending" }) => {}' \
        'Ok(Pulled::Nothing { .. }) => {}'
    [ "$rc" = 0 ] && echo "ALL MUTANTS KILLED"
    exit "$rc"
fi

out="$(mktemp)"
trap 'rm -f "$out" "$out".*' EXIT
cargo test -p termlink-sidecar --test p5_withhold >"$out.p5" 2>&1
cargo test -p termlink-sidecar --lib lease >"$out.lease" 2>&1
cargo test -p termlink-sidecar --lib mail_waiting >"$out.mw" 2>&1
cargo test -p termlink-sidecar --test deliver_tests >"$out.deliver" 2>&1
cargo test -p termlink-sidecar --test hooks_tests >"$out.hooks" 2>&1
for f in p5 lease mw deliver hooks; do
    grep -q "^test result:" "$out.$f" || { echo "TOOLING: the $f tests did not run"; tail -20 "$out.$f"; exit 2; }
done
cat "$out.p5" "$out.lease" "$out.mw" "$out.deliver" "$out.hooks" >"$out"
rc=0
check() {
    local label="$1"; shift
    local ok=1 t
    for t in "$@"; do
        if ! grep -q "^test \(lease::tests::\|mail_waiting::tests::\)\?${t} \.\.\. ok" "$out"; then
            echo "  missing or failed: $t"; ok=0
        fi
    done
    if [ "$ok" = 1 ]; then echo "PASS $label"; else echo "FAIL $label"; rc=1; fi
}
echo "== P-5 (T-3695): a non-main session is withheld role mail only"
check "E-161 (B-25 withhold change: S2 gets the exact-instance message only, the line once; main keeps the role mail)" \
    e161_non_main_session_gets_exact_instance_mail_only_and_the_line_once
check "E-161 (delivery switch off: the non-main pull refuses delivery-off and hands out nothing)" \
    e161_delivery_off_refuses_the_non_main_pull_too
check "AC-1 (mail to an exact instance reaches that session whether or not it is main)" \
    e161_non_main_session_gets_exact_instance_mail_only_and_the_line_once p5_exact_instance_mail_reaches_its_session_when_it_is_main_too \
    p5_a_send_from_a_session_binds_its_conversation p5_binding_first_wins_and_a_hand_over_binds
check "AC-2 (role mail to a non-main session is still withheld; another session's mail too)" \
    e161_non_main_session_gets_exact_instance_mail_only_and_the_line_once p5_mail_bound_to_another_session_is_not_handed_to_a_non_main_session
check "Rollback (withhold-all flag restores the built all-mail withhold in one step; held mail is counted waiting)" \
    p5_withhold_all_flag_restores_the_built_pull_and_counts_the_held_mail p5_withhold_all_flag_anything_present_counts_as_set
check "NID (built lease, waiting-mail, deliver and hook tests unchanged)" \
    withholds_only_from_other_sessions_of_this_agent_and_tells_once_per_generation unknown_stale_or_vacant_lease_never_withholds \
    e28_exact_instance_mail_never_counts p5_main_and_unknown_lease_pull_as_built
echo "DEFERRED E-161 unreadable kill switch (B-16 part): delivery_enabled already reads any error but NotFound as off (policy.rs); an unreadable flag cannot be staged as root, so the B-16 unit owns its live case"
echo "DEFERRED E-28 non-main line at a human prompt: the sidecar still sees no prompt on its withhold path (pull runs at tool calls); AEF's prompt hook (P-44/P-45)"
[ "$rc" = 0 ] && echo "ALL IN-SCOPE PASS"
exit "$rc"
