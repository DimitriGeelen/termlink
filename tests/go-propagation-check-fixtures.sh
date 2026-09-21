#!/usr/bin/env bash
# tests/go-propagation-check-fixtures.sh (T-3038)
#
# Fixtures for scripts/check-go-propagation.sh — hermetic. No live binary, no real
# .tasks tree, no network. Every run passes BOTH --tasks-dir and --allowlist so the
# suite can never read the repo's own corpus or its real ledger: a fixture that
# silently falls back to production state stops testing the code and starts testing
# the host (PL-213).
#
# Weighted toward the FIRING cases and the FALSE-POSITIVE guards, per the sibling
# suites. Two cases exist because the defects they pin were found by RUNNING the
# check against the real corpus, not by reading it:
#
#   case 7  — a quoted frontmatter timestamp ('2026-09-21T...') made fromisoformat
#             raise, age read as None, and the record reached the firing branch by
#             FALLING THROUGH rather than by being old. A task decided today fired.
#             The bug is invisible unless a fixture supplies a quoted stamp.
#   case 8  — NO-GO and DEFER must never read as GO. The verdict scanner walks
#             several patterns and returns on first hit; an ordering or prefix slip
#             turns a refusal into an approval, which would fire on inceptions that
#             were correctly declined and had no follow-on work BY DESIGN.
set -uo pipefail

SCRIPT="${SCRIPT:-scripts/check-go-propagation.sh}"
[ -f "$SCRIPT" ] || { echo "fixtures: $SCRIPT not found"; exit 2; }

PASS=0
FAIL=0
TMPROOT="$(mktemp -d)"
trap 'rm -rf "$TMPROOT"' EXIT

ok() { PASS=$((PASS+1)); echo "  ok   — $1"; }
no() { FAIL=$((FAIL+1)); echo "  FAIL — $1"; }

assert_rc() { if [ "$1" = "$2" ]; then ok "$3 (rc=$2)"; else no "$3 (expected rc=$1, got rc=$2)"; fi; }
assert_contains() { case "$1" in *"$2"*) ok "$3" ;; *) no "$3 (missing: $2)" ;; esac; }
assert_not_contains() { case "$1" in *"$2"*) no "$3 (unexpectedly present: $2)" ;; *) ok "$3" ;; esac; }

TODAY="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
OLD="2026-01-05T09:00:00Z"

# mkinception <dir> <id> <decided-stamp> <verdict-line> [related_tasks-value]
# Writes a workflow_type: inception task carrying a decision in its body.
mkinception() {
    local dir="$1" id="$2" stamp="$3" vline="$4" rel="${5:-[]}"
    mkdir -p "$dir/completed"
    {
        echo '---'
        echo "id: $id"
        echo "name: \"fixture inception $id\""
        echo "status: work-completed"
        echo "workflow_type: inception"
        echo "owner: agent"
        echo "related_tasks: $rel"
        echo "date_finished: $stamp"
        echo '---'
        echo
        echo '## Context'
        echo 'fixture body'
        echo
        echo "$vline"
    } > "$dir/completed/${id}-fixture.md"
}

# mkplain <dir> <id> [related_tasks-value] [extra-body]
# A non-inception task. Used to supply back-references and mentions.
mkplain() {
    local dir="$1" id="$2" rel="${3:-[]}" extra="${4:-}"
    mkdir -p "$dir/active"
    {
        echo '---'
        echo "id: $id"
        echo "name: \"fixture build $id\""
        echo "status: started-work"
        echo "workflow_type: build"
        echo "owner: agent"
        echo "related_tasks: $rel"
        echo '---'
        echo
        echo '## Context'
        echo "$extra"
    } > "$dir/active/${id}-fixture.md"
}

mkledger() { printf '# fixture ledger\n%s\n' "$2" > "$1"; }

# run <tasks-dir> <allowlist> [extra args...]
run() { bash "$SCRIPT" --no-heartbeat --tasks-dir "$1" --allowlist "$2" "${@:3}" 2>&1; }

echo "== case 1: an old, strictly-unlinked GO inception FIRES =="
C="$TMPROOT/c1"; L="$TMPROOT/c1.ledger"
mkinception "$C" T-9001 "$OLD" '**Recommendation:** GO'
mkledger "$L" ""
out="$(run "$C" "$L")"; rc=$?
assert_rc 1 "$rc" "unlinked GO fires"
assert_contains "$out" "FIRING" "names the firing state"
assert_contains "$out" "T-9001" "names the leaking inception"

echo "== case 2: the SAME inception acknowledged in the ledger does NOT fire =="
mkledger "$L" "T-9001  # baselined by T-3038"
out="$(run "$C" "$L")"; rc=$?
assert_rc 0 "$rc" "ledger entry suppresses the firing"
assert_contains "$out" "1 acknowledged" "acknowledged entry is COUNTED, not silently dropped (T-2483)"
assert_not_contains "$out" "FIRING" "no firing line while acknowledged"

echo "== case 3: removing the ledger line RE-FIRES it (load-bearing, both directions) =="
mkledger "$L" ""
out="$(run "$C" "$L")"; rc=$?
assert_rc 1 "$rc" "removal restores the firing"
assert_contains "$out" "T-9001" "re-fires on the same inception"

echo "== case 4: FALSE-POSITIVE GUARD — a GO with related_tasks never fires =="
C="$TMPROOT/c4"; L="$TMPROOT/c4.ledger"
mkinception "$C" T-9002 "$OLD" '**Recommendation:** GO' '[T-9100, T-9101]'
mkledger "$L" ""
out="$(run "$C" "$L")"; rc=$?
assert_rc 0 "$rc" "forward-linked GO is clean"
assert_contains "$out" "1 linked" "counted as linked"
assert_not_contains "$out" "FIRING" "properly linked GO must never fire"

echo "== case 5: FALSE-POSITIVE GUARD — a BACK-reference alone counts as linked =="
C="$TMPROOT/c5"; L="$TMPROOT/c5.ledger"
mkinception "$C" T-9003 "$OLD" '**Recommendation:** GO'
mkplain "$C" T-9200 '[T-9003]'
mkledger "$L" ""
out="$(run "$C" "$L")"; rc=$?
assert_rc 0 "$rc" "back-referenced GO is clean"
assert_contains "$out" "1 linked" "back-reference satisfies the strict predicate"

echo "== case 6: strict-vs-loose split — unlinked AND unmentioned is flagged ORPHAN =="
C="$TMPROOT/c6"; L="$TMPROOT/c6.ledger"
mkinception "$C" T-9004 "$OLD" '**Recommendation:** GO'
mkinception "$C" T-9005 "$OLD" '**Recommendation:** GO'
mkplain "$C" T-9201 '[]' 'prose mentioning T-9005 without linking it'
mkledger "$L" ""
out="$(run "$C" "$L")"; rc=$?
assert_rc 1 "$rc" "both strictly-unlinked GOs fire"
assert_contains "$out" "ORPHAN" "the unmentioned one carries the orphan marker"
orphan_line="$(printf '%s\n' "$out" | grep 'T-9004' || true)"
assert_contains "$orphan_line" "ORPHAN" "T-9004 (nobody names it) is the orphan"
mentioned_line="$(printf '%s\n' "$out" | grep 'T-9005' || true)"
assert_not_contains "$mentioned_line" "ORPHAN" "T-9005 is merely mentioned — metadata gap, not orphan"

echo "== case 7: REGRESSION — a QUOTED timestamp decided today lands in grace, not firing =="
C="$TMPROOT/c7"; L="$TMPROOT/c7.ledger"
mkinception "$C" T-9006 "'$TODAY'" '**Recommendation:** GO'
mkledger "$L" ""
out="$(run "$C" "$L")"; rc=$?
assert_rc 0 "$rc" "quoted stamp parses — today's GO is in grace, not firing"
assert_contains "$out" "1 in grace" "counted in the grace bucket"
assert_not_contains "$out" "age unknown" "the quote must not defeat the date parse"

echo "== case 8: REGRESSION — NO-GO and DEFER must never read as GO =="
C="$TMPROOT/c8"; L="$TMPROOT/c8.ledger"
mkinception "$C" T-9007 "$OLD" '**Recommendation:** NO-GO — not worth building'
mkinception "$C" T-9008 "$OLD" '**Recommendation:** DEFER until Q3'
mkledger "$L" ""
out="$(run "$C" "$L")"; rc=$?
assert_rc 0 "$rc" "declined inceptions are not GO leaks"
assert_contains "$out" "0 GO inception" "neither counts as a GO"
assert_not_contains "$out" "T-9007" "NO-GO must never fire"
assert_not_contains "$out" "T-9008" "DEFER must never fire"

echo "== case 9: grace window is honoured and --grace-days tunes it =="
C="$TMPROOT/c9"; L="$TMPROOT/c9.ledger"
mkinception "$C" T-9009 "$TODAY" '**Recommendation:** GO'
mkledger "$L" ""
out="$(run "$C" "$L")"; rc=$?
assert_rc 0 "$rc" "fresh GO is in grace by default"
out="$(run "$C" "$L" --grace-days 0)"; rc=$?
assert_rc 1 "$rc" "--grace-days 0 collapses the window and fires"

echo "== case 10: an undated GO fires by default (never silently excused) =="
C="$TMPROOT/c10"; L="$TMPROOT/c10.ledger"
mkinception "$C" T-9010 "" '**Recommendation:** GO'
mkledger "$L" ""
out="$(run "$C" "$L")"; rc=$?
assert_rc 1 "$rc" "unknown age fires rather than passing"
assert_contains "$out" "age unknown" "states why it fired"

echo "== case 11: scope disclaimer + census print on BOTH paths (T-2680) =="
assert_contains "$out" "Scope:" "firing path carries the scope disclaimer"
assert_contains "$out" "does NOT audit" "firing path disclaims what it does not cover"
C="$TMPROOT/c11"; L="$TMPROOT/c11.ledger"
mkinception "$C" T-9011 "$OLD" '**Recommendation:** GO' '[T-9300]'
mkledger "$L" ""
clean="$(run "$C" "$L")"; rc=$?
assert_rc 0 "$rc" "clean path exits 0"
assert_contains "$clean" "Scope:" "clean path ALSO carries the disclaimer — a green must not over-claim"
assert_contains "$clean" "GO-propagation:" "clean path states the census"

echo "== case 12: non-inception tasks are never examined =="
C="$TMPROOT/c12"; L="$TMPROOT/c12.ledger"
mkplain "$C" T-9012 '[]' 'body saying **Recommendation:** GO in a build task'
mkledger "$L" ""
out="$(run "$C" "$L")"; rc=$?
assert_rc 0 "$rc" "a build task carrying GO prose is not an inception"
assert_contains "$out" "0 GO inception" "workflow_type gates the corpus"

echo "== case 13: FAIL-CLOSED — tooling faults exit 2, never a vacuous clean =="
out="$(bash "$SCRIPT" --tasks-dir "$TMPROOT/does-not-exist" --allowlist "$TMPROOT/c1.ledger" 2>&1)"; rc=$?
assert_rc 2 "$rc" "missing tasks dir is a tooling error"
assert_contains "$out" "fail-closed" "says so explicitly"

EMPTY="$TMPROOT/empty"; mkdir -p "$EMPTY/active"
out="$(run "$EMPTY" "$TMPROOT/c1.ledger")"; rc=$?
assert_rc 2 "$rc" "a corpus of zero task files is never a clean census"
assert_contains "$out" "never a clean census" "names the reason"

BADL="$TMPROOT/ledger-as-dir"; mkdir -p "$BADL"
out="$(run "$TMPROOT/c1" "$BADL")"; rc=$?
assert_rc 2 "$rc" "an unreadable ledger is a tooling error, not an empty ledger"
assert_contains "$out" "fail-closed" "unreadable ledger fails closed"

out="$(run "$TMPROOT/c1" "$TMPROOT/c1.ledger" --grace-days notanumber)"; rc=$?
assert_rc 2 "$rc" "non-integer --grace-days is refused"

out="$(bash "$SCRIPT" --nonsense-flag 2>&1)"; rc=$?
assert_rc 2 "$rc" "unknown argument is refused rather than ignored"

echo "== case 14: --json envelope carries census, scope and the firing set =="
C="$TMPROOT/c14"; L="$TMPROOT/c14.ledger"
mkinception "$C" T-9014 "$OLD" '**Recommendation:** GO'
mkledger "$L" ""
out="$(run "$C" "$L" --json)"; rc=$?
assert_rc 1 "$rc" "json mode preserves the exit contract"
python3 - "$out" <<'PY' && ok "json envelope has ok/census/scope/firing with the leaking id" || no "json envelope malformed"
import json,sys
d=json.loads(sys.argv[1])
assert d["ok"] is False
assert d["census"]["go_inceptions"] == 1
assert d["census"]["firing"] == 1
assert "does NOT audit" in d["scope"]
assert d["firing"][0]["id"] == "T-9014"
PY

echo "== case 15: --quiet stays silent when clean, speaks when firing =="
C="$TMPROOT/c15"; L="$TMPROOT/c15.ledger"
mkinception "$C" T-9015 "$OLD" '**Recommendation:** GO' '[T-9400]'
mkledger "$L" ""
out="$(run "$C" "$L" --quiet)"; rc=$?
assert_rc 0 "$rc" "quiet clean exits 0"
if [ -z "$out" ]; then ok "quiet clean prints nothing"; else no "quiet clean printed: $out"; fi
mkinception "$C" T-9016 "$OLD" '**Recommendation:** GO'
out="$(run "$C" "$L" --quiet)"; rc=$?
assert_rc 1 "$rc" "quiet firing still exits 1"
assert_contains "$out" "T-9016" "quiet mode still names a real leak"

echo
echo "=================================================="
echo "  go-propagation fixtures: $PASS passed, $FAIL failed"
echo "=================================================="
[ "$FAIL" -eq 0 ] || exit 1
