#!/usr/bin/env bash
# T-3267 — fixtures for scripts/warn-escalation-file.sh and its wiring into the
# release canary (scripts/check-release-publication-freshness.sh step c2).
#
# Hermetic (PL-213): a fixture guard layer (fake WARN/FAIL members, driven through
# the REAL run-guard-layer.sh --list --json via GUARD_LAYER_SCRIPTS_DIR), a fixture
# task corpus (WARN_ESC_TASKS_DIR), a fake `fw task create` (WARN_ESC_CREATE_CMD)
# and a pinned clock (WARN_ESC_NOW / RELEASE_PUB_TEST_NOW). Never touches .tasks/.
#
# Weighted to the filing cases, each with a mutant proving the case is load-bearing.
set -uo pipefail
unset CI GITHUB_ACTIONS

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FILER="${FILER:-$REPO_ROOT/scripts/warn-escalation-file.sh}"
CANARY="$REPO_ROOT/scripts/check-release-publication-freshness.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $1 — $2"; }

NOW=1790000000
D=86400
iso() { date -u -d "@$1" +%Y-%m-%dT%H:%M:%SZ; }

# ---- fixture guard layer: 7 WARN members + 1 FAIL member -------------------
GL="$TMP/gl"; mkdir -p "$GL/scripts" "$GL/tests"
for i in 1 2 3 4 5 6 7; do
    printf '#!/usr/bin/env bash\n# guard-layer: source warn  # reason w%s: advisory by ruling\necho "w%s red output line"\nexit 1\n' "$i" "$i" > "$GL/scripts/check-w$i.sh"
done
printf '#!/usr/bin/env bash\n# guard-layer: source\necho f1\nexit 1\n' > "$GL/scripts/check-f1.sh"
export GUARD_LAYER_SCRIPTS_DIR="$GL/scripts" GUARD_LAYER_TESTS_DIR="$GL/tests"

# ---- fake `fw task create` --------------------------------------------------
FAKE="$TMP/fake-create.sh"
cat > "$FAKE" <<'EOF'
#!/usr/bin/env bash
name=""; owner=""; horizon=""
while [ $# -gt 0 ]; do case "$1" in --name) name="$2"; shift 2;; --owner) owner="$2"; shift 2;; --horizon) horizon="$2"; shift 2;; *) shift;; esac; done
n=$(( $(cat "$WARN_ESC_TASKS_DIR/.ctr" 2>/dev/null || echo 9000) + 1 )); echo "$n" > "$WARN_ESC_TASKS_DIR/.ctr"
f="$WARN_ESC_TASKS_DIR/active/T-$n-fixture.md"
printf -- '---\nid: T-%s\nname: "%s"\nstatus: captured\nowner: %s\nhorizon: %s\ndate_finished:\n---\n\n# T-%s\n\n## Context\n\n<!-- One sentence for small tasks. -->\n\n## Acceptance Criteria\n\n### Agent\n- [ ] [First criterion]\n- [ ] [Second criterion]\n\n## Verification\n' "$n" "$name" "$owner" "$horizon" "$n" > "$f"
echo "ID:       T-$n"; echo "File:     $f"
EOF
chmod +x "$FAKE"

newcorpus() { T="$TMP/tasks-$1"; mkdir -p "$T/active" "$T/completed"; LED="$TMP/led-$1"; : > "$LED"; }
red() { printf '%s  %s\n' "$1" "$(iso $((NOW - $2*D)))" >> "$LED"; }   # red <member> <days-ago>
run() { # run [filer] [now] [extra args] → OUT RC
    local f="${1:-$FILER}" n="${2:-$NOW}"; [ $# -gt 2 ] && shift 2 || set --
    OUT="$(WARN_ESC_TASKS_DIR="$T" WARN_ESC_CREATE_CMD="$FAKE" WARN_ESC_NOW="$n" GUARD_WARN_LEDGER="$LED" bash "$f" "$@" 2>&1)"; RC=$?
}
nact() { ls "$T/active/"*.md 2>/dev/null | wc -l | tr -d ' '; }
close_all() { # close_all <epoch> — move every active task to completed with date_finished
    for f in "$T/active/"*.md; do
        [ -e "$f" ] || continue
        sed -i "s/^date_finished:.*/date_finished: $(iso "$1")/; s/^status:.*/status: work-completed/" "$f"
        mv "$f" "$T/completed/"
    done
}

echo "T-3267 WARN escalation filer fixtures"

echo "== stale (per member) =="
newcorpus s1; red check-w1.sh 13; run
[ "$RC" = 0 ] && [ "$(nact)" = 0 ] && ! grep -q FILED <<< "$OUT" && ok "S1 red 13d files NOTHING" || bad "S1 13d" "rc=$RC n=$(nact) out=$OUT"
newcorpus s2; red check-w1.sh 15; run
F="$(ls "$T/active/"*.md 2>/dev/null | head -1)"
if [ "$RC" = 0 ] && [ "$(nact)" = 1 ] && grep -q $'^FILED\tstale\tcheck-w1.sh\tT-9001' <<< "$OUT"; then ok "S2 red 15d files exactly ONE task (FILED line names it)"
else bad "S2 15d files" "rc=$RC n=$(nact) out=$OUT"; fi
if [ -n "$F" ] && grep -q '<!-- warn-escalation: member=check-w1.sh -->' "$F" && grep -q "$(iso $((NOW-15*D)))" "$F" \
   && grep -q 'reason w1: advisory by ruling' "$F" && grep -q 'w1 red output line' "$F" \
   && grep -q -- '- \[ \] `check-w1.sh` is green, or the operator has reclassified it' "$F" && ! grep -q 'First criterion' "$F"
then ok "S3 the filed task carries marker, first-red date, tier reason, CURRENT output and the real AC"
else bad "S3 task body" "$(cat "$F" 2>/dev/null | head -30)"; fi
grep -q '^owner: agent' "$F" && grep -q '^horizon: now' "$F" && ok "S4 filed as owner agent, horizon now" || bad "S4 owner/horizon" "$(head -8 "$F")"
run
[ "$RC" = 0 ] && [ "$(nact)" = 1 ] && grep -q $'^OPEN\tstale\tcheck-w1.sh\tT-9001' <<< "$OUT" && ok "S5 a second run does NOT duplicate (OPEN T-9001)" || bad "S5 dedup" "n=$(nact) out=$OUT"
sed -i 's/^status:.*/status: issues/' "$F"; run
[ "$(nact)" = 1 ] && ok "S6 de-dup holds whatever the open task's status (issues)" || bad "S6 dedup any status" "n=$(nact)"

echo "== accumulation (umbrella) =="
newcorpus u1; for i in 1 2 3 4 5 6; do red check-w$i.sh 1; done; run
F="$(ls "$T/active/"*.md 2>/dev/null | head -1)"
if [ "$RC" = 0 ] && [ "$(nact)" = 1 ] && grep -q $'^FILED\tumbrella\tumbrella' <<< "$OUT" && grep -q '<!-- warn-escalation: umbrella -->' "$F" \
   && [ "$(grep -c '^- `check-w[1-6].sh` red since' "$F")" = 6 ]
then ok "U1 six red at once (1d old) files ONE umbrella naming all six, immediately"
else bad "U1 umbrella" "rc=$RC n=$(nact) out=$OUT"; fi
run; [ "$(nact)" = 1 ] && grep -q $'^OPEN\tumbrella' <<< "$OUT" && ok "U2 umbrella is not duplicated on rerun" || bad "U2 umbrella dedup" "n=$(nact) out=$OUT"
newcorpus u3; for i in 1 2 3 4 5; do red check-w$i.sh 1; done; run
[ "$RC" = 0 ] && [ "$(nact)" = 0 ] && ok "U3 five red at once files NO umbrella" || bad "U3 five" "n=$(nact) out=$OUT"
newcorpus u4; for i in 1 2 3 4 5 6; do red check-w$i.sh 20; done; run
[ "$(nact)" = 7 ] && ok "U4 six stale members file six per-member tasks + one umbrella (7)" || bad "U4 6 stale" "n=$(nact) out=$OUT"

echo "== re-file after close =="
newcorpus r1; red check-w1.sh 15; run; close_all "$NOW"
run "$FILER" "$NOW"
[ "$(nact)" = 0 ] && grep -q $'^WAIT\tstale\tcheck-w1.sh\t14' <<< "$OUT" && ok "R1 closed today and still red → no re-file; WAIT 14d" || bad "R1 wait" "n=$(nact) out=$OUT"
run "$FILER" "$((NOW + 13*D))"
[ "$(nact)" = 0 ] && ok "R2 13d after the close → still no re-file" || bad "R2 13d after close" "n=$(nact) out=$OUT"
run "$FILER" "$((NOW + 15*D))"
[ "$(nact)" = 1 ] && grep -q $'^FILED\tstale\tcheck-w1.sh' <<< "$OUT" && ok "R3 15d after the close and still red → files AGAIN (new window)" || bad "R3 refile" "n=$(nact) out=$OUT"
newcorpus r4; for i in 1 2 3 4 5 6; do red check-w$i.sh 1; done; run; close_all "$NOW"
run "$FILER" "$((NOW + 2*D))"
[ "$(nact)" = 0 ] && grep -q $'^WAIT\tumbrella' <<< "$OUT" && ok "R4 a closed umbrella waits a window before re-filing" || bad "R4 umbrella wait" "n=$(nact) out=$OUT"
run "$FILER" "$((NOW + 15*D))"
[ "$(nact)" -ge 1 ] && grep -q $'^FILED\tumbrella' <<< "$OUT" && ok "R5 still accumulated 15d after the close → umbrella files again" || bad "R5 umbrella refile" "n=$(nact) out=$OUT"

echo "== fences =="
newcorpus c1; red check-w1.sh 30
OUT="$(CI=true WARN_ESC_TASKS_DIR="$T" WARN_ESC_CREATE_CMD="$FAKE" WARN_ESC_NOW="$NOW" GUARD_WARN_LEDGER="$LED" bash "$FILER" 2>&1)"; RC=$?
[ "$RC" = 0 ] && [ "$(nact)" = 0 ] && grep -q $'^SKIP\tci' <<< "$OUT" && ok "C1 under \$CI nothing is filed (SKIP ci)" || bad "C1 CI" "rc=$RC n=$(nact) out=$OUT"
newcorpus n1; red check-f1.sh 30; run
[ "$(nact)" = 0 ] && grep -q $'^SKIP\tnot-warn\tcheck-f1.sh' <<< "$OUT" && ok "N1 a FAIL-tier member is never filed on (scope fence)" || bad "N1 not-warn" "n=$(nact) out=$OUT"
newcorpus d1; red check-w1.sh 30; run "$FILER" "$NOW" --dry-run
[ "$(nact)" = 0 ] && grep -q $'^WOULD\tstale\tcheck-w1.sh' <<< "$OUT" && ok "D1 --dry-run reports WOULD and files nothing" || bad "D1 dry" "n=$(nact) out=$OUT"
newcorpus t1; echo "check-w1.sh  not-a-date" > "$LED"; run
[ "$RC" = 2 ] && [ "$(nact)" = 0 ] && ok "T1 an unparseable ledger date is tooling (rc 2), never a silent pass" || bad "T1 bad date" "rc=$RC out=$OUT"
newcorpus t2; red check-w1.sh 30
OUT="$(WARN_ESC_TASKS_DIR="$T" WARN_ESC_CREATE_CMD=false WARN_ESC_NOW="$NOW" GUARD_WARN_LEDGER="$LED" bash "$FILER" 2>&1)"; RC=$?
[ "$RC" = 2 ] && ok "T2 a failing task create is rc 2 (loud), not a silent no-op" || bad "T2 create fail" "rc=$RC out=$OUT"

newcorpus t3; red check-w1.sh 30
OUT="$(WARN_ESC_RUNNER="$TMP/no-such-runner.sh" WARN_ESC_TASKS_DIR="$T" WARN_ESC_CREATE_CMD="$FAKE" WARN_ESC_NOW="$NOW" GUARD_WARN_LEDGER="$LED" bash "$FILER" 2>&1)"; RC=$?
[ "$RC" = 2 ] && [ "$(nact)" = 0 ] && ok "T3 an unreadable member list is rc 2 — never files without the tier check" || bad "T3 no meta" "rc=$RC n=$(nact) out=$OUT"

echo "== the canary still fires, and files on its host path =="
W="$TMP/world"; mkdir -p "$W"
echo "v9.9.9" > "$W/tag.txt"
echo '{"tag_name":"v9.9.9","draft":false,"published_at":"2026-09-20T10:00:00Z"}' > "$W/release.json"
for wf in install-check.yml doc-lint.yml; do
    echo "{\"total_count\":1,\"workflow_runs\":[{\"created_at\":\"$(iso $((NOW-D)))\",\"head_sha\":\"abcdef0\",\"conclusion\":\"success\"}]}" > "$W/runs-$wf.json"
done
canary() { # canary [canary-script] → OUT RC
    OUT="$(RELEASE_PUB_TEST_DIR="$W" RELEASE_PUB_TEST_NOW="$NOW" RELEASE_PUB_TEST_FILE_TASKS=1 WARN_ESC_FILER="$FILER" \
           WARN_ESC_TASKS_DIR="$T" WARN_ESC_CREATE_CMD="${CREATE:-$FAKE}" GUARD_WARN_LEDGER="$LED" bash "${1:-$CANARY}" --no-heartbeat 2>&1)"; RC=$?
}
newcorpus k1; red check-w1.sh 15; canary
[ "$RC" = 1 ] && grep -q "check-w1.sh has been red since" <<< "$OUT" && grep -q "filed T-9001 for check-w1.sh (stale) — UNCOMMITTED" <<< "$OUT" && [ "$(nact)" = 1 ] \
    && ok "K1 stale: canary FIRES, files the task and names it UNCOMMITTED" || bad "K1 canary stale" "rc=$RC n=$(nact) out=$OUT"
canary
[ "$RC" = 1 ] && [ "$(nact)" = 1 ] && ok "K2 next day: canary still FIRES, no second task" || bad "K2 canary dedup" "rc=$RC n=$(nact)"
newcorpus k3; for i in 1 2 3 4 5 6; do red check-w$i.sh 1; done; canary
[ "$RC" = 1 ] && grep -q "6 WARN guard members red at once (> 5)" <<< "$OUT" && grep -q "(umbrella) — UNCOMMITTED" <<< "$OUT" \
    && ok "K3 accumulation: canary FIRES and files the umbrella" || bad "K3 canary umbrella" "rc=$RC out=$OUT"
newcorpus k4; for i in 1 2 3 4 5; do red check-w$i.sh 1; done; canary
[ "$RC" = 0 ] && [ "$(nact)" = 0 ] && ok "K4 five young reds: canary healthy, nothing filed" || bad "K4 canary five" "rc=$RC out=$OUT"
newcorpus k5; red check-w1.sh 15; CREATE=false canary
[ "$RC" = 1 ] && grep -q "escalation filing FAILED" <<< "$OUT" && ok "K5 a filing failure FIRES (not exit 2) so the verdict survives" || bad "K5 filing failure" "rc=$RC out=$OUT"
newcorpus k6; red check-w1.sh 15
OUT="$(RELEASE_PUB_TEST_DIR="$W" RELEASE_PUB_TEST_NOW="$NOW" WARN_ESC_FILER="$FILER" WARN_ESC_TASKS_DIR="$T" WARN_ESC_CREATE_CMD="$FAKE" GUARD_WARN_LEDGER="$LED" bash "$CANARY" --no-heartbeat 2>&1)"; RC=$?
[ "$RC" = 1 ] && [ "$(nact)" = 0 ] && ok "K6 the test seam never files unless asked (fires, files nothing)" || bad "K6 seam" "rc=$RC n=$(nact)"

echo "== mutants: each rule removed must turn its case red =="
mutant() { # mutant <label> <file> <sed-expr> → sets M (mutant path) or fails
    M="$TMP/mut-$1.sh"; sed "$3" "$2" > "$M"
    if cmp -s "$2" "$M"; then bad "$1 mutant applied" "sed matched nothing"; return 1; fi
}
if mutant MA "$FILER" 's/    if m in open_: print("OPEN/    if False: print("OPEN/'; then
    newcorpus ma; red check-w1.sh 15; run "$M"; run "$M"
    [ "$(nact)" = 2 ] && ok "MA de-dup removed → the second run duplicates (so S5 is load-bearing)" || bad "MA" "n=$(nact)"
fi
if mutant MB "$FILER" 's/if now - start > win: print("FILE/if now - start > 0: print("FILE/'; then
    newcorpus mb; red check-w1.sh 13; run "$M"
    [ "$(nact)" = 1 ] && ok "MB threshold removed → 13d files (so S1 is load-bearing)" || bad "MB" "n=$(nact)"
fi
if mutant MC "$FILER" 's/^if len(red) > max_red:/if len(red) >= max_red:/'; then
    newcorpus mc; for i in 1 2 3 4 5; do red check-w$i.sh 1; done; run "$M"
    [ "$(nact)" = 1 ] && ok "MC > became >= → five reds file an umbrella (so U3 is load-bearing)" || bad "MC" "n=$(nact)"
fi
if mutant MD "$FILER" 's/start = max(red\[m\], closed.get(m, 0))/start = red[m]/'; then
    newcorpus md; red check-w1.sh 15; run "$M"; close_all "$NOW"; run "$M" "$NOW"
    [ "$(nact)" = 1 ] && ok "MD re-file window removed → a closed task re-files the same day (so R1 is load-bearing)" || bad "MD" "n=$(nact)"
fi
if mutant ME "$FILER" 's/^if \[ -n "\${CI:-}" \]; then/if false; then/'; then
    newcorpus me; red check-w1.sh 30
    OUT="$(CI=true WARN_ESC_TASKS_DIR="$T" WARN_ESC_CREATE_CMD="$FAKE" WARN_ESC_NOW="$NOW" GUARD_WARN_LEDGER="$LED" bash "$M" 2>&1)"
    [ "$(nact)" = 1 ] && ok "ME CI guard removed → CI files (so C1 is load-bearing)" || bad "ME" "n=$(nact)"
fi
if mutant MF "$CANARY" 's/^FILE_TASKS=1$/FILE_TASKS=0/'; then
    newcorpus mf; red check-w1.sh 15; canary "$M"
    [ "$(nact)" = 0 ] && ok "MF canary filing call disabled → nothing filed (so K1 is load-bearing)" || bad "MF" "n=$(nact)"
fi
if mutant MG "$CANARY" 's/^if \[ "\$n_red" -gt "\$WARN_MAX_RED" \]; then/if false; then/'; then
    newcorpus mg; for i in 1 2 3 4 5 6; do red check-w$i.sh 1; done; canary "$M"
    grep -q "red at once" <<< "$OUT" && bad "MG" "accumulation line still present" || ok "MG accumulation FIRING removed → K3's line disappears (so K3 is load-bearing)"
fi

echo ""
echo "warn-escalation filer fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = "0" ] || exit 1
