#!/usr/bin/env bash
# tests/bvp-no-signal-ranking-fixtures.sh — T-3185
#
# guard-layer: source
#
# Operator decision 2026-09-27, option (d) of four (832's offset 172): keep the no-signal
# score of 2, but exclude a fully-unassessed task from quadrant assignment and from the
# medians that set everyone else's thresholds.
#
# THE PROPERTY UNDER TEST IS PLACEMENT, NOT SCORE. Option (d) was chosen because it
# rescores nothing, so Case 4 asserting the scores are UNCHANGED is as load-bearing as
# Case 1 asserting the exclusion happened. A version of this that quietly adjusted values
# would look identical in the ranking and would be a calibration change nobody authorised.
#
# Exit 0 = all pass, 1 = failure, 2 = tooling (fail-closed).

set -uo pipefail

PROJECT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FW_ROOT="$PROJECT/.agentic-framework"
BVP="$FW_ROOT/lib/bvp.sh"

PASS=0; FAIL=0
ok()   { printf '  \033[0;32mok\033[0m    %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; FAIL=$((FAIL+1)); }

[ -f "$BVP" ] || { echo "TOOLING: $BVP not found" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "TOOLING: python3 missing" >&2; exit 2; }
python3 -c 'import yaml' 2>/dev/null || { echo "TOOLING: PyYAML missing" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "TOOLING: git missing" >&2; exit 2; }

SCRATCH="$(mktemp -d)"; trap 'rm -rf "$SCRATCH"' EXIT
PROJ="$SCRATCH/p"; mkdir -p "$PROJ/.tasks/active" "$PROJ/.context" "$PROJ/policy"
printf 'project_name: fx\n' > "$PROJ/.framework.yaml"
cp -r "$PROJECT/policy/." "$PROJ/policy/" 2>/dev/null || true

mk_task() { # <id> <rationale> <extra-frontmatter-line>
    cat > "$PROJ/.tasks/active/$1-fx.md" <<MD
---
id: $1
name: "Fixture $1"
status: started-work
workflow_type: build
owner: agent
bvp_scores_proposed:
  - ts: '2026-09-27T00:00:00Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
    rationale: $2
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-27T00:00:00Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 4
    rationale: blast_radius=3 (2-components); tier=2; effort=4
    rubric_sha: e4a00f38e801
---

# $1
MD
}

rank_at() { # <lib> [args...]
    env PROJECT_ROOT="$PROJ" FRAMEWORK_ROOT="$FW_ROOT" BVP_LIB="$1" \
        bash -c '. "$BVP_LIB"; bvp_dispatch "$@"' _ "${@:2}" 2>&1
}
rank() { rank_at "$BVP" "$@"; }

quad_of() { # <task-id> <output>
    grep -E "^$1 " <<< "$2" | awk '{print $5}'
}

# All three drivers no-signal -> fully unassessed.
mk_task T-9001 'D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal)'
# Two of three carry real evidence -> assessed, keeps its place.
mk_task T-9002 'D1=2 (body:structural-gate); D2=2 (body:fw-audit-or-doctor); D3=2 (no-signal)'
# Rationale absent entirely -> cannot tell, must NOT be excluded.
mk_task T-9003 ''

echo "=== T-3185: unassessed tasks are excluded from the ranking, not rescored ==="

# ================================================================= Case 0
echo
echo "Case 0 — harness control"
out="$(rank --include-proposed)"
if ! grep -qE '^T-900[123] ' <<< "$out"; then
    echo "  HARNESS BROKEN: no fixture task appeared in the ranking." >&2
    echo "  Got: $(head -c 300 <<< "$out")" >&2
    exit 2
fi
ok "the ranking runs and lists the fixture tasks"

# ================================================================= Case 1
echo
echo "Case 1 ★ a fully no-signal task gets NO quadrant"
q1="$(quad_of T-9001 "$out")"
[ "$q1" = "-" ] && ok "T-9001 (all no-signal) has no quadrant" \
                || fail "T-9001 got quadrant '$q1' — should be '-'"

# ================================================================= Case 2
echo
echo "Case 2 — a task with ANY real signal keeps its quadrant"
q2="$(quad_of T-9002 "$out")"
if [ -n "$q2" ] && [ "$q2" != "-" ]; then ok "T-9002 (partial signal) keeps quadrant '$q2'"; else
    fail "T-9002 lost its quadrant — the filter is too broad"; fi

# ================================================================= Case 3
echo
echo "Case 3 ★ fail-safe: an unreadable rationale is NOT treated as no-signal"
q3="$(quad_of T-9003 "$out")"
if [ -n "$q3" ] && [ "$q3" != "-" ]; then
    ok "T-9003 (no rationale) keeps its quadrant — 'could not measure' is not 'empty'"
else
    fail "T-9003 was excluded on an unreadable rationale (T-3105: did-not-measure != measured-none)"
fi

# ================================================================= Case 4  ★
echo
echo "Case 4 ★ NOTHING WAS RESCORED — option (d) changes placement only"
# Every fixture carries the same three 2s, so every BVP total must be identical
# regardless of exclusion. A version that adjusted values would diverge here.
vals="$(grep -E '^T-900[123] ' <<< "$out" | awk '{print $2}' | sort -u | tr '\n' ' ')"
if [ "$(wc -w <<< "$vals")" -eq 1 ]; then
    ok "all three score identically ($vals) — exclusion did not touch any value"
else
    fail "scores diverged ($vals) — this changed calibration, which (d) must not"
fi

# ================================================================= Case 5
echo
echo "Case 5 — the exclusion is DISCLOSED, never silent (T-2680/T-3068)"
grep -qF 'scored every driver no-signal' <<< "$out" \
    && ok "the ranking says how many were excluded and why" \
    || fail "no disclosure printed — a filter that removes tasks silently reads as coverage"
grep -qF 'UNASSESSED, not low-value' <<< "$out" \
    && ok "the disclosure distinguishes unassessed from low-value" \
    || fail "disclosure does not make the unassessed/low-value distinction"

# ================================================================= Case 6  ★★
echo
echo "Case 6 ★★ ground truth: the PRE-CHANGE code, from git, must still rank the stub"
PRE="$SCRATCH/pre.sh"; found=""
# Deterministic, not a walk. `git log -S<symbol>` lists the commits that changed the
# symbol's occurrence count; the LAST of those introduced it, so its parent is the
# pre-change tree. The earlier form walked HEAD..HEAD~15 looking for a ref that did not
# contain the symbol, which is fragile twice over: it re-selects as soon as the change is
# committed, and a comment naming the symbol defeats the match (the trap that fired three
# times in T-3178 and is now a registered learning).
_intro="$(git -C "$PROJECT" log -S'def _proposal_is_all_no_signal' --format=%H \
          -- .agentic-framework/lib/bvp.sh 2>/dev/null | tail -1)"
if [ -n "$_intro" ] && git -C "$PROJECT" show "${_intro}^:.agentic-framework/lib/bvp.sh" \
        > "$PRE" 2>/dev/null; then
    found="${_intro:0:9}^"
fi
if [ -z "$found" ]; then
    fail "MUTATION SETUP BROKEN — no ref in HEAD..HEAD~15 predates the change"
else
    ok "pre-change copy recovered from git ($found)"
    pout="$(rank_at "$PRE" --include-proposed)"
    if grep -qE '^T-9001 ' <<< "$pout"; then
        ok "control: pre-change copy runs and ranks"
        pq="$(quad_of T-9001 "$pout")"
        if [ -n "$pq" ] && [ "$pq" != "-" ]; then
            ok "pre-change code DID give the all-no-signal stub a quadrant ('$pq') — the change is load-bearing"
        else
            fail "pre-change code already excluded it; Case 1 proves nothing"
        fi
    else
        fail "MUTATION SETUP BROKEN — pre-change copy did not rank"
    fi
fi

echo
echo "=== SUMMARY ==="
echo "Pass: $PASS"
echo "Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
