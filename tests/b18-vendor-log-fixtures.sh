#!/usr/bin/env bash
# Fixture suite for B-18 log half (arc-011 plan unit P-26, T-3710): the vendor-channel log hook
# K-20 (AL-39). Prints one PASS line per E-n of the plan row (E-71, E-140, E-170) or FAIL, and a
# DEFERRED line, with its reason, for each part this unit cannot serve in this repository state.
# Exit 0 only when every E-n in scope passed; 1 on a failure; 2 when the tests could not run.
#
#   bash tests/b18-vendor-log-fixtures.sh            # the E-n
#   bash tests/b18-vendor-log-fixtures.sh --mutants  # FP 3.1.b: every mutant must be KILLED
#
# The Rust tests run on a clean export of HEAD (git archive, never a worktree) under
# $B18_DIR (default /var/tmp/b18-vendor-log), so other workers' uncommitted edits cannot break
# or fake the run. TERMLINK_BIN is not needed: these tests start no hub.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
REPO="$(pwd)"
export TMPDIR="${TMPDIR:-/var/tmp}" CARGO_TARGET_TMPDIR="${CARGO_TARGET_TMPDIR:-/var/tmp}"
root="${B18_DIR:-/var/tmp/b18-vendor-log}"
export CARGO_TARGET_DIR="$root/target"
mkdir -p "$root" || exit 2
HOOK=scripts/hooks/vendor-channel-log.sh
RUST="cargo test -p termlink-sidecar --test vendor_log_tests"

export_head() { # export_head <dir>
    rm -rf "$1" && mkdir -p "$1" || return 2
    git -C "$REPO" archive HEAD | tar -x -C "$1"
}

# settings_ok <settings.json>: one PreToolUse entry, matcher exactly SendMessage|Agent|Task, runs the hook script.
settings_ok() {
    F="$1" python3 -I -c '
import json,os,sys
s=json.load(open(os.environ["F"]))
hits=[e for e in s.get("hooks",{}).get("PreToolUse",[]) for h in e.get("hooks",[]) if h.get("command","").endswith("scripts/hooks/vendor-channel-log.sh")]
sys.exit(0 if len(hits)==1 and hits[0].get("matcher")=="SendMessage|Agent|Task" else 1)'
}

# hook_script_ok <script>: never blocks (exit 0, no stdout) and passes the input through.
hook_script_ok() {
    local s="$1" d rc out ok=1
    d="$(mktemp -d)" || return 2
    printf '#!/bin/sh\necho "$*" > "%s/args"; cat > "%s/stdin"; exit 2\n' "$d" "$d" > "$d/tl"; chmod +x "$d/tl"
    # no socket: nothing runs
    out=$(env -u TERMLINK_SIDECAR_SOCKET TERMLINK_VENDOR_HOOK_BIN="$d/tl" bash "$s" < /dev/null); rc=$?
    { [ "$rc" = 0 ] && [ -z "$out" ] && [ ! -e "$d/args" ]; } || { echo "  no-socket case: rc=$rc"; ok=0; }
    # a binary that fails (older termlink, exit 2): still exit 0, prints nothing, got the input
    out=$(echo '{"session_id":"s","tool_name":"Agent"}' | TERMLINK_SIDECAR_SOCKET=/nonexistent TERMLINK_VENDOR_HOOK_BIN="$d/tl" bash "$s"); rc=$?
    { [ "$rc" = 0 ] && [ -z "$out" ] && grep -q "^sidecar hook vendor-channel$" "$d/args" && grep -q '"tool_name":"Agent"' "$d/stdin"; } \
        || { echo "  failing-binary case: rc=$rc out=$out"; ok=0; }
    # termlink not installed: exit 0
    out=$(echo '{}' | TERMLINK_SIDECAR_SOCKET=/nonexistent TERMLINK_VENDOR_HOOK_BIN="$d/absent" bash "$s"); rc=$?
    { [ "$rc" = 0 ] && [ -z "$out" ]; } || { echo "  missing-binary case: rc=$rc"; ok=0; }
    rm -rf "$d"
    [ "$ok" = 1 ]
}

if [ "${1:-}" = "--mutants" ]; then
    src="$root/mut-src"
    rc=0
    # rust_mutant <id> <file> <test-filter> <python-literal-old> <python-literal-new>
    rust_mutant() {
        local id="$1" file="$2" filt="$3" old="$4" new="$5"
        export_head "$src" || { echo "TOOLING $id: export failed"; return 2; }
        if ! OLD="$old" NEW="$new" F="$src/$file" python3 -I -c '
import os,sys
p=os.environ["F"]; s=open(p).read(); o=os.environ["OLD"]
if s.count(o)!=1: sys.exit(3)
open(p,"w").write(s.replace(o,os.environ["NEW"],1))'; then
            echo "TOOLING $id: the mutation no longer applies to $file"; return 2
        fi
        local log="$root/$id.log"
        ( cd "$src" && env -u PROJECT_ROOT $RUST -- "$filt" ) >"$log" 2>&1
        local trc=$?
        if grep -q "^error\[E\|^error: could not compile" "$log"; then
            echo "TOOLING $id: the mutant does not compile ($log)"
            grep -m1 -A6 "^error" "$log" | sed 's/^/    /'; return 2
        fi
        if ! grep -q "^test result:" "$log" && ! grep -q "panicked" "$log"; then
            echo "TOOLING $id: the tests did not run ($log)"; return 2
        fi
        if [ "$trc" -ne 0 ] && grep -q "FAILED\|panicked" "$log"; then
            echo "KILLED $id"; return 0
        fi
        echo "SURVIVED $id ($log)"; return 1
    }
    run() { "$@"; local r=$?; [ $r -gt $rc ] && rc=$r; return 0; }
    A=crates/termlink-sidecar/src/api.rs
    H=crates/termlink-sidecar/src/hook.rs
    S=crates/termlink-sidecar/src/store.rs
    # E-71.5 control: a hook that knows SendMessage only misses the sub-agent rows.
    run rust_mutant M1-sendmessage-only "$H" e71_ \
        'pub const VENDOR_TOOLS: [&str; 3] = ["SendMessage", "Agent", "Task"];' \
        'pub const VENDOR_TOOLS: [&str; 3] = ["SendMessage", "SendMessage", "SendMessage"];'
    run rust_mutant M2-coalesce-across-tools "$A" e71_each_tool \
        'let key = format!("{session}:{day}:{tool}");' \
        'let key = format!("{session}:{day}");'
    run rust_mutant M3-no-coalescing "$S" e71_ \
        '        if seen {
            return Ok(false);' \
        '        if false && seen {
            return Ok(false);'
    run rust_mutant M4-row-failure-answered-ok "$A" e170_row \
        '        Err(e) => return err("vendor-log", &format!("cannot write the vendor-channel row: {e}"), "FCW-146: the use is in the hook error log; status shows vendor_log: degraded"),' \
        '        Err(_) => 0,'
    # E-170.5 control (vendor half): a status that never degrades passes a check it must fail.
    run rust_mutant M5-status-never-degrades "$A" e170_ \
        '    } else if errors > 0 && last_error' \
        '    } else if false && errors > 0 && last_error'
    run rust_mutant M6-unreadable-error-log-read-as-ok "$A" e170_control \
        '        Some(Err(_)) => (0, None, false),' \
        '        Some(Err(_)) => (0, None, true),'
    run rust_mutant M7-no-row-only-the-entry "$A" e71_coalesce \
        'let seq = match store.append_event_unless(ADMIN_CONV, VENDOR_USE_EVENT, &id, session, &digest, extra, |_| false) {' \
        'let seq = match store.append_event_unless(ADMIN_CONV, VENDOR_USE_EVENT, &id, session, &digest, extra, |_| true) {'
    # Shell mutants: the settings entry and the never-blocking wrapper.
    t="$(mktemp -d)" || exit 2
    sed 's/"SendMessage|Agent|Task"/"SendMessage"/' .claude/settings.json > "$t/settings.json"
    if grep -q '"SendMessage"' "$t/settings.json" && ! settings_ok "$t/settings.json"; then echo "KILLED M8-settings-matcher-sendmessage-only"
    else echo "SURVIVED M8-settings-matcher-sendmessage-only"; rc=1; fi
    sed 's|^"\$bin" sidecar hook vendor-channel >/dev/null 2>&1 \|\| true$|exec "$bin" sidecar hook vendor-channel|' "$HOOK" > "$t/hook.sh"
    if cmp -s "$HOOK" "$t/hook.sh"; then echo "TOOLING M9: the mutation no longer applies to $HOOK"; [ $rc -lt 2 ] && rc=2
    elif ! hook_script_ok "$t/hook.sh" >/dev/null; then echo "KILLED M9-hook-passes-a-failing-exit-through"
    else echo "SURVIVED M9-hook-passes-a-failing-exit-through"; rc=1; fi
    rm -rf "$t"
    [ "$rc" = 0 ] && echo "ALL MUTANTS KILLED"
    exit "$rc"
fi

src="$root/src"
export_head "$src" || { echo "TOOLING: export of HEAD failed"; exit 2; }
out="$root/rust.out"
( cd "$src" && env -u PROJECT_ROOT $RUST ) >"$out" 2>&1
grep -q "^test result:" "$out" || { echo "TOOLING: the Rust tests did not run"; tail -20 "$out"; exit 2; }
rc=0
check() { # check <label> <test-name>...: every named test ran and passed (absent from the run = FAIL)
    local label="$1"; shift
    local ok=1 t
    for t in "$@"; do
        grep -q "^test ${t} \.\.\. ok" "$out" || { echo "  missing or failed: $t"; ok=0; }
    done
    if [ "$ok" = 1 ]; then echo "PASS $label"; else echo "FAIL $label"; rc=1; fi
}
echo "== P-26 (T-3710): B-18 log half, vendor-channel log hook K-20"
check "E-71 (each of SendMessage/Agent/Task twice: six rows, three coalesced entries; the hook exits 0 and prints nothing)" \
    e71_each_tool_twice_gives_six_rows_three_entries_and_never_blocks e71_hook_binary_exits_zero_prints_nothing_and_logs_each_tool \
    e71_a_tool_that_is_not_a_vendor_channel_leaves_nothing e71_coalesce_key_is_session_and_day \
    e71_matcher_names_exactly_the_three_vendor_tools e71_vendor_use_is_a_send_scope_op_with_plain_names
if settings_ok .claude/settings.json; then echo "PASS E-71 settings (one PreToolUse entry, matcher SendMessage|Agent|Task, runs $HOOK)"
else echo "FAIL E-71 settings (PreToolUse entry for $HOOK missing or matcher not SendMessage|Agent|Task)"; rc=1; fi
if hook_script_ok "$HOOK"; then echo "PASS E-71 hook script (never blocks: exit 0 and no output with no socket, a failing or missing termlink)"
else echo "FAIL E-71 hook script"; rc=1; fi
check "E-140 (every use leaves an _admin row naming session, tool and time: the monitor's data)" \
    e71_each_tool_twice_gives_six_rows_three_entries_and_never_blocks e71_coalesce_key_is_session_and_day
check "E-170 vendor half (row unwritable or sidecar unreachable: error-log line, exit 0, status vendor_log: degraded; never ok when it cannot look)" \
    e170_row_that_cannot_be_written_logs_the_use_and_status_degrades e170_unreachable_sidecar_logs_the_use_and_exits_zero \
    e170_control_status_is_ok_only_with_no_newer_error_and_unknown_when_unreadable
echo "DEFERRED E-71 tier 3 (EP-5 live session with the hook installed): needs a release carrying the vendor-channel event installed on the host (G-1/G-2)"
echo "DEFERRED E-140 daily monitor (session tool logs vs rows): plan unit P-33"
echo "DEFERRED E-170 reconciliation half (UNRECONCILED never CLEAR until a pass): plan unit P-20 (B-10)"
[ "$rc" = 0 ] && echo "ALL IN-SCOPE PASS"
exit "$rc"
