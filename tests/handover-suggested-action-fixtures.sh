#!/usr/bin/env bash
# T-2882 — fixtures for the handover "Suggested First Action" generator.
#
# The defect: handover.sh ranked started-work candidates by task id compared as a
# STRING and printed the first agent-owned one. That is a constant until the task
# closes — it emitted "Continue T-1166" for 916 handovers, then "Continue T-1457"
# for 68 more, while the session's actual focus was never named.
#
# These fixtures pin the replacement behaviour: focus first, then last_update
# descending, never id order. The final case is the load-bearing one — it runs the
# SAME assertions against the pre-fix script from git and requires them to FAIL. A
# suite that passes both before and after proves nothing (T-2814 lesson).
#
# Usage: bash tests/handover-suggested-action-fixtures.sh
# Exit:  0 = all assertions pass, 1 = a failure, 2 = tooling error (fail-closed).

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HANDOVER_SH="${HANDOVER_SH:-$REPO_ROOT/.agentic-framework/agents/handover/handover.sh}"
# The pre-fix commit, pinned deliberately. This was `HEAD` in the first draft, which
# was correct for exactly as long as the fix was uncommitted: the moment T-2882 landed,
# HEAD became the FIXED script and case 9 compared the fix against itself, reporting
# "the fixtures do not discriminate" — the load-bearing case quietly disarming itself.
# Caught by rehearsing the P-011 line under `set -eo pipefail` rather than by hand.
# 35affce76 is the last commit before the T-2882 fix.
PREFIX_REF="${PREFIX_REF:-35affce76}"

PASS=0
FAIL=0

ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n     expected: %s\n     actual:   %s\n' "$1" "$2" "$3"; }
die()  { printf 'handover-suggested-action-fixtures: %s\n' "$1" >&2; exit 2; }

command -v python3 >/dev/null 2>&1 || die "python3 not found"
[ -f "$HANDOVER_SH" ] || die "handover.sh not found at $HANDOVER_SH"

WORK="$(mktemp -d)" || die "mktemp failed"
trap 'rm -rf "$WORK"' EXIT

# ── Extract the generator block from a handover.sh ───────────────────────────────
# Anchored on markers both the pre-fix and post-fix versions carry, so the same
# extractor works against either and the mutant case is a fair comparison.
extract_block() {
    local script="$1" out="$2"
    awk '
      /^\$\(python3 -c "$/            { start = NR; buf = ""; collecting = 1 }
      collecting                      { buf = buf $0 "\n" }
      /status: started-work/          { if (collecting) seen = 1 }
      /^" 2>\/dev\/null \|\| echo "See active tasks"\)$/ {
          if (collecting && seen) { printf "%s", buf; exit }
          collecting = 0; seen = 0
      }
    ' "$script" > "$out"
    [ -s "$out" ] || return 1
    # Turn the bare command substitution into an assignment we can echo.
    sed -i '1s/^\$(/RESULT=$(/' "$out"
    printf 'printf "%%s\\n" "$RESULT"\n' >> "$out"
}

# ── Build a fixture project root ────────────────────────────────────────────────
# make_task <root> <id> <owner> <horizon> <last_update> <name>
make_task() {
    local root="$1" id="$2" owner="$3" horizon="$4" lu="$5" name="$6"
    mkdir -p "$root/.tasks/active"
    cat > "$root/.tasks/active/$id.md" <<TASKEOF
---
id: $id
name: "$name"
status: started-work
workflow_type: build
horizon: $horizon
owner: $owner
last_update: $lu
---

# $id
TASKEOF
}

set_focus() {
    local root="$1" tid="$2"
    mkdir -p "$root/.context/working"
    if [ -n "$tid" ]; then
        printf 'current_task: %s\n' "$tid" > "$root/.context/working/focus.yaml"
    else
        printf 'current_task:\n' > "$root/.context/working/focus.yaml"
    fi
}

run_gen() {
    local blk="$1" root="$2"
    TASKS_DIR="$root/.tasks" PROJECT_ROOT="$root" bash "$blk" 2>/dev/null
}

# Scenario shared by every case: three agent-owned horizon:now tasks.
#   T-1001 — lexicographically FIRST, oldest update  (the pre-fix winner)
#   T-5002 — middle
#   T-9003 — lexicographically LAST, newest update   (the recency winner)
build_scenario() {
    local root="$1"
    make_task "$root" T-1001 agent now 2026-01-01T00:00:00Z "Oldest, lexicographically first"
    make_task "$root" T-5002 agent now 2026-05-05T00:00:00Z "Middle"
    make_task "$root" T-9003 agent now 2026-09-09T00:00:00Z "Newest, lexicographically last"
}

BLK="$WORK/blk.sh"
extract_block "$HANDOVER_SH" "$BLK" || die "could not extract generator block from $HANDOVER_SH"

echo "handover-suggested-action-fixtures: $HANDOVER_SH"
echo

# ── Case 1: current focus wins, even when it is neither first nor newest ────────
R1="$WORK/c1"; build_scenario "$R1"; set_focus "$R1" T-5002
out="$(run_gen "$BLK" "$R1")"
case "$out" in
    *"Continue T-5002"*) ok "case 1: current focus is chosen over id order and recency" ;;
    *) bad "case 1: current focus is chosen" "Continue T-5002" "$out" ;;
esac

# ── Case 2: no focus → most recently updated, not lexicographically first ───────
R2="$WORK/c2"; build_scenario "$R2"; set_focus "$R2" ""
out="$(run_gen "$BLK" "$R2")"
case "$out" in
    *"Continue T-9003"*) ok "case 2: absent focus falls back to most recently updated" ;;
    *) bad "case 2: absent focus falls back to recency" "Continue T-9003" "$out" ;;
esac

# ── Case 3: the lexicographic-first task is never chosen by id order alone ──────
# This is the regression itself: T-1001 is first by string sort but neither
# focused nor recent, so it must not win.
case "$out" in
    *"Continue T-1001"*) bad "case 3: lexicographic-first must not win" "not T-1001" "$out" ;;
    *) ok "case 3: lexicographically-first task is not chosen on id order" ;;
esac

# ── Case 4: a stale focus pointing at a non-candidate degrades to recency ──────
R4="$WORK/c4"; build_scenario "$R4"; set_focus "$R4" T-9999
out="$(run_gen "$BLK" "$R4")"
case "$out" in
    *"Continue T-9003"*) ok "case 4: focus naming an unknown task degrades to recency" ;;
    *) bad "case 4: unknown focus degrades to recency" "Continue T-9003" "$out" ;;
esac

# ── Case 5: the output declares itself a mechanical fallback ────────────────────
case "$out" in
    *"Mechanical fallback"*) ok "case 5: output is labelled a mechanical fallback" ;;
    *) bad "case 5: output labelled mechanical" "text containing 'Mechanical fallback'" "$out" ;;
esac

# ── Case 6: human-owned tasks still rank after agent-owned ─────────────────────
R6="$WORK/c6"; mkdir -p "$R6"
make_task "$R6" T-2001 human now 2026-09-09T00:00:00Z "Human, newest"
make_task "$R6" T-2002 agent now 2026-01-01T00:00:00Z "Agent, oldest"
set_focus "$R6" ""
out="$(run_gen "$BLK" "$R6")"
case "$out" in
    *"Continue T-2002"*) ok "case 6: agent-owned still outranks human-owned" ;;
    *) bad "case 6: agent-owned outranks human-owned" "Continue T-2002" "$out" ;;
esac

# ── Case 7: both frontmatter blocks stamp enrichment_status ─────────────────────
n_stamp="$(grep -c 'enrichment_status: pending' "$HANDOVER_SH" 2>/dev/null || echo 0)"
if [ "$n_stamp" -ge 2 ]; then
    ok "case 7: both generation paths stamp enrichment_status: pending ($n_stamp)"
else
    bad "case 7: both generation paths stamp enrichment_status" ">=2 occurrences" "$n_stamp"
fi

# ── Case 8: the fabricated narrative literals are gone ──────────────────────────
# The generator must not emit a bare "None" or the gaps-register deflection as if
# they were findings. Checked as whole lines inside the emitted template.
if grep -qx 'None' "$HANDOVER_SH" || grep -qx 'See gaps register above\.' "$HANDOVER_SH"; then
    bad "case 8: fabricated narrative literals removed" "no bare 'None' / 'See gaps register above.' lines" "still present"
else
    ok "case 8: fabricated narrative literals are gone"
fi

# ── Case 9 (LOAD-BEARING): the same assertions must FAIL against the pre-fix ────
# Without this the suite could be green against any implementation. We re-run
# cases 1 and 2 against the script as it stands in git and require both to fail.
echo
PRE="$WORK/prefix-handover.sh"
if git -C "$REPO_ROOT" show "$PREFIX_REF:.agentic-framework/agents/handover/handover.sh" > "$PRE" 2>/dev/null && [ -s "$PRE" ]; then
    PREBLK="$WORK/preblk.sh"
    if extract_block "$PRE" "$PREBLK"; then
        pre1="$(run_gen "$PREBLK" "$R1")"   # focus=T-5002 scenario
        pre2="$(run_gen "$PREBLK" "$R2")"   # no-focus scenario
        pre_wrong=0
        case "$pre1" in *"Continue T-5002"*) : ;; *) pre_wrong=$((pre_wrong+1)) ;; esac
        case "$pre2" in *"Continue T-9003"*) : ;; *) pre_wrong=$((pre_wrong+1)) ;; esac
        if [ "$pre_wrong" -eq 2 ]; then
            ok "case 9: suite is RED against the pre-fix script (both cases fail as expected)"
            printf '       pre-fix picked: %s | %s\n' \
                "$(printf '%s' "$pre1" | head -1)" "$(printf '%s' "$pre2" | head -1)"
        else
            bad "case 9: suite must be red against pre-fix" "both case 1 and 2 to fail pre-fix" \
                "only $pre_wrong of 2 failed — the fixtures do not discriminate"
        fi
    else
        die "could not extract generator block from the pre-fix script at $PREFIX_REF"
    fi
else
    die "could not read $PREFIX_REF:.agentic-framework/agents/handover/handover.sh (needed for the load-bearing case)"
fi

echo
printf 'passed: %d   failed: %d\n' "$PASS" "$FAIL"
if [ "$FAIL" -eq 0 ]; then
    echo "ALL PASS"
    exit 0
fi
echo "FAILURES PRESENT"
exit 1
