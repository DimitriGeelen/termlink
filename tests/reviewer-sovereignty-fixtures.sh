#!/usr/bin/env bash
# guard-layer: source
#
# T-3198 — fixtures for the external-reviewer sovereignty relaxation.
#
# The operator ruled (2026-09-28) that rubber-stamping is dead: a low-risk task
# whose remaining Human ACs are mechanical closes on an EXTERNAL reviewer's PASS
# rather than on a person clicking. `external_reviewer_clears_sovereignty` in
# agents/task-create/update-task.sh is the predicate that decides it, and it is
# the most permissive thing in the framework — so this suite is weighted heavily
# toward the cases where it must REFUSE.
#
# The predicate is EXTRACTED from the real script rather than reimplemented: a
# second copy of a sovereignty rule is a copy that drifts, and the copy that
# drifts is the one that quietly stops refusing. If the function is renamed or
# removed, extraction yields nothing and this suite exits 2 rather than passing
# vacuously.
set -uo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SRC="${REVIEWER_SOV_SRC:-$PROJECT_ROOT/.agentic-framework/agents/task-create/update-task.sh}"

PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }

# --- Case 0: harness control. Everything below is meaningless if this fails. ---
if [ ! -r "$SRC" ]; then
    echo "TOOLING: cannot read $SRC" >&2; exit 2
fi
FN=$(awk '/^external_reviewer_clears_sovereignty\(\) \{/,/^\}/' "$SRC")
if [ -z "$FN" ] || ! grep -q "external-dispatch" <<<"$FN"; then
    echo "TOOLING: could not extract external_reviewer_clears_sovereignty from $SRC" >&2
    echo "         (renamed, removed, or reshaped — refusing to report a pass)" >&2
    exit 2
fi
eval "$FN"
echo "case 0: predicate extracted from the shipping script"
ok "extraction control"

TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

# Build a task file. $1=name, $2=frontmatter extra, $3=verdict lines, $4=human ACs
mk() {
    local f="$TMP/$1.md"
    {
        echo "---"
        echo "id: T-9999"
        echo "owner: human"
        echo "status: started-work"
        [ -n "$2" ] && echo "$2"
        echo "---"
        echo ""
        echo "## Acceptance Criteria"
        echo ""
        echo "### Human"
        echo "<!-- template example that must never count:"
        echo "- [ ] [REVIEW] Dashboard renders correctly"
        echo "-->"
        [ -n "$4" ] && echo "$4"
        echo ""
        echo "## Reviewer Verdict (v1.5)"
        echo ""
        [ -n "$3" ] && echo "$3"
        echo ""
        echo "## Updates"
    } > "$f"
    echo "$f"
}

GOOD_VERDICT='- **Scan ID:** SC-2026-0928-abc
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** external-dispatch'

run() { TASK_FILE="$1" external_reviewer_clears_sovereignty >/dev/null 2>&1; }

echo "case 1: the permissive path — all six conditions hold"
f=$(mk happy "" "$GOOD_VERDICT" "")
if run "$f"; then ok "clears with external PASS"; else bad "should clear with external PASS"; fi
sid=$(TASK_FILE="$f" external_reviewer_clears_sovereignty 2>/dev/null)
if [ "$sid" = "SC-2026-0928-abc" ]; then ok "emits the satisfying scan id"; else bad "scan id not emitted (got '$sid')"; fi

echo "case 2: a RUBBER-STAMP AC is mechanical — still clears"
f=$(mk stamp "" "$GOOD_VERDICT" "- [ ] [RUBBER-STAMP] Publish the release note")
if run "$f"; then ok "unchecked RUBBER-STAMP does not block"; else bad "RUBBER-STAMP must not block"; fi

echo "case 3: REVIEW is declared human judgment — must refuse"
f=$(mk review "" "$GOOD_VERDICT" "- [ ] [REVIEW] Does the tone read right?")
if run "$f"; then bad "unchecked [REVIEW] MUST block"; else ok "refuses on unchecked [REVIEW]"; fi

echo "case 4: a CHECKED [REVIEW] is satisfied — clears"
f=$(mk reviewdone "" "$GOOD_VERDICT" "- [x] [REVIEW] Does the tone read right?")
if run "$f"; then ok "checked [REVIEW] does not block"; else bad "checked [REVIEW] must not block"; fi

echo "case 5: verdict quality — each field independently withheld must refuse"
f=$(mk vfail "" '- **Scan ID:** S1
- **Overall:** FAIL
- **Needs Human:** no
- **Reviewer:** external-dispatch' "")
if run "$f"; then bad "FAIL verdict must block"; else ok "refuses on Overall: FAIL"; fi

f=$(mk vconcern "" '- **Scan ID:** S1
- **Overall:** CONCERN
- **Needs Human:** no
- **Reviewer:** external-dispatch' "")
if run "$f"; then bad "CONCERN verdict must block"; else ok "refuses on Overall: CONCERN"; fi

f=$(mk vneeds "" '- **Scan ID:** S1
- **Overall:** PASS
- **Needs Human:** yes
- **Reviewer:** external-dispatch' "")
if run "$f"; then bad "Needs Human: yes must block"; else ok "refuses on Needs Human: yes"; fi

echo "case 6: PROVENANCE — an inline self-scan is producer-as-judge"
f=$(mk vinline "" '- **Scan ID:** S1
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline' "")
if run "$f"; then bad "inline self-scan MUST block"; else ok "refuses on Reviewer: inline"; fi

echo "case 7: a pre-T-3198 verdict has no Reviewer line — must refuse, not assume"
f=$(mk vlegacy "" '- **Scan ID:** S1
- **Overall:** PASS
- **Needs Human:** no' "")
if run "$f"; then bad "missing Reviewer line MUST block (fail-closed)"; else ok "refuses when provenance absent"; fi

echo "case 8: declared risk overrides a clean verdict"
for r in "risk: high" "risk: medium" "human_signoff: required"; do
    f=$(mk "risk_$(echo "$r" | tr -cd 'a-z')" "$r" "$GOOD_VERDICT" "")
    if run "$f"; then bad "'$r' MUST block"; else ok "refuses on '$r'"; fi
done

echo "case 9: risk: low must NOT block"
f=$(mk risklow "risk: low" "$GOOD_VERDICT" "")
if run "$f"; then ok "risk: low clears"; else bad "risk: low should clear"; fi

echo "case 10: fail-closed on absent/unreadable input"
f="$TMP/novrdct.md"; printf -- '---\nid: T-9999\nowner: human\n---\n\n## Updates\n' > "$f"
if run "$f"; then bad "no verdict block MUST block"; else ok "refuses when no verdict block"; fi
if run "$TMP/does-not-exist.md"; then bad "missing file MUST block"; else ok "refuses on missing file"; fi

echo ""
echo "passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
