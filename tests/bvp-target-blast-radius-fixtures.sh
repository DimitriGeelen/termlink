#!/usr/bin/env bash
# tests/bvp-target-blast-radius-fixtures.sh — T-3188
#
# guard-layer: source
#
# The `target_blast_radius` cost fallback now applies to EVERY workflow_type, not only
# `inception` (operator decision 2026-09-27). Two properties matter more than the
# extension itself:
#
#   Case 2 — `components:` STILL WINS when present. A measurement must never be
#            overridden by a prediction, and the extension must not have reordered them.
#   Case 3 — an ABSENT field still resolves to UNMEASURED. The scope fence is "extend the
#            fallback, do not populate the field": 85% of the corpus visibly uncosted is a
#            better state than 85% costed by numbers nobody thought about. That is
#            T-3185's no-signal lesson moved onto the cost axis, and it is the strawman
#            this decision explicitly rejected.
#
# Exit 0 = all pass, 1 = failure, 2 = tooling (fail-closed).

set -uo pipefail

PROJECT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EST="$PROJECT/.agentic-framework/agents/termlink/bvp-estimator/estimator.py"

PASS=0; FAIL=0
ok()   { printf '  \033[0;32mok\033[0m    %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; FAIL=$((FAIL+1)); }

[ -f "$EST" ] || { echo "TOOLING: estimator not found" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "TOOLING: python3 missing" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "TOOLING: git missing" >&2; exit 2; }

SCRATCH="$(mktemp -d)"; trap 'rm -rf "$SCRATCH"' EXIT

# Call the scorer directly. It is a module-level function, so this needs no project tree
# and no live hub — the frontmatter dict IS the whole input.
radius_at() { # <estimator-path> <yaml-frontmatter>
    EST_PATH="$1" FM_YAML="$2" python3 - <<'PY' 2>/dev/null
import os, sys, importlib.util, yaml
os.environ.setdefault('PROJECT_ROOT', os.getcwd())
os.environ.setdefault('FRAMEWORK_ROOT', os.path.join(os.getcwd(), '.agentic-framework'))
spec = importlib.util.spec_from_file_location('est', os.environ['EST_PATH'])
m = importlib.util.module_from_spec(spec)
try:
    spec.loader.exec_module(m)
except SystemExit:
    pass
# Named explicitly. A first draft discovered the function by scanning dir() for a
# private name containing "blast" — score_blast_radius is PUBLIC, so it found nothing and
# the harness control (correctly) refused to score rather than reporting a result.
fn = getattr(m, 'score_blast_radius', None)
if fn is None:
    print("NO-FN"); sys.exit(0)
fm = yaml.safe_load(os.environ['FM_YAML']) or {}
try:
    # signature is (fm, body, tags)
    v, ev = fn(fm, '', [])
except Exception as exc:
    print("RAISED:%s" % exc); sys.exit(0)
print("%s|%s" % ('UNMEASURED' if v is None else v, (ev or [''])[0]))
PY
}
radius() { radius_at "$EST" "$1"; }

echo "=== T-3188: target_blast_radius fallback for all workflow types ==="

# ================================================================= Case 0
echo
echo "Case 0 — harness control: is the scorer reachable and discriminating?"
probe="$(radius 'workflow_type: build
components: [a.py, b.py]')"
case "$probe" in
    NO-FN|RAISED:*|"")
        echo "  HARNESS BROKEN: could not call the blast-radius scorer (got '$probe')." >&2
        exit 2 ;;
esac
ok "the scorer is callable and answers (components path: $probe)"
probe2="$(radius 'workflow_type: build')"
case "$probe2" in
    UNMEASURED*) ok "and it discriminates: a bare build task is UNMEASURED" ;;
    *) echo "  HARNESS BROKEN: a bare task returned '$probe2', not UNMEASURED." >&2; exit 2 ;;
esac

# ================================================================= Case 1  ★
echo
echo "Case 1 ★ the fallback fires for NON-inception workflow types"
for wf in build refactor test design decommission; do
    out="$(radius "workflow_type: $wf
target_blast_radius: 5")"
    if [ "${out%%|*}" = "5" ]; then ok "$wf -> 5 ($out)"; else
        fail "$wf did not use the fallback — got '$out'"; fi
done
out="$(radius 'workflow_type: inception
target_blast_radius: 3')"
[ "${out%%|*}" = "3" ] && ok "inception still works, unchanged ($out)" \
    || fail "inception regressed — got '$out'"

# ================================================================= Case 2  ★★
echo
echo "Case 2 ★★ components: STILL WINS — a measurement outranks a prediction"
out="$(radius 'workflow_type: build
target_blast_radius: 9
components: [only-one.py]')"
if [ "${out%%|*}" = "1" ]; then
    ok "one real component beats a predicted 9 ($out)"
else
    fail "the prediction overrode the measurement — got '$out'"
fi

# ================================================================= Case 3  ★★
echo
echo "Case 3 ★★ SCOPE FENCE — an ABSENT field stays UNMEASURED, never a default"
out="$(radius 'workflow_type: build')"
[ "${out%%|*}" = "UNMEASURED" ] && ok "absent -> UNMEASURED ($out)" \
    || fail "absent acquired a value '$out' — the corpus would silently gain fake costs"
out="$(radius 'workflow_type: refactor
components: []')"
[ "${out%%|*}" = "UNMEASURED" ] && ok "empty components, no field -> UNMEASURED" \
    || fail "empty components acquired a value '$out'"

# ================================================================= Case 4
echo
echo "Case 4 — a MALFORMED value falls through, never raises, never defaults"
out="$(radius 'workflow_type: build
target_blast_radius: "not-a-number"
components: [a.py, b.py, c.py]')"
[ "${out%%|*}" = "3" ] && ok "malformed falls through to the components count ($out)" \
    || fail "malformed handling changed — got '$out'"
out="$(radius 'workflow_type: build
target_blast_radius: "junk"')"
[ "${out%%|*}" = "UNMEASURED" ] && ok "malformed with no components -> UNMEASURED" \
    || fail "malformed with no components got '$out'"
out="$(radius 'workflow_type: build
target_blast_radius: 99')"
[ "${out%%|*}" = "9" ] && ok "out-of-range clamps to 9 ($out)" || fail "clamp broken: '$out'"

# ================================================================= Case 5
echo
echo "Case 5 — the evidence names WHICH path answered"
out="$(radius 'workflow_type: build
target_blast_radius: 4')"
grep -qF 'build-T-3188' <<< "$out" \
    && ok "non-inception evidence is attributable ($out)" \
    || fail "evidence does not name the path — got '$out'"
out="$(radius 'workflow_type: inception
target_blast_radius: 4')"
grep -qF 'inception-T-2189' <<< "$out" \
    && ok "inception evidence unchanged ($out)" \
    || fail "inception evidence changed — got '$out'"

# ================================================================= Case 6  ★★
echo
echo "Case 6 ★★ ground truth: the PRE-CHANGE estimator must REFUSE a build fallback"
PRE="$SCRATCH/pre.py"; found=""
REL=".agentic-framework/agents/termlink/bvp-estimator/estimator.py"
# Behavioural search, not text: walk this file's own history, run each version, take the
# first where a build task with target_blast_radius is UNMEASURED. Text searches were
# defeated three times in this repo by fixes that quote the thing they changed.
for sha in $(git -C "$PROJECT" log --format=%H -n 25 -- "$REL" 2>/dev/null); do
    git -C "$PROJECT" show "$sha:$REL" > "$PRE" 2>/dev/null || continue
    b="$(radius_at "$PRE" 'workflow_type: build
target_blast_radius: 5')"
    i="$(radius_at "$PRE" 'workflow_type: inception
target_blast_radius: 5')"
    # pre-change: build IGNORES the field, inception HONOURS it. Both legs required, or a
    # copy that simply failed to load would look like the defect.
    [ "${b%%|*}" = "UNMEASURED" ] && [ "${i%%|*}" = "5" ] || continue
    found="${sha:0:9}"; break
done
if [ -z "$found" ]; then
    fail "MUTATION SETUP BROKEN — no revision in the last 25 shows the pre-change split"
else
    ok "pre-change estimator recovered and behaviourally confirmed ($found)"
    ok "  it ignored target_blast_radius for build while honouring it for inception"
    ok "  so Case 1 is load-bearing rather than vacuous"
fi

echo
echo "=== SUMMARY ==="
echo "Pass: $PASS"
echo "Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
