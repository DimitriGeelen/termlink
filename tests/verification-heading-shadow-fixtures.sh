#!/usr/bin/env bash
# Fixtures for scripts/check-verification-heading-shadow.sh (T-2877).
#
# Weighted toward the FIRING cases and the false-positive guards: a
# register-driven check is trivially green when nothing is wrong, and a green
# check that cannot go red is not a check. Three mutants are pinned at the end.
#
# Hermetic: builds its own task corpus in a temp dir. It DOES source the repo's
# real `lib/verification-port.sh`, deliberately — the whole point of the check is
# that it asserts against the extractor the gate actually uses.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$ROOT/scripts/check-verification-heading-shadow.sh"
PASS=0; FAIL=0

ok()   { PASS=$((PASS+1)); echo "  ok   - $1"; }
bad()  { FAIL=$((FAIL+1)); echo "  FAIL - $1"; }
assert_rc() { # <expected> <actual> <label>
  if [ "$1" = "$2" ]; then ok "$3 (rc=$2)"; else bad "$3 (expected rc=$1, got rc=$2)"; fi
}
assert_has() { # <needle> <haystack> <label>
  case "$2" in *"$1"*) ok "$3" ;; *) bad "$3 — missing: $1" ;; esac
}
assert_not() {
  case "$2" in *"$1"*) bad "$3 — unexpectedly present: $1" ;; *) ok "$3" ;; esac
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkcorpus() { rm -rf "$TMP/t"; mkdir -p "$TMP/t/active" "$TMP/t/completed"; }

clean_task() {
  cat > "$1" <<'EOF'
---
id: T-9001
---
# T-9001

## Acceptance Criteria
- [x] done

## Verification

grep -q "something" README.md
test -f Cargo.toml

## Updates
EOF
}

# The real shape, reduced: a counterfeit heading at column 0 above the genuine one,
# with an orphaned `-->` and no opening `<!--`.
# T-3370: the shadow AEF 1.8.3's exact-match extractor still falls for — an EXACT
# '## Verification' line inside an HTML comment before the real heading. First exact
# match wins; the comment's prose reaches the gate.
exact_shadow_task() {
  printf '%s\n' '---' 'id: T-9002' '---' '# T-9002' '' '## Acceptance Criteria' '- [x] done' '' \
    '<!-- template guidance' '## Verification' '1. Open https://example.com/dashboard in browser' '-->' '' \
    '## Verification' '' 'grep -q "real" README.md' '' '## Updates' > "$1"
}

shadow_task() {
  cat > "$1" <<'EOF'
---
id: T-9002
---
# T-9002

## Acceptance Criteria
- [x] done

## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste.
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
-->

## Verification

grep -q "real" README.md

## Updates
EOF
}

echo "== check-verification-heading-shadow fixtures =="

# --- 1. clean corpus ---------------------------------------------------------
mkcorpus; clean_task "$TMP/t/active/T-9001.md"
out="$(bash "$CHECK" --tasks-dir "$TMP/t" 2>&1)"; rc=$?
assert_rc 0 "$rc" "clean corpus exits 0"
assert_has "clean" "$out" "clean corpus says clean"
assert_has "SCOPE" "$out" "clean path states its scope (T-2680)"
assert_has "does NOT verify" "$out" "clean path disclaims adequacy"

# --- 2a. the ORIGINAL defect shape is fixed upstream (T-3370 regression pin) --
# A line that merely STARTS with '## Verification' (the wrapped template comment)
# shadowed the real heading under the old sed range. AEF 1.8.3 (its T-3134) anchors the
# heading exactly, so this shape no longer reaches the gate. If it fires again, the
# extractor regressed to prefix matching.
mkcorpus; shadow_task "$TMP/t/active/T-9002.md"
out="$(bash "$CHECK" --tasks-dir "$TMP/t" 2>&1)"; rc=$?
assert_rc 0 "$rc" "prefix-shadowed heading is clean under the 1.8.3 exact-match extractor"

# --- 2b. the shadow 1.8.3 still falls for: an EXACT heading inside a comment ----
# First exact match wins and its comment opener sits ABOVE the block, so the block's
# '-->' is orphaned and the comment's prose reaches the gate.
mkcorpus; exact_shadow_task "$TMP/t/active/T-9002.md"
out="$(bash "$CHECK" --tasks-dir "$TMP/t" 2>&1)"; rc=$?
assert_rc 1 "$rc" "shadowed heading fires"
assert_has "FIRING" "$out" "firing path says FIRING"
assert_has "T-9002.md" "$out" "names the offending file"
assert_has "markdown prose" "$out" "explains what the gate would run"
assert_has "headings=2" "$out" "diagnosis reports the duplicate heading count"
# The opener of the counterfeit's comment sits above the block, so across the file the
# comment is balanced: the diagnosis must say so rather than claim an orphan (T-3370).
assert_has "orphaned '-->' with no opener=no" "$out" "diagnosis reports the comment as balanced for this shape"
assert_has "Remediation" "$out" "firing path carries remediation"
assert_has "SCOPE" "$out" "firing path also states scope"

# --- 3. each prose arm independently -----------------------------------------
for arm in numbered checkbox bold; do
  mkcorpus
  case "$arm" in
    numbered) body='1. Open the dashboard in a browser' ;;
    checkbox) body='- [ ] [REVIEW] something' ;;
    bold)     body='**Steps:**' ;;
  esac
  printf '%s\n' '---' 'id: T-9003' '---' '# T-9003' '' '## Verification' '' "$body" '' '## Updates' > "$TMP/t/active/T-9003.md"
  out="$(bash "$CHECK" --tasks-dir "$TMP/t" 2>&1)"; rc=$?
  assert_rc 1 "$rc" "prose arm '$arm' fires"
done

# --- 4. false-positive guards ------------------------------------------------
mkcorpus
printf '%s\n' '---' 'id: T-9004' '---' '# T-9004' '' '## Verification' '' \
  'grep -q "1. step one" docs/x.md' \
  'test -f a && echo "- [ ] not a checkbox"' \
  'printf "**bold** in output"' '' '## Updates' > "$TMP/t/active/T-9004.md"
out="$(bash "$CHECK" --tasks-dir "$TMP/t" 2>&1)"; rc=$?
assert_rc 0 "$rc" "commands merely CONTAINING prose markers do not fire"
assert_not "T-9004" "$out" "no false positive on prose-looking arguments"

# --- 5. empty verification block is in scope and passes ----------------------
mkcorpus
printf '%s\n' '---' 'id: T-9005' '---' '# T-9005' '' '## Verification' '' '## Updates' > "$TMP/t/active/T-9005.md"
clean_task "$TMP/t/active/T-9001.md"
out="$(bash "$CHECK" --tasks-dir "$TMP/t" 2>&1)"; rc=$?
assert_rc 0 "$rc" "a task with an empty verification block passes (documented scope gap)"

# --- 6. allowlist ------------------------------------------------------------
mkcorpus; exact_shadow_task "$TMP/t/active/T-9002.md"
echo "T-9002.md  # acknowledged for fixture purposes" > "$TMP/allow"
out="$(bash "$CHECK" --tasks-dir "$TMP/t" --allowlist "$TMP/allow" 2>&1)"; rc=$?
assert_rc 0 "$rc" "an allowlisted file does not fire"
assert_has "1 acknowledged" "$out" "acknowledged entries are still counted and reported"
# a comment-only allowlist must not silence anything
printf '%s\n' '# only a comment' > "$TMP/allow2"
out="$(bash "$CHECK" --tasks-dir "$TMP/t" --allowlist "$TMP/allow2" 2>&1)"; rc=$?
assert_rc 1 "$rc" "comment-only allowlist silences nothing"

# --- 7. fail-closed ----------------------------------------------------------
out="$(bash "$CHECK" --tasks-dir "$TMP/definitely-not-here" 2>&1)"; rc=$?
assert_rc 2 "$rc" "missing tasks dir is TOOLING, never a clean census"
assert_has "TOOLING" "$out" "missing tasks dir says TOOLING"

mkcorpus
out="$(bash "$CHECK" --tasks-dir "$TMP/t" 2>&1)"; rc=$?
assert_rc 2 "$rc" "zero task files is TOOLING, never a vacuous clean"

mkcorpus; clean_task "$TMP/t/active/T-9001.md"
out="$(VERIFICATION_PORT_LIB=/nonexistent/verification-port.sh bash "$CHECK" --tasks-dir "$TMP/t" 2>&1)"; rc=$?
assert_rc 2 "$rc" "an unsourceable extractor is TOOLING, never clean"
assert_has "cannot assert what the gate would run" "$out" "names why it refuses to answer"

out="$(bash "$CHECK" --tasks-dir "$TMP/t" --bogus-flag 2>&1)"; rc=$?
assert_rc 2 "$rc" "unknown argument is TOOLING"

# --- 8. json -----------------------------------------------------------------
mkcorpus; exact_shadow_task "$TMP/t/active/T-9002.md"
out="$(bash "$CHECK" --tasks-dir "$TMP/t" --json 2>&1)"; rc=$?
assert_rc 1 "$rc" "--json preserves the exit code"
assert_has '"ok": false' "$out" "--json reports ok:false when firing"
assert_has '"firing_count": 1' "$out" "--json carries firing_count"
assert_has '"heading_count": 2' "$out" "--json carries the diagnosis"
assert_has '"scope"' "$out" "--json carries the scope disclaimer"
echo "$out" | python3 -c 'import sys,json; json.load(sys.stdin)' 2>/dev/null \
  && ok "--json emits parseable JSON" || bad "--json emits parseable JSON"

mkcorpus; clean_task "$TMP/t/active/T-9001.md"
out="$(bash "$CHECK" --tasks-dir "$TMP/t" --json 2>&1)"; rc=$?
assert_rc 0 "$rc" "--json clean exits 0"
assert_has '"ok": true' "$out" "--json clean reports ok:true"

# --- 9. guard-layer marker ---------------------------------------------------
head -3 "$CHECK" > "$TMP/hdr"
assert_has "guard-layer: source" "$(cat "$TMP/hdr")" "carries the guard-layer marker"

# --- 10. MUTANTS -------------------------------------------------------------
mut() { # <sed-expr> <label> <expected-rc-on-shadow-corpus>
  cp "$CHECK" "$TMP/mutant.sh"
  sed -i "$1" "$TMP/mutant.sh"
  if cmp -s "$CHECK" "$TMP/mutant.sh"; then bad "$2 — mutation did not apply"; return; fi
  mkcorpus; exact_shadow_task "$TMP/t/active/T-9002.md"
  env REPO_ROOT="$ROOT" ${MUT_ENV:-} bash "$TMP/mutant.sh" --tasks-dir "$TMP/t" >/dev/null 2>&1; mrc=$?
  if [ "$mrc" = "$3" ]; then ok "mutant caught: $2"; else bad "mutant SURVIVED: $2 (rc=$mrc)"; fi
}
# Disabling the prose detector must stop it firing on the real shape.
mut "s@^PROSE_RE=.*@PROSE_RE='ZZZ_NEVER_MATCHES_ZZZ'@" "prose detector disabled" 0
# Silently skipping non-empty blocks must stop it firing.
# T-3282: the default path is BATCH; skipping its recorded hits must stop it firing.
mut 's@^    \[ -n "\$hit" \] || continue@    continue@' "block scan short-circuited (batch)" 0
# ...and the per-file reference path keeps its own pin.
MUT_ENV="HEADING_SHADOW_MODE=per-file" mut 's@^    \[ -z "\$blk" \] && continue@    continue@' "block scan short-circuited (per-file)" 0

# --- 11. T-3282 batch extraction: equivalence + run-time drift guard ----------
# A corpus with the awkward shapes: counterfeit heading + orphan comment (shadow),
# a clean task, no heading at all, a Verification section running to EOF, and two
# real ## Verification sections (GNU sed ranges repeat).
mkcorpus; clean_task "$TMP/t/active/T-9001.md"; exact_shadow_task "$TMP/t/active/T-9002.md"
printf -- '---\nid: T-9003\n---\n# T-9003\n\n## Context\nno verification here\n' > "$TMP/t/completed/T-9003.md"
printf -- '---\nid: T-9004\n---\n## Verification\n\ntest -f x\n1. prose at eof\nlast line\n' > "$TMP/t/completed/T-9004.md"
printf -- '---\nid: T-9005\n---\n## Verification\ntrue\n## Notes\nx\n## Verification\n**Bold** prose\nfalse\n## End\n' > "$TMP/t/completed/T-9005.md"
out_b="$(HEADING_SHADOW_XCHECK=100 bash "$CHECK" --tasks-dir "$TMP/t" --json 2>&1)"; rc_b=$?
out_p="$(HEADING_SHADOW_MODE=per-file bash "$CHECK" --tasks-dir "$TMP/t" --json 2>&1)"; rc_p=$?
if [ "$rc_b" = "$rc_p" ] && [ "$out_b" = "$out_p" ]; then ok "batch and per-file give the identical verdict + JSON on the awkward corpus (rc=$rc_b)"
else bad "batch/per-file diverge (batch rc=$rc_b, per-file rc=$rc_p)"; fi
pout="$(bash "$ROOT/tests/verification-block-batch-proof.sh" --tasks-dir "$TMP/t" 2>&1)"; prc=$?
if [ "$prc" = "0" ] && echo "$pout" | grep -q "5 file(s) compared.*0 mismatch"; then ok "full proof: all 5 fixture files byte-identical to extract_verification_block"
else bad "fixture-corpus proof failed (rc=$prc): $pout"; fi

# A DRIFTED batch extractor must be caught by the run-time cross-check as TOOLING
# (rc 2) — never reported as a verdict. T-3370: the mutant re-opens the block on a LATER
# exact heading (drops 1.8.3's first-match-only `seen` guard); T-9005 above has two.
sed 's@^            if not seen:@            if True:@' "$ROOT/scripts/lib/verification-block-batch.py" > "$TMP/drift-batch.py"
if cmp -s "$ROOT/scripts/lib/verification-block-batch.py" "$TMP/drift-batch.py"; then bad "drift mutant did not apply"; else
  dout="$(HEADING_SHADOW_BATCH="$TMP/drift-batch.py" HEADING_SHADOW_XCHECK=100 bash "$CHECK" --tasks-dir "$TMP/t" 2>&1)"; drc=$?
  if [ "$drc" = "2" ] && echo "$dout" | grep -q "diverged from extract_verification_block"; then ok "drifted batch extractor caught by the cross-check (rc 2, no verdict)"
  else bad "drift mutant SURVIVED (rc=$drc)"; fi
fi

# Fail-open on a missing extractor must be caught.
cp "$CHECK" "$TMP/mutant.sh"
sed -i 's@|| die_tooling "extractor not found: \$EXTRACTOR (cannot assert what the gate would run)"@|| exit 0@' "$TMP/mutant.sh"
if cmp -s "$CHECK" "$TMP/mutant.sh"; then bad "fail-open mutant did not apply"; else
  mkcorpus; clean_task "$TMP/t/active/T-9001.md"
  REPO_ROOT="$ROOT" VERIFICATION_PORT_LIB=/nonexistent/x.sh bash "$TMP/mutant.sh" --tasks-dir "$TMP/t" >/dev/null 2>&1; mrc=$?
  if [ "$mrc" = "0" ]; then ok "mutant caught: fail-open on missing extractor (would report clean)"; else bad "mutant SURVIVED: fail-open (rc=$mrc)"; fi
fi

echo ""
echo "verification-heading-shadow fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
