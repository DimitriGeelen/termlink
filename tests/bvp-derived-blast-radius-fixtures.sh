#!/usr/bin/env bash
# tests/bvp-derived-blast-radius-fixtures.sh — T-3189
#
# guard-layer: source
#
# The estimator now DERIVES a blast radius from the real files a task's own prose names,
# when neither `components:` nor `target_blast_radius:` can speak (operator decision
# 2026-09-27, SQ-2: 107/120 derive vs 48/120 default-in-template).
#
# Three properties matter more than the derivation itself:
#
#   Case 2 — PRECEDENCE at both boundaries. `components:` beats `target_blast_radius:`
#            beats derived. An inference must never overrule a measurement or an author's
#            declaration. T-3188 shipped a draft that inverted the first boundary; the
#            same mistake is available one tier down, and "hoist the cheap checks" is
#            exactly the refactor that reintroduces it.
#
#   Case 3 — THE DERIVATION CANNOT MANUFACTURE. Nothing readable → None → UNMEASURED.
#            No default, no floor. This is the scope fence: 50% of the corpus visibly
#            uncosted beats 100% costed by numbers nobody thought about.
#
#   Case 4 — THE TEMPLATE-BOILERPLATE TRAP. `.claude/settings.json` appears in 177 of the
#            210 in-corpus build tasks, inside a `#` comment in `## Verification`. Reading
#            it would give 98 tasks a cost derived entirely from guidance they never
#            touch. This is the highest-value case in the file: it is the failure that
#            turns this task back into the strawman it rejected.
#
# HERMETIC BY INJECTION. `_repo_file_index()` reads the live repo, so assertions that
# depended on it would change meaning as files are added (measured during authoring:
# `mod.rs` resolves uniquely in this repo today and might not tomorrow). Every
# deterministic case therefore injects a controlled `_REPO_INDEX`. Case 7 exercises the
# real index on purpose, asserting only a property that cannot rot.
#
# Exit 0 = all pass, 1 = failure, 2 = tooling (fail-closed).

set -uo pipefail

PROJECT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EST="$PROJECT/.agentic-framework/agents/termlink/bvp-estimator/estimator.py"

PASS=0; FAIL=0
ok()   { printf '  \033[0;32mok\033[0m    %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; FAIL=$((FAIL+1)); }

[ -f "$EST" ] || { echo "TOOLING: estimator not found at $EST" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "TOOLING: python3 missing" >&2; exit 2; }
command -v git     >/dev/null 2>&1 || { echo "TOOLING: git missing" >&2; exit 2; }
python3 -c 'import yaml' 2>/dev/null || { echo "TOOLING: PyYAML missing" >&2; exit 2; }

SCRATCH="$(mktemp -d)"; trap 'rm -rf "$SCRATCH"' EXIT

# score <estimator> <fm-yaml> <body> [inject_index:1|0]
# Prints "<value>|<evidence>" — value is UNMEASURED when the scorer returns None.
score() {
    EST_PATH="$1" FM_YAML="$2" BODY_TXT="$3" INJECT="${4:-1}" python3 - <<'PY' 2>/dev/null
import os, sys, importlib.util, yaml
os.environ.setdefault('PROJECT_ROOT', '/opt/termlink')
spec = importlib.util.spec_from_file_location('est', os.environ['EST_PATH'])
m = importlib.util.module_from_spec(spec)
try:
    spec.loader.exec_module(m)
except SystemExit:
    pass
except Exception as exc:
    print("LOADFAIL:%s" % exc); sys.exit(0)

fn = getattr(m, 'score_blast_radius', None)
if fn is None:
    print("NO-FN"); sys.exit(0)

if os.environ.get('INJECT') == '1':
    # A controlled repo: five tracked files. `dup.rs` appears twice, so its basename is
    # ambiguous and must never count; `solo.py` appears once and must count bare.
    tracked = frozenset({
        'scripts/alpha.sh', 'scripts/beta.sh', 'crates/x/src/solo.py',
        'crates/a/src/dup.rs', 'crates/b/src/dup.rs',
        '.tasks/active/T-0001-other.md', '.context/handovers/S-1.md',
        '.context/checks/some-allowlist',
    })
    uniq = frozenset({'alpha.sh', 'beta.sh', 'solo.py'})
    if not hasattr(m, '_REPO_INDEX'):
        print("NO-INDEX-HOOK"); sys.exit(0)
    m._REPO_INDEX = (tracked, uniq)

fm = yaml.safe_load(os.environ['FM_YAML']) or {}
try:
    v, ev = fn(fm, os.environ['BODY_TXT'], [])
except Exception as exc:
    print("RAISED:%s" % exc); sys.exit(0)
print("%s|%s" % ('UNMEASURED' if v is None else v, (ev or [''])[0]))
PY
}
s() { score "$EST" "$1" "$2" "${3:-1}"; }

echo "=== T-3189: derived blast radius from readable task evidence ==="

# ================================================================= Case 0
echo
echo "Case 0 — harness control: is the scorer reachable, injectable, and discriminating?"
probe="$(s 'workflow_type: build' 'touches scripts/alpha.sh')"
case "$probe" in
    NO-FN|RAISED:*|LOADFAIL:*|NO-INDEX-HOOK|"")
        echo "  HARNESS BROKEN: could not exercise the scorer (got '$probe')." >&2
        echo "  Refusing to report a result rather than scoring a subject that never ran." >&2
        exit 2 ;;
esac
ok "scorer callable with an injected index (got: $probe)"
probe2="$(s 'workflow_type: build' 'no file references at all here')"
case "$probe2" in
    UNMEASURED*) ok "and it discriminates: prose with no refs is UNMEASURED" ;;
    *) echo "  HARNESS BROKEN: bare prose returned '$probe2', not UNMEASURED." >&2; exit 2 ;;
esac

# ================================================================= Case 1  ★
echo
echo "Case 1 ★ the derivation fires, and scales with the number of distinct files"
out="$(s 'workflow_type: build' 'edit scripts/alpha.sh')"
[ "${out%%|*}" = "1" ] && ok "1 ref -> 1 ($out)" || fail "1 ref gave '$out'"
out="$(s 'workflow_type: build' 'edit scripts/alpha.sh and scripts/beta.sh')"
[ "${out%%|*}" = "3" ] && ok "2 refs -> 3 ($out)" || fail "2 refs gave '$out'"
out="$(s 'workflow_type: build' 'scripts/alpha.sh scripts/beta.sh crates/x/src/solo.py crates/a/src/dup.rs')"
[ "${out%%|*}" = "5" ] && ok "4 refs -> 5 ($out)" || fail "4 refs gave '$out'"
out="$(s 'workflow_type: build' 'see scripts/alpha.sh and scripts/alpha.sh again')"
[ "${out%%|*}" = "1" ] && ok "duplicates collapse: same path twice is still 1 ($out)" \
    || fail "duplicate path not deduped — got '$out'"
for wf in build refactor test design decommission; do
    out="$(s "workflow_type: $wf" 'edit scripts/alpha.sh')"
    [ "${out%%|*}" = "1" ] && ok "$wf derives too ($out)" || fail "$wf gave '$out'"
done

# ================================================================= Case 2  ★★
echo
echo "Case 2 ★★ PRECEDENCE — measurement > declaration > derivation, at BOTH boundaries"
out="$(s 'workflow_type: build
components: [only-one.py]
target_blast_radius: 9' 'scripts/alpha.sh scripts/beta.sh crates/x/src/solo.py')"
[ "${out%%|*}" = "1" ] && ok "components beats BOTH declaration and derivation ($out)" \
    || fail "a measured single component was overruled — got '$out'"
out="$(s 'workflow_type: build
target_blast_radius: 7' 'scripts/alpha.sh')"
[ "${out%%|*}" = "7" ] && ok "declaration beats derivation ($out)" \
    || fail "the derivation overrode an explicit declaration — got '$out'"
grep -qF 'target_blast_radius' <<< "$out" \
    && ok "  and the winning tier is named in the evidence" \
    || fail "  evidence does not name the declaration tier: '$out'"
out="$(s 'workflow_type: build
components: [a.py, b.py]' 'scripts/alpha.sh')"
[ "${out%%|*}" = "3" ] && ok "components wins with no declaration present ($out)" \
    || fail "components lost to derivation — got '$out'"

# ================================================================= Case 3  ★★
echo
echo "Case 3 ★★ SCOPE FENCE — the derivation cannot manufacture a value"
out="$(s 'workflow_type: build' 'A prose-only task. No paths, no filenames, nothing.')"
[ "${out%%|*}" = "UNMEASURED" ] && ok "nothing readable -> UNMEASURED ($out)" \
    || fail "prose acquired a cost '$out' — the corpus would gain fake numbers"
out="$(s 'workflow_type: build' 'references scripts/does-not-exist.sh and other/unknown.py')"
[ "${out%%|*}" = "UNMEASURED" ] && ok "UNRESOLVABLE paths do not count ($out)" \
    || fail "an unresolvable path was counted as evidence — got '$out'"
out="$(s 'workflow_type: build' 'see crates/a/src/dup.rs mentioned as dup.rs')"
[ "${out%%|*}" = "1" ] && ok "ambiguous BARE name excluded; the full path still counts 1 ($out)" \
    || fail "ambiguous basename handling wrong — got '$out'"
out="$(s 'workflow_type: build' 'only dup.rs is named here')"
[ "${out%%|*}" = "UNMEASURED" ] && ok "an ambiguous bare name ALONE resolves nothing ($out)" \
    || fail "ambiguous bare name counted — got '$out'"

# ================================================================= Case 4  ★★★
echo
echo "Case 4 ★★★ THE TEMPLATE TRAP — boilerplate in '## Verification' must not be read"
# Verbatim shape of the template's L-398 hint: a '#' shell comment inside ## Verification.
TRAP_BODY='## Context

A task that touches nothing in particular.

## Verification

# Enforcement-baseline hint (L-398, T-1886): if you edited `scripts/alpha.sh`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline`.
'
out="$(s 'workflow_type: build' "$TRAP_BODY")"
[ "${out%%|*}" = "UNMEASURED" ] \
    && ok "a '#' comment inside ## Verification is NOT evidence ($out)" \
    || fail "TEMPLATE BOILERPLATE WAS READ AS WORK — got '$out'. This is the 98-task fabrication."
# ...but a REAL command in the same section must still count.
REAL_BODY='## Verification

# a comment naming scripts/beta.sh which must be ignored
bash scripts/alpha.sh > /tmp/.o 2>&1
'
out="$(s 'workflow_type: build' "$REAL_BODY")"
[ "${out%%|*}" = "1" ] \
    && ok "a real command line in the same section DOES count, and only it ($out)" \
    || fail "verification commands are not being read — got '$out'"
HTML_BODY='## Context

<!-- guidance: edit scripts/alpha.sh and scripts/beta.sh as examples -->

Real work described without paths.
'
out="$(s 'workflow_type: build' "$HTML_BODY")"
[ "${out%%|*}" = "UNMEASURED" ] && ok "HTML-comment guidance is NOT evidence ($out)" \
    || fail "HTML comment was read as work — got '$out'"
# A '#' line OUTSIDE ## Verification is a markdown heading, not a comment — must be kept.
HEAD_BODY='## Context

# scripts/alpha.sh

Work on it.
'
out="$(s 'workflow_type: build' "$HEAD_BODY")"
[ "${out%%|*}" = "1" ] \
    && ok "a '#' line OUTSIDE Verification is a heading, still read ($out)" \
    || fail "section-awareness broken: heading dropped — got '$out'"

# ================================================================= Case 5  ★
echo
echo "Case 5 ★ CITATIONS are not blast radius"
out="$(s 'workflow_type: build' 'as described in .tasks/active/T-0001-other.md')"
[ "${out%%|*}" = "UNMEASURED" ] && ok "citing another TASK file is not a change ($out)" \
    || fail "a task-file citation was counted — got '$out'"
out="$(s 'workflow_type: build' 'see .context/handovers/S-1.md for background')"
[ "${out%%|*}" = "UNMEASURED" ] && ok "citing a handover is not a change ($out)" \
    || fail "a handover citation was counted — got '$out'"
out="$(s 'workflow_type: build' 'add an entry to .context/checks/some-allowlist')"
[ "${out%%|*}" = "1" ] && ok "but a guard ALLOWLIST is a real artifact and counts ($out)" \
    || fail "a genuine .context/checks artifact was excluded — got '$out'"

# ================================================================= Case 6
echo
echo "Case 6 — malformed / hostile input falls through, never raises, never defaults"
out="$(s 'workflow_type: build
components: "not-a-list"' 'scripts/alpha.sh')"
case "$out" in
    RAISED:*) fail "malformed components raised: $out" ;;
    *) ok "non-list components handled without raising ($out)" ;;
esac
out="$(s 'workflow_type: build
target_blast_radius: "junk"' 'scripts/alpha.sh')"
[ "${out%%|*}" = "1" ] && ok "junk declaration falls through TO the derivation ($out)" \
    || fail "junk declaration handling changed — got '$out'"
out="$(s 'workflow_type: build
target_blast_radius: "junk"' 'no refs here')"
[ "${out%%|*}" = "UNMEASURED" ] && ok "junk declaration + no refs -> UNMEASURED ($out)" \
    || fail "junk declaration defaulted — got '$out'"
out="$(s 'workflow_type: build' '')"
[ "${out%%|*}" = "UNMEASURED" ] && ok "empty body -> UNMEASURED ($out)" \
    || fail "empty body gave '$out'"

# ================================================================= Case 7  ★★
echo
echo "Case 7 ★★ FAIL-CLOSED: no repo index means UNMEASURED, never a cheap default"
out="$(EST_PATH="$EST" python3 - <<'PY' 2>/dev/null
import os, importlib.util
os.environ.setdefault('PROJECT_ROOT', '/opt/termlink')
spec = importlib.util.spec_from_file_location('est', os.environ['EST_PATH'])
m = importlib.util.module_from_spec(spec)
try: spec.loader.exec_module(m)
except SystemExit: pass
m._REPO_INDEX = (frozenset(), frozenset())      # git missing / not a repo
v, ev = m.score_blast_radius({'workflow_type': 'build'},
                             'touches scripts/alpha.sh and crates/x/src/solo.py', [])
print("%s|%s" % ('UNMEASURED' if v is None else v, (ev or [''])[0]))
PY
)"
[ "${out%%|*}" = "UNMEASURED" ] \
    && ok "an unavailable index yields UNMEASURED, not 0 and not a guess ($out)" \
    || fail "FAILED OPEN: no index produced a value '$out' — 'could not look' became an answer"

# ================================================================= Case 8  ★★
echo
echo "Case 8 ★★ ground truth: the PRE-CHANGE estimator must NOT derive from the body"
PRE="$SCRATCH/pre.py"; found=""
REL=".agentic-framework/agents/termlink/bvp-estimator/estimator.py"
# Behavioural search, not text. Text searches have been defeated repeatedly in this repo by
# fixes that quote what they changed (the comment-vs-code trap, T-3178). Walk this file's
# own history, RUN each version, and take the first where a body-only task is UNMEASURED
# while a declared one still scores — both legs required, so a copy that merely failed to
# load cannot masquerade as the pre-change behaviour.
for sha in $(git -C "$PROJECT" log --format=%H -n 25 -- "$REL" 2>/dev/null); do
    git -C "$PROJECT" show "$sha:$REL" > "$PRE" 2>/dev/null || continue
    d="$(score "$PRE" 'workflow_type: build' 'edit scripts/alpha.sh and scripts/beta.sh' 0)"
    t="$(score "$PRE" 'workflow_type: build
target_blast_radius: 5' 'edit scripts/alpha.sh' 0)"
    [ "${d%%|*}" = "UNMEASURED" ] && [ "${t%%|*}" = "5" ] || continue
    found="${sha:0:9}"; break
done
if [ -z "$found" ]; then
    fail "MUTATION SETUP BROKEN — no revision in the last 25 shows the pre-change behaviour"
else
    ok "pre-change estimator recovered and behaviourally confirmed ($found)"
    ok "  it ignored body evidence while still honouring target_blast_radius"
    ok "  so Case 1 is load-bearing rather than vacuous"
fi

echo
echo "=== SUMMARY ==="
echo "Pass: $PASS"
echo "Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
