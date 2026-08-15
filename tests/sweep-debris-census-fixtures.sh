#!/usr/bin/env bash
# tests/sweep-debris-census-fixtures.sh — T-2756 fixtures for the sweep census.
#
# Hermetic: a stub `termlink` on PATH serves a canned `channel list --json`, so the
# classifier is driven over a known topic set with no live hub.
#
# What is under test is the TRUTHFULNESS of the report, not deletion policy. Before
# T-2756 the script printed only `debris-candidates=N`; against a hub holding ~630
# test-debris topics that read as "the hub is clean". It was not — the tool
# structurally could not see them (deny guard) or declined to classify them
# (conservative default) and said nothing about either.
#
# Every fixture runs WITHOUT --yes. Nothing here can delete anything.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
SWEEP="$REPO/scripts/sweep-test-debris.sh"

PASS=0
FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

test -f "$SWEEP" || { echo "FATAL: sweeper not found at $SWEEP"; exit 2; }
command -v jq >/dev/null 2>&1 || { echo "SKIP: jq not available"; exit 0; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
BIN="$WORK/bin"; mkdir -p "$BIN"

# Canned topic set, one per class:
#   denied       — channel:learnings (exact), agent-conv-selftest-x (agent-conv-*), dm:a:b
#   candidates   — smoke:demo-1, stress-abc, t-1234-thing
#   unclassified — dummy-99, arc004-dbg-7, agent-presence-t2302-5
cat > "$WORK/topics.json" <<'JSON'
{"topics":[
 {"name":"channel:learnings","count":1,"retention":{"kind":"forever"}},
 {"name":"agent-conv-selftest-x","count":1,"retention":{"kind":"forever"}},
 {"name":"dm:a:b","count":1,"retention":{"kind":"forever"}},
 {"name":"smoke:demo-1","count":1,"retention":{"kind":"forever"}},
 {"name":"stress-abc","count":1,"retention":{"kind":"forever"}},
 {"name":"t-1234-thing","count":1,"retention":{"kind":"forever"}},
 {"name":"dummy-99","count":1,"retention":{"kind":"forever"}},
 {"name":"arc004-dbg-7","count":1,"retention":{"kind":"forever"}},
 {"name":"agent-presence-t2302-5","count":1,"retention":{"kind":"forever"}}
]}
JSON

cat > "$BIN/termlink" <<STUB
#!/usr/bin/env bash
if [ "\${1:-}" = "channel" ] && [ "\${2:-}" = "delete" ] && [ "\${3:-}" = "--help" ]; then
    echo "usage: channel delete <topic> --yes"; exit 0
fi
if [ "\${1:-}" = "channel" ] && [ "\${2:-}" = "list" ]; then
    cat "$WORK/topics.json"; exit 0
fi
if [ "\${1:-}" = "channel" ] && [ "\${2:-}" = "delete" ]; then
    echo "STUB-DELETE \$3" >> "$WORK/deleted.log"; exit 0
fi
exit 0
STUB
chmod +x "$BIN/termlink"

run_sweep() { ( export PATH="$BIN:$PATH"; bash "$SWEEP" "$@" ); }

echo "T-2756 sweep-debris census fixtures"
echo ""

# --- 1: census reports all three classes, and they sum to the total ---------
echo "1: three-way census"
out="$(run_sweep 2>&1)"
if echo "$out" | grep -q "topics=9"; then pass "total=9"; else fail "expected topics=9: $(echo "$out" | head -1)"; fi
if echo "$out" | grep -q "candidates=3"; then pass "candidates=3 (smoke/stress/t-NNNN)"; else fail "expected candidates=3: $(echo "$out" | head -1)"; fi
if echo "$out" | grep -q "denied=3"; then pass "denied=3 (learnings/agent-conv/dm)"; else fail "expected denied=3: $(echo "$out" | head -1)"; fi
if echo "$out" | grep -q "unclassified=3"; then pass "unclassified=3 (dummy/arc004-dbg/agent-presence-t2302)"; else fail "expected unclassified=3: $(echo "$out" | head -1)"; fi

# --- 2: the scope note states the limitation in words ----------------------
echo "2: scope note"
if echo "$out" | grep -qi "allowlist"; then pass "names the allowlist as the subject of the count"; else fail "no scope note"; fi
if echo "$out" | grep -qi "not.*clean\|Neither is a statement"; then pass "explicitly refuses the 'hub is clean' reading"; else fail "scope note does not address the misread"; fi

# --- 3: denied topics never become candidates (the safety property) --------
echo "3: deny guard holds"
listed="$(run_sweep --list-only 2>/dev/null)"
for denied in "channel:learnings" "agent-conv-selftest-x" "dm:a:b"; do
    if echo "$listed" | grep -qx "$denied"; then
        fail "DENIED topic reached the candidate set: $denied"
    else
        pass "denied stays out of candidates: $denied"
    fi
done
# Source-level assertion: deny is evaluated BEFORE allow. A reorder would let an
# allow pattern override a protection, which no output check could detect.
# (A single name matching both a deny AND an allow pattern is not constructible
# with the current pattern sets — every pattern is prefix-anchored and the two
# sets share no prefix — so the ordering is asserted structurally instead.)
if grep -q "if deny_topic" "$SWEEP" && grep -A3 "if deny_topic" "$SWEEP" | grep -q "continue"; then
    pass "deny_topic short-circuits before allow_topic"
else
    fail "deny-before-allow ordering not detectable in source"
fi

# --- 4: --list-only stdout stays a bare, parseable name list ---------------
echo "4: piping contract"
if [ "$(echo "$listed" | grep -c .)" -eq 3 ]; then pass "stdout has exactly 3 lines"; else fail "expected 3 lines, got: $(echo "$listed" | grep -c .)"; fi
if echo "$listed" | grep -qi "census\|unclassified\|denied\|scope"; then
    fail "census text leaked into --list-only stdout"
else
    pass "no census text on stdout"
fi
err="$(run_sweep --list-only 2>&1 >/dev/null)"
if echo "$err" | grep -q "unclassified"; then pass "census still emitted, on stderr"; else fail "census lost entirely under --list-only"; fi

# --- 5: --explain names the unclassified topics ---------------------------
echo "5: --explain"
ex="$(run_sweep --explain 2>&1)"
if echo "$ex" | grep -q "dummy-99"; then pass "lists dummy-99"; else fail "--explain did not list dummy-99"; fi
if echo "$ex" | grep -q "arc004-dbg-7"; then pass "lists arc004-dbg-7"; else fail "--explain did not list arc004-dbg-7"; fi
if echo "$ex" | grep -q "agent-presence-t2302-5"; then pass "lists agent-presence-t2302-5"; else fail "--explain did not list agent-presence-t2302-5"; fi

# --- 6: nothing was deleted by any of the above ---------------------------
echo "6: no deletion without --yes"
if [ ! -f "$WORK/deleted.log" ]; then pass "no delete invoked across every dry-run path"; else fail "something deleted: $(cat "$WORK/deleted.log")"; fi

# --- 7: census prints on the CLEAN path too (T-2680) ----------------------
# The clean-looking result is exactly where a bare count misleads, so the
# disclaimer must survive there. Canned list with zero allowlist matches.
echo "7: census on the nothing-to-sweep path"
cat > "$WORK/topics.json" <<'JSON'
{"topics":[
 {"name":"channel:learnings","count":1,"retention":{"kind":"forever"}},
 {"name":"dummy-1","count":1,"retention":{"kind":"forever"}}
]}
JSON
clean="$(run_sweep 2>&1)"
if echo "$clean" | grep -q "nothing to sweep"; then pass "reports nothing to sweep"; else fail "expected nothing-to-sweep path"; fi
if echo "$clean" | grep -q "unclassified=1"; then pass "census survives on the clean path"; else fail "census dropped when candidates=0 — the exact T-2680 misread"; fi
if echo "$clean" | grep -qi "allowlist"; then pass "scope note survives on the clean path"; else fail "scope note dropped when candidates=0"; fi

echo ""
echo "Results: $PASS pass / $FAIL fail"
[ "$FAIL" -eq 0 ]
