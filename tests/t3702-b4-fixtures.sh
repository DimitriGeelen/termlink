#!/usr/bin/env bash
# Fixture suite for arc-011 plan unit P-14 (T-3702, build item B-4: founding-verb use log at
# session and hub; owner-only sessions; 06a AL-32, 04 6.6.4).
# Runs the hermetic tests that carry each listed E-n and prints one PASS line per E-n (or FAIL).
# Exit 0 only when every listed E-n passed; 1 on a failure; 2 when the tests could not run.
#
#   bash tests/t3702-b4-fixtures.sh            # E-68, E-169 (B-4 part), rollback (the working tree)
#   bash tests/t3702-b4-fixtures.sh mutants    # FP 3.1.b: every mutant must be KILLED
#
# Mutants run on a `git archive HEAD` export under .context/working/t3702/mut (gitignored),
# never on the shared working tree (other workers build crates/ concurrently). Each mutant
# edits ONE spot in the export, runs the test(s) named for it, and must turn them red.
# A mutant whose pattern is not found, or whose export does not compile, is TOOLING (exit 2):
# "could not look" is never reported as killed.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
export TMPDIR="${TMPDIR:-/var/tmp}" CARGO_TARGET_TMPDIR="${CARGO_TARGET_TMPDIR:-/var/tmp}"
# T-3755: never rely on PROJECT_ROOT being unset.
unset PROJECT_ROOT
mode="${1:-fixtures}"
work=.context/working/t3702
mkdir -p "$work" || exit 2

SES_UNIT=(-p termlink-session --lib founding_log::tests)
SES_SRV=(-p termlink-session --lib server::tests::e68_)
HUB_SRV=(-p termlink-hub --lib _hub_)

# run_tests <outfile> <cargo args...>: run, keep output; 2 when the suite did not run at all.
run_tests() {
    local out="$1"; shift
    cargo test "$@" >"$out" 2>&1
    grep -q "^test result:" "$out" || return 2
    return 0
}

if [ "$mode" = fixtures ]; then
    out="$work/fx.out"; : >"$out"
    for args in "SES_UNIT" "SES_SRV" "HUB_SRV"; do
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
    echo "== P-14 (T-3702): B-4 founding-verb use log at session and hub; owner-only sessions"
    check "E-68 (observe/notify refused for inject and exec; every attempt, allowed or refused, has a row with the caller at the session and at the hub; owner-only refuses a second identity; the owner still runs and reads output back)" \
        e68_scope_matrix \
        e68_owner_only \
        e68_gate_writes_a_row_per_attempt \
        e68_founding_verbs_logged_and_owner_only_over_socket \
        e68_hub_logs_refused_founding_verbs_with_caller \
        e68_hub_logs_allowed_founding_verbs
    check "E-169 (B-4 part, FCW-132: a founding verb whose use-log row cannot be written is refused with the reason, at the session and at the hub)" \
        e169_unwritable_log_refuses \
        e169_hub_refuses_founding_verb_when_row_unwritable
    check "Rollback (TERMLINK_OWNER_ONLY_CHECK=0 turns the owner check off; the log stays)" \
        policy_from_env_lookup \
        refusal_names_fcw_and_caller
    [ "$rc" = 0 ] && echo "ALL PASS (P-14)"
    exit "$rc"
fi

if [ "$mode" != mutants ]; then echo "usage: $0 [fixtures|mutants]"; exit 2; fi

# ---- mutants (FP 3.1.b) ----------------------------------------------------
mut="$work/mut"
src="$mut/src"
rm -rf "$src"; mkdir -p "$src" || exit 2
git archive HEAD | tar -x -C "$src" || { echo "TOOLING: cannot export HEAD"; exit 2; }
export CARGO_TARGET_DIR="$PWD/$mut/target"
( cd "$src" && cargo test -p termlink-hub -p termlink-session --lib --no-run >"$PWD/../build.out" 2>&1 ) \
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

F=crates/termlink-session/src/founding_log.rs
A=crates/termlink-hub/src/rpc_audit.rs
S=crates/termlink-hub/src/server.rs

# E-68.5 CONTROL: no hub-side log row fails the row count.
mutant M1-no-hub-row "$A" "            return termlink_session::founding_log::append_row(&p, &row);" "            let _ = (&p, &row); return Ok(());" \
    -p termlink-hub --lib e68_hub_logs
mutant M2-no-session-row "$F" "    if let Err(e) = append_row(&path, &row) {" "    if let Err(e) = Ok::<(), std::io::Error>(()).map(|_| (&path, &row)) {" \
    -p termlink-session --lib founding_log::tests::e68_gate_writes_a_row_per_attempt
mutant M3-owner-only-ignored "$F" "        && caller_from != Some(owner.as_str())" "        && caller_from.is_some() && false" \
    -p termlink-session --lib founding_log::tests::e68_owner_only
mutant M4-scope-not-checked "$F" "    if !granted.satisfies(required) {
        return Decision::RefusedScope" "    if false && !granted.satisfies(required) {
        return Decision::RefusedScope" \
    -p termlink-session --lib founding_log::tests::e68_scope_matrix
mutant M5-session-acts-without-its-row "$F" "        return Some(log_unwritable(id, method, &e));" "        let _ = &e;" \
    -p termlink-session --lib founding_log::tests::e169_unwritable_log_refuses
mutant M6-hub-relays-without-its-row "$S" "            Some(founding_log::log_unwritable(id, &req.method, &e))" "            let _ = (id, e); None" \
    -p termlink-hub --lib e169_hub_refuses_founding_verb
mutant M7-hub-skips-refused-attempts "$S" "    if !founding_log::is_founding_verb(&req.method) {" "    if !founding_log::is_founding_verb(&req.method) || !granted.satisfies(required) {" \
    -p termlink-hub --lib e68_hub_logs_refused
mutant M8-session-row-without-caller "$F" '    let caller = serde_json::json!({"from": from, "uid": peer.0, "pid": peer.1});' '    let caller = serde_json::json!({"from": from, "uid": peer.0});' \
    -p termlink-session --lib server::tests::e68_
mutant M9-rollback-ignored "$F" '            Some("0") | Some("false") | Some("off") | Some("no")' '            Some("never-matches")' \
    -p termlink-session --lib founding_log::tests::policy_from_env_lookup

case "$rc" in
    0) echo "ALL MUTANTS KILLED (P-14)";;
    1) echo "MUTANT SURVIVED (P-14)";;
    *) echo "TOOLING: some mutants could not be judged";;
esac
exit "$rc"
