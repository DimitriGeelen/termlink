#!/usr/bin/env bash
# T-2985 — fixtures for scripts/canary-escalation-file.sh. Hermetic: a canned
# `canary-status --json` (CANARY_ESC_STATUS_CMD), a scratch task corpus (--tasks-dir), a
# fake `fw task create` (CANARY_ESC_CREATE_CMD) and a fixed clock (CANARY_ESC_NOW). Never
# touches .tasks/, the pending-commit manifest or any real canary.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FILER="$ROOT/scripts/canary-escalation-file.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
pass=0; fail=0
ok()  { echo "ok   $1"; pass=$((pass+1)); }
bad() { echo "FAIL $1"; fail=$((fail+1)); }
NOW=1792800000; H=3600
iso() { date -u -d "@$1" +%Y-%m-%dT%H:%M:%SZ; }

FAKE="$TMP/fake-create.sh"
cat > "$FAKE" <<'EOF'
#!/usr/bin/env bash
name=""
while [ $# -gt 0 ]; do case "$1" in --name) name="$2"; shift 2;; *) shift;; esac; done
n=$(( $(cat "$CE_T/.ctr" 2>/dev/null || echo 9000) + 1 )); echo "$n" > "$CE_T/.ctr"
f="$CE_T/active/T-$n-fixture.md"
printf -- '---\nid: T-%s\nname: "%s"\nstatus: captured\ndate_finished:\n---\n\n# T-%s\n\n## Context\n\n<!-- One sentence. -->\n\n## Acceptance Criteria\n\n### Agent\n- [ ] [First criterion]\n- [ ] [Second criterion]\n\n## Verification\n' "$n" "$name" "$n" > "$f"
echo "ID:       T-$n"; echo "File:     $f"
EOF
chmod +x "$FAKE"

# status <name:STATUS>... → a canary-status --json document
status() {
    local items="" sep="" a n s
    for a in "$@"; do n="${a%%:*}"; s="${a#*:}"
        items="$items$sep{\"name\":\"$n\",\"status\":\"$s\",\"latest_entry\":\"signal line of $n\"}"; sep=","; done
    printf '{"ok":false,"canaries":[%s]}\n' "$items" > "$TMP/status.json"
}
newcorpus() { T="$TMP/tasks-$1"; mkdir -p "$T/active" "$T/completed"; }
run() { # run [filer] [extra args] → OUT RC
    local f="${1:-$FILER}"; [ $# -gt 0 ] && shift
    OUT="$(CE_T="$T" CANARY_ESC_STATUS_CMD="cat $TMP/status.json" CANARY_ESC_CREATE_CMD="$FAKE" \
        CANARY_ESC_PENDING_CMD=true CANARY_ESC_NOW="$NOW" bash "$f" --tasks-dir "$T" "$@" 2>&1)"; RC=$?
}
closed_task() { # closed_task <canary> <hours-ago>
    printf -- '---\nid: T-8001\nstatus: work-completed\ndate_finished: %s\n---\n<!-- canary-escalation: canary=%s -->\n' \
        "$(iso $((NOW - $2*H)))" "$1" > "$T/completed/T-8001-old.md"
}

# 1. FIRING, ERRORING and STALE each file one task; HEALTHY / NOT_SCHEDULED never do.
newcorpus c1; status dead-letter-canary:FIRING x-canary:ERRORING y-canary:STALE ok-canary:HEALTHY ns-canary:NOT_SCHEDULED nh-canary:NO_HEARTBEAT
run
n_filed=$(grep -c '^FILED' <<< "$OUT")
[ "$RC" = 0 ] && [ "$n_filed" = 3 ] && ! grep -qE 'ok-canary|ns-canary|nh-canary' <<< "$OUT" \
    && ok "1 FIRING/ERRORING/STALE file one task each; HEALTHY/NOT_SCHEDULED/NO_HEARTBEAT ignored" || bad "1 rc=$RC: $OUT"

# 2. The filed task carries the marker, the signal line and a real criterion, not the placeholders.
F="$(grep -l 'canary=dead-letter-canary' "$T"/active/*.md | head -1)"
[ -n "$F" ] && grep -q '<!-- canary-escalation: canary=dead-letter-canary -->' "$F" && grep -q 'signal line of dead-letter-canary' "$F" \
    && grep -q -- '- \[ \] `dead-letter-canary` reports HEALTHY' "$F" && ! grep -q 'First criterion' "$F" \
    && ok "2 task body has marker, signal line and criterion" || bad "2 body: $(cat "$F" 2>/dev/null | head -30)"

# 3. De-dup: a second run while the tasks are open files nothing and reports OPEN.
run
[ "$RC" = 0 ] && ! grep -q '^FILED' <<< "$OUT" && [ "$(grep -c '^OPEN' <<< "$OUT")" = 3 ] \
    && [ "$(ls "$T"/active/*.md | wc -l)" = 3 ] && ok "3 second run while open files nothing (OPEN x3)" || bad "3 rc=$RC: $OUT"

# 4. Closed 2h ago and still firing: WAIT inside the 24h window.
newcorpus c4; status dead-letter-canary:FIRING; closed_task dead-letter-canary 2
run
grep -qE '^WAIT	FIRING	dead-letter-canary	22$' <<< "$OUT" && ! grep -q '^FILED' <<< "$OUT" \
    && ok "4 closed 2h ago: WAIT 22h, nothing filed" || bad "4 rc=$RC: $OUT"

# 5. Closed 30h ago and still firing: files again.
newcorpus c5; status dead-letter-canary:FIRING; closed_task dead-letter-canary 30
run
grep -q '^FILED	FIRING	dead-letter-canary	T-' <<< "$OUT" && ok "5 closed 30h ago and still bad: files again" || bad "5 rc=$RC: $OUT"

# 6. --dry-run reports and writes nothing.
newcorpus c6; status dead-letter-canary:FIRING
run "$FILER" --dry-run
grep -q '^WOULD	FIRING	dead-letter-canary' <<< "$OUT" && [ -z "$(ls "$T"/active)" ] \
    && ok "6 --dry-run prints WOULD and writes nothing" || bad "6 rc=$RC: $OUT"

# 7. Under $CI it refuses with SKIP, exit 0, and files nothing.
newcorpus c7; status dead-letter-canary:FIRING
OUT="$(CI=true CE_T="$T" CANARY_ESC_STATUS_CMD="cat $TMP/status.json" CANARY_ESC_CREATE_CMD="$FAKE" CANARY_ESC_PENDING_CMD=true \
    CANARY_ESC_NOW="$NOW" bash "$FILER" --tasks-dir "$T" 2>&1)"; RC=$?
[ "$RC" = 0 ] && grep -q '^SKIP	ci' <<< "$OUT" && [ -z "$(ls "$T"/active)" ] && ok "7 CI refuses (SKIP, exit 0)" || bad "7 rc=$RC: $OUT"

# 8. Unreadable canary status is a tooling error (exit 2), never a silent clean.
newcorpus c8; echo 'not json' > "$TMP/status.json"
run
[ "$RC" = 2 ] && grep -q 'cannot read canary status' <<< "$OUT" && ok "8 unreadable status exits 2" || bad "8 rc=$RC: $OUT"

# 9. A missing tasks dir is a tooling error (exit 2).
status dead-letter-canary:FIRING; T="$TMP/no-such-dir"
run
[ "$RC" = 2 ] && ok "9 missing tasks dir exits 2" || bad "9 rc=$RC: $OUT"

# 10. A task create failure is exit 2 and loud.
newcorpus c10; status dead-letter-canary:FIRING
OUT="$(CE_T="$T" CANARY_ESC_STATUS_CMD="cat $TMP/status.json" CANARY_ESC_CREATE_CMD=false CANARY_ESC_PENDING_CMD=true \
    CANARY_ESC_NOW="$NOW" bash "$FILER" --tasks-dir "$T" 2>&1)"; RC=$?
[ "$RC" = 2 ] && grep -q 'task create failed' <<< "$OUT" && ok "10 create failure exits 2" || bad "10 rc=$RC: $OUT"

# M1. Mutant: drop the open-task check. Case 3 must then file duplicates, so de-dup is load-bearing.
M="$TMP/mut-nodedup.sh"
sed 's/    if name in open_: print("OPEN/    if False: print("OPEN/' "$FILER" > "$M"
grep -q 'if False: print("OPEN' "$M" || bad "M1 mutant did not apply"
newcorpus m1; status dead-letter-canary:FIRING
run "$M"; run "$M"
[ "$(ls "$T"/active/*.md | wc -l)" -ge 2 ] && ok "M1 without the open-task check a second run files a duplicate (de-dup is load-bearing)" \
    || bad "M1 mutant still de-duplicated: $OUT"

echo "---"
echo "canary-escalation-file fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
