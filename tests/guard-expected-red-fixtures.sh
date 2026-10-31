#!/usr/bin/env bash
# guard-layer: source
# guard-expected-red-fixtures.sh — T-3632 (operator ruling C on T-3580, 2026-10-31).
#
# Hermetic proof for the dated "expected red" WARN line:
#     # guard-layer-expected-red: <YYYY-MM-DD>  # <reason>
# across its four readers: the helper (scripts/guard-expected-red.sh), the release canary's WARN
# escalation, the escalation filer, and the marker-form check. Scratch member dirs, a scratch ledger,
# a scratch task corpus and a pinned "now"; nothing real is read or filed. Weighted to the cases that
# must STILL escalate (expired, malformed, over the cap, not WARN): the line may only ever hold back.
set -uo pipefail
unset CI GITHUB_ACTIONS

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 2
HELPER="$REPO_ROOT/scripts/guard-expected-red.sh"
CANARY="$REPO_ROOT/scripts/check-release-publication-freshness.sh"
FILER="$REPO_ROOT/scripts/warn-escalation-file.sh"
MARKERS="$REPO_ROOT/scripts/check-guard-severity-markers.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $1 — $2"; }
has() { grep -q -- "$1" <<< "$OUT"; }

NOW=1790000000                       # 2026-09-21T14:13:20Z
D=86400
iso() { date -u -d "@$1" +%Y-%m-%dT%H:%M:%SZ; }
day() { date -u -d "@$1" +%F; }

# Members: one per verdict, all red, all WARN unless stated.
GL="$TMP/gl"; mkdir -p "$GL/scripts" "$GL/tests"
mk() { # mk <name> <tier-word or ""> <expected-red line or "">
    { echo '#!/usr/bin/env bash'
      if [ -n "$2" ]; then echo "# guard-layer: source $2  # reason: fixture"; else echo "# guard-layer: source"; fi
      [ -n "$3" ] && echo "$3"
      echo 'echo red'; echo 'exit 1'; } > "$GL/scripts/$1"
}
mk check-active.sh   warn "# guard-layer-expected-red: $(day $((NOW + 10*D)))  # red by design until PH-2"
mk check-expired.sh  warn "# guard-layer-expected-red: $(day $((NOW - 2*D)))  # was red by design"
mk check-noreason.sh warn "# guard-layer-expected-red: $(day $((NOW + 10*D)))"
mk check-overcap.sh  warn "# guard-layer-expected-red: $(day $((NOW + 120*D)))  # far too far"
mk check-baddate.sh  warn "# guard-layer-expected-red: 2026-02-30  # no such day"
mk check-fail.sh     ""   "# guard-layer-expected-red: $(day $((NOW + 10*D)))  # but this member is FAIL"
mk check-plain.sh    warn ""
export GUARD_SEVERITY_DIRS="$GL/scripts $GL/tests" GUARD_LAYER_SCRIPTS_DIR="$GL/scripts" GUARD_LAYER_TESTS_DIR="$GL/tests"

echo "T-3632 expected-red fixtures"

echo "== helper =="
h() { OUT="$(GUARD_EXPECTED_RED_NOW="$NOW" bash "$HELPER" "$1" 2>&1)"; RC=$?; }
h check-active.sh;   [ "$RC" = 0 ] && has "^active	$(day $((NOW + 10*D)))	red by design" && ok "X1 valid, future date => active" || bad X1 "$OUT"
h check-expired.sh;  has "^expired	" && ok "X2 past date => expired" || bad X2 "$OUT"
h check-noreason.sh; has "^invalid	no reason" && ok "X3 no reason => invalid" || bad X3 "$OUT"
h check-overcap.sh;  has "^invalid	.*days ahead (cap)" && ok "X4 more than 90 days ahead => invalid" || bad X4 "$OUT"
h check-baddate.sh;  has "^invalid	date 2026-02-30" && ok "X5 a date that does not exist => invalid" || bad X5 "$OUT"
h check-fail.sh;     has "^invalid	not a WARN member" && ok "X6 on a FAIL member => invalid" || bad X6 "$OUT"
h check-plain.sh;    [ "$OUT" = none ] && ok "X7 no line => none" || bad X7 "$OUT"
h check-missing.sh;  [ "$OUT" = none ] && [ "$RC" = 0 ] && ok "X8 unknown member => none (escalates), not an error" || bad X8 "rc=$RC $OUT"
OUT="$(GUARD_EXPECTED_RED_NOW=$(( $(date -u -d "$(day $((NOW + 10*D)))" +%s) + D - 1 )) bash "$HELPER" check-active.sh)"
has "^active" && ok "X9 the stated day itself is included (last second => active)" || bad X9 "$OUT"
OUT="$(GUARD_EXPECTED_RED_NOW=$(( $(date -u -d "$(day $((NOW + 10*D)))" +%s) + D )) bash "$HELPER" check-active.sh)"
has "^expired" && ok "X10 the next day => expired" || bad X10 "$OUT"
OUT="$(cd "$TMP" && GUARD_EXPECTED_RED_NOW="$NOW" bash "$HELPER" "$GL/scripts/check-active.sh")"
has "^active" && ok "X11 an absolute path works from any directory" || bad X11 "$OUT"

echo "== release canary (escalation) =="
W="$TMP/world"; mkdir -p "$W"; echo "v9.9.9" > "$W/tag.txt"
echo '{"tag_name":"v9.9.9","draft":false,"published_at":"2026-09-20T10:00:00Z"}' > "$W/release.json"
for wf in install-check.yml doc-lint.yml; do
    echo "{\"total_count\":1,\"workflow_runs\":[{\"created_at\":\"$(iso $((NOW - 2*D)))\",\"head_sha\":\"abcdef0123456\",\"conclusion\":\"success\"}]}" > "$W/runs-$wf.json"
done
LED="$TMP/ledger"
canary() { # canary <member> [canary script]
    printf '%s  %s\n' "$1" "$(iso $((NOW - 20*D)))" > "$LED"
    OUT="$(GUARD_WARN_LEDGER="$LED" RELEASE_PUB_TEST_DIR="$W" RELEASE_PUB_TEST_NOW="$NOW" bash "${2:-$CANARY}" --no-heartbeat 2>&1)"; RC=$?
}
canary check-active.sh
[ "$RC" = 0 ] && has "expected red until $(day $((NOW + 10*D)))" && ! has "— escalated" \
    && ok "C1 active member red 20d is NOT escalated, and says until when (rc 0)" || bad C1 "rc=$RC $(head -5 <<< "$OUT")"
canary check-expired.sh;  [ "$RC" = 1 ] && has "escalated (its expected-red date .* has passed)" && ok "C2 expired => escalates, says why" || bad C2 "rc=$RC"
canary check-noreason.sh; [ "$RC" = 1 ] && has "escalated (its expected-red line is invalid: no reason" && ok "C3 no reason => escalates, says why" || bad C3 "rc=$RC"
canary check-overcap.sh;  [ "$RC" = 1 ] && has "invalid: .*days ahead" && ok "C4 over the cap => escalates" || bad C4 "rc=$RC"
canary check-plain.sh;    [ "$RC" = 1 ] && has "check-plain.sh has been red since" && ok "C5 plain WARN (no line) escalates exactly as before" || bad C5 "rc=$RC"
canary check-missing.sh;  [ "$RC" = 1 ] && ok "C6 a ledger member with no file escalates (fail-safe)" || bad C6 "rc=$RC"
MUT="$TMP/canary-mutant.sh"
sed 's/^                     active)$/                     active-MUTATED)/' "$CANARY" > "$MUT"
if cmp -s "$CANARY" "$MUT"; then bad "M1 mutant applied" "sed matched nothing"
else canary check-active.sh "$MUT"
     [ "$RC" = 1 ] && ok "M1 with the active branch removed, C1's input escalates — C1 is load-bearing" || bad M1 "rc=$RC"; fi

echo "== escalation filer =="
FAKE="$TMP/fake-create.sh"
cat > "$FAKE" <<'EOF'
name=""; while [ $# -gt 0 ]; do case "$1" in --name) name="$2"; shift 2;; *) shift;; esac; done
n=$(( $(cat "$WARN_ESC_TASKS_DIR/.ctr" 2>/dev/null || echo 9000) + 1 )); echo "$n" > "$WARN_ESC_TASKS_DIR/.ctr"
f="$WARN_ESC_TASKS_DIR/active/T-$n-fixture.md"
printf -- '---\nid: T-%s\nname: "%s"\nstatus: captured\ndate_finished:\n---\n\n## Context\n\n<!-- x -->\n\n## Acceptance Criteria\n\n### Agent\n- [ ] [First criterion]\n- [ ] [Second criterion]\n\n## Verification\n' "$n" "$name" > "$f"
echo "ID:       T-$n"; echo "File:     $f"
EOF
chmod +x "$FAKE"
filer() { # filer <member...>
    T="$TMP/tasks-$1"; rm -rf "$T"; mkdir -p "$T/active" "$T/completed"; : > "$LED"
    for m in "$@"; do printf '%s  %s\n' "$m" "$(iso $((NOW - 20*D)))" >> "$LED"; done
    OUT="$(WARN_ESC_TASKS_DIR="$T" WARN_ESC_CREATE_CMD="$FAKE" WARN_ESC_NOW="$NOW" GUARD_WARN_LEDGER="$LED" \
           WARN_ESC_PENDING_CMD=true bash "$FILER" 2>&1)"; RC=$?
}
filer check-active.sh
[ "$RC" = 0 ] && has "^SKIP	expected-red	check-active.sh" && [ -z "$(ls "$T/active")" ] \
    && ok "F1 active member: no task filed (SKIP expected-red)" || bad F1 "rc=$RC $OUT"
filer check-expired.sh;  has "^FILED	stale	check-expired.sh" && ok "F2 expired member: filed as before" || bad F2 "rc=$RC $OUT"
filer check-noreason.sh; has "^FILED	stale	check-noreason.sh" && ok "F3 invalid line: filed as before" || bad F3 "rc=$RC $OUT"
filer check-plain.sh;    has "^FILED	stale	check-plain.sh" && ok "F4 plain WARN: filed as before" || bad F4 "rc=$RC $OUT"

echo "== marker-form check =="
OUT="$(GUARD_EXPECTED_RED_NOW="$NOW" bash "$MARKERS" 2>&1)"; RC=$?
[ "$RC" = 1 ] && ok "K1 malformed / expired lines make the marker check fire (rc 1)" || bad K1 "rc=$RC"
for m in check-noreason.sh check-overcap.sh check-baddate.sh check-fail.sh; do
    has "BAD-EXPECTED-RED .*$m" && ok "K2 names $m as a bad expected-red line" || bad "K2 $m" "$(head -8 <<< "$OUT")"
done
has "EXPIRED .*check-expired.sh" && ok "K3 names the expired line" || bad K3 "$OUT"
! has "check-active.sh" && ok "K4 the valid, current line is not reported" || bad K4 "$OUT"
rm -f "$GL/scripts/check-"{expired,noreason,overcap,baddate,fail}.sh
OUT="$(GUARD_EXPECTED_RED_NOW="$NOW" bash "$MARKERS" 2>&1)"; RC=$?
[ "$RC" = 0 ] && ok "K5 only valid lines left => marker check clean" || bad K5 "rc=$RC $OUT"

echo "T-3632 expected-red fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
