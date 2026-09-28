#!/usr/bin/env bash
# guard-layer: source
#
# T-3205 — fixtures for the stale-sidecar-code detector.
#
# The failure it guards is silent by construction: a long-lived process keeps executing
# the code it loaded, the supervisor never restarts a live sidecar, and every surface
# reports healthy. Measured in T-3204 — a fix committed, tested and pushed sat dark for
# hours. So these weight the FIRING cases and the fail-closed paths: a detector that
# cannot go red is not a detector.
set -uo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CHECK="$PROJECT_ROOT/scripts/check-stale-sidecar-code.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }

[ -r "$CHECK" ] || { echo "TOOLING: cannot read $CHECK" >&2; exit 2; }

TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

# Reference script with a KNOWN mtime. Everything is judged against this.
REF="$TMP/notify-sidecar.sh"; : > "$REF"
touch -d '2026-09-28 12:00:00' "$REF"
REF_EPOCH=$(stat -c %Y "$REF")
OLDER=$((REF_EPOCH - 3600))   # process started an hour BEFORE the code changed
NEWER=$((REF_EPOCH + 3600))   # process started an hour after

SIB="$PROJECT_ROOT/scripts/check-stale-waker-code-freshness.sh"

run() {  # run <procs-file> [extra args...]
    local pf="$1"; shift
    SIDECAR_STALE_TEST_PROCS="$pf" SIDECAR_STALE_SCRIPT="$REF" \
    SIDECAR_STALE_SIBLING="${SIB_OVERRIDE:-$SIB}" \
        bash "$CHECK" --no-heartbeat "$@" > "$TMP/out.txt" 2> "$TMP/err.txt"
    RC=$?
}

echo "case 1: a process older than the script FIRES"
printf '111\t%s\tbash /opt/x/notify-sidecar.sh --agent-id alpha\n' "$OLDER" > "$TMP/p1"
run "$TMP/p1"
[ "$RC" -eq 1 ] && ok "exit 1 on stale" || bad "expected exit 1, got $RC"
grep -q 'FIRING' "$TMP/out.txt" && ok "says FIRING" || bad "no FIRING line"
grep -q 'agent=alpha' "$TMP/out.txt" && ok "names the agent" || bad "does not name the agent"

echo "case 2: a process newer than the script is CURRENT"
printf '222\t%s\tbash /opt/x/notify-sidecar.sh --agent-id beta\n' "$NEWER" > "$TMP/p2"
run "$TMP/p2"
[ "$RC" -eq 0 ] && ok "exit 0 on current" || bad "expected exit 0, got $RC ($(head -2 "$TMP/out.txt" "$TMP/err.txt"))"
grep -q 'healthy' "$TMP/out.txt" && ok "reports healthy" || bad "no healthy line"

echo "case 3: mixed — one stale among current still fires"
{ printf '222\t%s\tbash /opt/x/notify-sidecar.sh --agent-id beta\n' "$NEWER"
  printf '111\t%s\tbash /opt/x/notify-sidecar.sh --agent-id alpha\n' "$OLDER"; } > "$TMP/p3"
run "$TMP/p3"
[ "$RC" -eq 1 ] && ok "one stale among current fires" || bad "mixed case did not fire (rc=$RC)"

echo "case 4: pid-recycle guard — an old pid that is NOT a sidecar is not reported"
printf '333\t%s\t/usr/bin/some-other-daemon --agent-id gamma\n' "$OLDER" > "$TMP/p4"
run "$TMP/p4"
[ "$RC" -eq 0 ] && ok "recycled pid does not fire" || bad "reported a non-sidecar as stale"
grep -q 'gamma' "$TMP/out.txt" && bad "named an unrelated process" || ok "does not name the unrelated process"

echo "case 5: a pid that vanished mid-scan (no start mtime) is skipped, not judged"
printf '444\t\tbash /opt/x/notify-sidecar.sh --agent-id delta\n' > "$TMP/p5"
run "$TMP/p5"
[ "$RC" -eq 0 ] && ok "vanished pid does not fire" || bad "invented a verdict for a gone process"

echo "case 6: FAIL-CLOSED — sibling detector missing exits 2, never 0"
SIB_OVERRIDE="$TMP/nope.sh" run "$TMP/p2"
[ "$RC" -eq 2 ] && ok "missing sibling exits 2" || bad "expected 2, got $RC"
grep -q 'refusing to reimplement' "$TMP/err.txt" && ok "says why" || bad "silent about the refusal"

echo "case 7: FAIL-CLOSED — sibling present but primitives renamed exits 2"
printf '#!/usr/bin/env bash\nsomething_else() { :; }\n' > "$TMP/sib-bad.sh"
SIB_OVERRIDE="$TMP/sib-bad.sh" run "$TMP/p2"
[ "$RC" -eq 2 ] && ok "renamed primitives exit 2" || bad "expected 2, got $RC"

echo "case 8: FAIL-CLOSED — a sibling whose is_stale is WRONG is caught behaviourally"
# Existence is not correctness. The first draft of the real script extracted three
# functions with one awk, producing nested definitions that eval'd fine and passed a
# declare -F check while classifying everything wrongly. This pins the self-test.
cat > "$TMP/sib-wrong.sh" <<'EOS'
#!/usr/bin/env bash
code_mtime() { stat -c %Y "$1" 2>/dev/null || echo 0; }
proc_start_mtime() { stat -c %Y "/proc/$1" 2>/dev/null || true; }
is_stale() {
    return 0
}
EOS
SIB_OVERRIDE="$TMP/sib-wrong.sh" run "$TMP/p2"
[ "$RC" -eq 2 ] && ok "a wrong-but-present is_stale exits 2" || bad "accepted broken primitive (rc=$RC)"
grep -q 'self-test' "$TMP/err.txt" && ok "names the self-test" || bad "no self-test message"

echo "case 9: FAIL-CLOSED — missing reference script exits 2"
SIDECAR_STALE_TEST_PROCS="$TMP/p2" SIDECAR_STALE_SCRIPT="$TMP/absent.sh" \
    bash "$CHECK" --no-heartbeat > "$TMP/out.txt" 2> "$TMP/err.txt"; RC=$?
[ "$RC" -eq 2 ] && ok "missing reference exits 2" || bad "expected 2, got $RC"

echo "case 10: JSON envelope carries the counts and the stale entries"
run "$TMP/p3" --json
if command -v jq >/dev/null 2>&1; then
    jq -e '.ok == false and .stale_count == 1 and .current_count == 1' "$TMP/out.txt" >/dev/null 2>&1 \
      && ok "json counts correct" || bad "json wrong: $(cat "$TMP/out.txt")"
    jq -e '.stale[0].agent_id == "alpha"' "$TMP/out.txt" >/dev/null 2>&1 \
      && ok "json names the stale agent" || bad "json missing stale agent"
else
    ok "jq absent — json assertions skipped"; ok "jq absent — json assertions skipped"
fi

echo "case 11: DETECTION ONLY — the checker never restarts anything"
# A pkill/supervisor invocation may appear in REMEDIATION TEXT, but must never be an
# executable line. T-2943: prose about a thing must not be mistaken for the thing.
if grep -nE '^[^#]*(pkill|kill -TERM|notify-sidecar-supervisor\.sh)' "$CHECK" | grep -vE 'echo|printf' >/dev/null 2>&1; then
    bad "checker contains an executable restart — it must only detect"
else
    ok "no executable restart in the checker"
fi

echo ""
echo "passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
