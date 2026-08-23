#!/usr/bin/env bash
# tests/reap-topic-fixtures.sh — T-2754 fixture suite for scripts/lib/reap-topic.sh
#
# Hermetic: no live hub, no real termlink. A stub `termlink` on PATH stands in for
# the binary, so every branch of the helper's contract is exercised deterministically.
#
# The contract under test (see the helper's header):
#   - ALWAYS returns 0 — a prover's verdict is never about whether cleanup worked.
#   - Skips with a named warning on a binary predating T-2421 (`channel delete` absent).
#   - Honours TERMLINK_KEEP_TEST_TOPICS=1.
#   - Treats an empty topic name as a silent no-op (traps fire before assignment).
#   - Preserves the caller's exit code when wired into a trap.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
HELPER="$REPO/scripts/lib/reap-topic.sh"

PASS=0
FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

test -f "$HELPER" || { echo "FATAL: helper not found at $HELPER"; exit 2; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# ---------------------------------------------------------------------------
# Stub builders. Each writes a fake `termlink` into its own bin dir.
# ---------------------------------------------------------------------------

# A binary that supports `channel delete` and succeeds. Records each delete.
make_stub_ok() {
    local d="$1"; mkdir -p "$d"
    cat > "$d/termlink" <<'STUB'
#!/usr/bin/env bash
if [ "${1:-}" = "channel" ] && [ "${2:-}" = "delete" ]; then
    if [ "${3:-}" = "--help" ]; then echo "usage: channel delete <topic> --yes"; exit 0; fi
    echo "$3" >> "$REAP_TEST_LOG"
    echo "Deleted topic '$3'"
    exit 0
fi
exit 0
STUB
    chmod +x "$d/termlink"
}

# A binary that supports `channel delete` but FAILS every delete.
make_stub_delete_fails() {
    local d="$1"; mkdir -p "$d"
    cat > "$d/termlink" <<'STUB'
#!/usr/bin/env bash
if [ "${1:-}" = "channel" ] && [ "${2:-}" = "delete" ]; then
    if [ "${3:-}" = "--help" ]; then echo "usage"; exit 0; fi
    echo "error: topic not found" >&2
    exit 1
fi
exit 0
STUB
    chmod +x "$d/termlink"
}

# A binary PREDATING T-2421 — `channel delete` is not a known subcommand.
make_stub_no_delete_verb() {
    local d="$1"; mkdir -p "$d"
    cat > "$d/termlink" <<'STUB'
#!/usr/bin/env bash
if [ "${1:-}" = "channel" ] && [ "${2:-}" = "delete" ]; then
    echo "error: unrecognized subcommand 'delete'" >&2
    exit 2
fi
exit 0
STUB
    chmod +x "$d/termlink"
}

# Run a snippet in a subshell with the helper sourced and a stub on PATH.
# Usage: run_with <bindir> <log> <snippet...>   → prints rc on stdout line 1
run_with() {
    local bindir="$1" log="$2"; shift 2
    (
        export PATH="$bindir:$PATH"
        export REAP_TEST_LOG="$log"
        # shellcheck source=/dev/null
        . "$HELPER"
        eval "$@"
    )
}

echo "T-2754 reap-topic fixtures"
echo ""

# --- 1: happy path deletes the topic and returns 0 -------------------------
echo "1: successful delete"
B1="$WORK/b1"; L1="$WORK/l1"; : > "$L1"
make_stub_ok "$B1"
out="$(run_with "$B1" "$L1" 'reap_topic "topic-alpha"; echo "rc=$?"' 2>/dev/null)"
if echo "$out" | grep -q "rc=0"; then pass "returns 0"; else fail "expected rc=0, got: $out"; fi
if grep -q "^topic-alpha$" "$L1"; then pass "delete actually invoked with the topic name"; else fail "stub never saw topic-alpha"; fi

# --- 2: a FAILING delete still returns 0 (best-effort contract) ------------
echo "2: failing delete is non-fatal"
B2="$WORK/b2"; L2="$WORK/l2"; : > "$L2"
make_stub_delete_fails "$B2"
out="$(run_with "$B2" "$L2" 'reap_topic "topic-beta"; echo "rc=$?"' 2>/dev/null)"
if echo "$out" | grep -q "rc=0"; then pass "failing delete still returns 0"; else fail "expected rc=0, got: $out"; fi
err="$(run_with "$B2" "$L2" 'reap_topic "topic-beta"' 2>&1 >/dev/null)"
if echo "$err" | grep -q "topic-beta"; then pass "names the leaked topic on stderr"; else fail "warning did not name the topic: $err"; fi
if echo "$err" | grep -q "leaked"; then pass "warning says 'leaked'"; else fail "warning missing 'leaked': $err"; fi

# --- 3: binary predating T-2421 skips with a named warning ----------------
echo "3: no 'channel delete' verb (pre-T-2421 binary)"
B3="$WORK/b3"; L3="$WORK/l3"; : > "$L3"
make_stub_no_delete_verb "$B3"
out="$(run_with "$B3" "$L3" 'reap_topic "topic-gamma"; echo "rc=$?"' 2>/dev/null)"
if echo "$out" | grep -q "rc=0"; then pass "returns 0 on missing verb"; else fail "expected rc=0, got: $out"; fi
err="$(run_with "$B3" "$L3" 'reap_topic "topic-gamma"' 2>&1 >/dev/null)"
if echo "$err" | grep -q "T-2421"; then pass "warning cites the T-2421 requirement"; else fail "warning missing T-2421: $err"; fi

# --- 4: opt-out is honoured ------------------------------------------------
echo "4: TERMLINK_KEEP_TEST_TOPICS=1 opt-out"
B4="$WORK/b4"; L4="$WORK/l4"; : > "$L4"
make_stub_ok "$B4"
out="$(run_with "$B4" "$L4" 'TERMLINK_KEEP_TEST_TOPICS=1 reap_topic "topic-delta"; echo "rc=$?"' 2>/dev/null)"
if echo "$out" | grep -q "rc=0"; then pass "returns 0 under opt-out"; else fail "expected rc=0, got: $out"; fi
if [ ! -s "$L4" ]; then pass "no delete attempted under opt-out"; else fail "opt-out still deleted: $(cat "$L4")"; fi
err="$(run_with "$B4" "$L4" 'TERMLINK_KEEP_TEST_TOPICS=1 reap_topic "topic-delta"' 2>&1 >/dev/null)"
if echo "$err" | grep -q "retaining"; then pass "opt-out says it is retaining"; else fail "opt-out silent: $err"; fi

# --- 5: control — WITHOUT opt-out the same call DOES delete ---------------
# PL-219: an assertion that cannot fail is not an assertion. Fixture 4 only
# means something if the identical call deletes when the opt-out is absent.
echo "5: control — opt-out is what suppressed the delete"
B5="$WORK/b5"; L5="$WORK/l5"; : > "$L5"
make_stub_ok "$B5"
run_with "$B5" "$L5" 'reap_topic "topic-delta"' >/dev/null 2>&1
if grep -q "^topic-delta$" "$L5"; then pass "same call deletes when opt-out absent"; else fail "control failed — delete never happened"; fi

# --- 6: empty topic name is a silent no-op --------------------------------
echo "6: empty topic name"
B6="$WORK/b6"; L6="$WORK/l6"; : > "$L6"
make_stub_ok "$B6"
out="$(run_with "$B6" "$L6" 'reap_topic ""; echo "rc=$?"' 2>/dev/null)"
if echo "$out" | grep -q "rc=0"; then pass "returns 0 on empty name"; else fail "expected rc=0, got: $out"; fi
if [ ! -s "$L6" ]; then pass "no delete attempted for empty name"; else fail "deleted something for empty name"; fi
err="$(run_with "$B6" "$L6" 'reap_topic ""' 2>&1 >/dev/null)"
if [ -z "$err" ]; then pass "empty name is SILENT (traps fire pre-assignment)"; else fail "empty name warned: $err"; fi

# --- 7: exit-code preservation through a trap -----------------------------
# The reason the helper returning 0 is not sufficient on its own.
echo "7: trap preserves the caller's exit code"
B7="$WORK/b7"; L7="$WORK/l7"; : > "$L7"
make_stub_ok "$B7"
cat > "$WORK/caller.sh" <<CALLER
#!/usr/bin/env bash
set -u
export PATH="$B7:\$PATH"
export REAP_TEST_LOG="$L7"
. "$HELPER"
topic="topic-epsilon"
trap 'rc=\$?; reap_topic "\${topic:-}"; exit \$rc' EXIT
exit 7
CALLER
chmod +x "$WORK/caller.sh"
bash "$WORK/caller.sh" >/dev/null 2>&1
rc=$?
if [ "$rc" -eq 7 ]; then pass "caller's exit 7 survives the trap"; else fail "expected 7, got $rc"; fi
if grep -q "^topic-epsilon$" "$L7"; then pass "topic still reaped on the failing path"; else fail "no reap on failing path"; fi

# --- 8: missing binary is non-fatal ---------------------------------------
echo "8: termlink absent from PATH"
out="$(
    (
        export PATH="$WORK/definitely-empty-bin"
        # shellcheck source=/dev/null
        . "$HELPER"
        reap_topic "topic-zeta"; echo "rc=$?"
    ) 2>/dev/null
)"
if echo "$out" | grep -q "rc=0"; then pass "returns 0 when termlink is absent"; else fail "expected rc=0, got: $out"; fi

# --- 9: double-sourcing is safe -------------------------------------------
echo "9: double-source guard"
B9="$WORK/b9"; L9="$WORK/l9"; : > "$L9"
make_stub_ok "$B9"
out="$(run_with "$B9" "$L9" '. "'"$HELPER"'"; reap_topic "topic-eta"; echo "rc=$?"' 2>/dev/null)"
if echo "$out" | grep -q "rc=0"; then pass "re-sourcing does not break the helper"; else fail "double-source broke it: $out"; fi

echo ""
echo "Results: $PASS pass / $FAIL fail"
[ "$FAIL" -eq 0 ]
