#!/usr/bin/env bash
# tests/woken-silent-triage.sh (T-2416) — hermetic test for the woken-but-silent
# re-verify-and-clear triage (G-085 prevention). Two layers:
#   (1) pure field-parser unit tests (cid/topic/offset/hub/ts extraction)
#   (2) end-to-end via the PL-213 seam (WOKEN_TRIAGE_CONFIRM_CMD stub returning
#       canned per-cid verdicts) — no hub, no network. Asserts:
#         - all-resolved  → live log emptied on --apply, exit 0 (canary green)
#         - mixed         → still-silent entry KEPT, resolved archived, exit 1
#         - report mode   → non-mutating (log byte-identical)
#         - malformed     → kept, never silently dropped
#         - --hub entry   → forwards --hub to the matcher
#         - inconclusive  → kept (never clear what we could not re-verify)
#         - empty log     → healthy exit 0
set -u
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SELF_DIR/.."
SCRIPT="$ROOT/scripts/woken-silent-triage.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
fails=0
pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1"; fails=$((fails+1)); }

# ── (1) pure field-parser unit tests ─────────────────────────────────────────
WOKEN_TRIAGE_LIB=1 . "$SCRIPT"

BLK_LOCAL='=== 2026-07-17T10:59:55Z ===
woken-but-silent: no receipt for cid=cid-1784285903-5798 on topic=dm:0e7ee6ca:d1993c2c
  recipient=aef session=aef
  turn posted at offset=1; rings=3; reason=direct-path: rung 3 x, receiver never acked
  remediation: confirm peer LIVE (/peers --all); re-send; or drop the thread if dead
---'
BLK_HUB='=== 2026-07-12T19:34:04Z ===
woken-but-silent: no receipt for cid=cid-1783884811-24990 on topic=dm:88743a9a:d1993c2c
  recipient=ring20-concierge session=claude-master hub=192.168.10.122:9100
  turn posted at offset=0; rings=3; reason=direct-path: rung 3 x, receiver never acked
  remediation: confirm peer LIVE (/peers --all); re-send; or drop the thread if dead
---'

[ "$(wst_field_cid "$BLK_LOCAL")" = "cid-1784285903-5798" ] && pass "parse cid" || fail "parse cid got '$(wst_field_cid "$BLK_LOCAL")'"
[ "$(wst_field_topic "$BLK_LOCAL")" = "dm:0e7ee6ca:d1993c2c" ] && pass "parse topic" || fail "parse topic got '$(wst_field_topic "$BLK_LOCAL")'"
[ "$(wst_field_offset "$BLK_LOCAL")" = "1" ] && pass "parse offset" || fail "parse offset got '$(wst_field_offset "$BLK_LOCAL")'"
[ -z "$(wst_field_hub "$BLK_LOCAL")" ] && pass "no hub on local entry" || fail "local entry had spurious hub"
[ "$(wst_field_hub "$BLK_HUB")" = "192.168.10.122:9100" ] && pass "parse hub" || fail "parse hub got '$(wst_field_hub "$BLK_HUB")'"
[ "$(wst_field_ts "$BLK_HUB")" = "2026-07-12T19:34:04Z" ] && pass "parse ts" || fail "parse ts got '$(wst_field_ts "$BLK_HUB")'"
wst_entry_valid "$BLK_LOCAL" && pass "valid entry passes" || fail "valid entry rejected"
wst_entry_valid "garbage no fields" && fail "garbage accepted as valid" || pass "garbage entry rejected"

# ── (2) e2e via the PL-213 confirm-command stub ──────────────────────────────
# The stub maps cid -> exit code (0 consumed / 3 silent / 2 inconclusive) and,
# when consumed, prints a wake-confirm-shaped JSON with a reply offset. It also
# records whether it was called with --hub for the hub-forwarding assertion.
STUB="$TMP/confirm-stub.sh"
cat > "$STUB" <<'STUBEOF'
#!/usr/bin/env bash
cid=""; hub=""
while [ $# -gt 0 ]; do case "$1" in
  --cid) cid="$2"; shift 2;; --hub) hub="$2"; shift 2;;
  --topic|--since-offset|--timeout) shift 2;; *) shift;; esac; done
[ -n "$hub" ] && echo "$cid $hub" >> "$STUB_HUB_LOG"
rc="$(cat "$STUB_DIR/$cid.rc" 2>/dev/null || echo 3)"
[ "$rc" = 0 ] && printf '{"consumed":true,"receipt_offset":%s}\n' "$(cat "$STUB_DIR/$cid.off" 2>/dev/null || echo 0)"
exit "$rc"
STUBEOF
chmod +x "$STUB"

mkfix() { printf '%s' "$2" > "$TMP/fix/$1.rc"; [ -n "${3:-}" ] && printf '%s' "$3" > "$TMP/fix/$1.off"; }
frame() { # cid topic offset [hub]
    printf '=== 2026-07-17T00:00:00Z ===\n'
    printf 'woken-but-silent: no receipt for cid=%s on topic=%s\n' "$1" "$2"
    printf '  recipient=peer session=peer%s\n' "${4:+ hub=$4}"
    printf '  turn posted at offset=%s; rings=3; reason=test\n' "$3"
    printf '  remediation: ...\n---\n'
}

run() { # logfile extra-args...  → sets RUN_OUT RUN_RC
    local lf="$1"; shift
    RUN_OUT=$(env WOKEN_TRIAGE_CONFIRM_CMD="bash $STUB" \
                  STUB_DIR="$TMP/fix" STUB_HUB_LOG="$TMP/hub.log" \
                  TERMLINK_WOKEN_SILENT_LOG="$lf" \
                  bash "$SCRIPT" --no-heartbeat "$@" 2>/dev/null); RUN_RC=$?
}

# case A: all-resolved → --apply empties the live log, exit 0
mkdir -p "$TMP/fix"; : > "$TMP/hub.log"
mkfix all-a 0 11; mkfix all-b 0 22
LOG_A="$TMP/a.log"; { frame all-a dm:x:y 10; frame all-b dm:p:q 21; } > "$LOG_A"
run "$LOG_A" --json --apply
echo "$RUN_OUT" | jq -e '.summary.resolved==2 and .summary.still_silent==0 and .ok==true' >/dev/null 2>&1 \
    && pass "A: all-resolved summary ok" || fail "A: summary wrong: $RUN_OUT"
[ "$RUN_RC" -eq 0 ] && pass "A: exit 0 when all resolved" || fail "A: expected exit 0 got $RUN_RC"
[ ! -s "$LOG_A" ] && pass "A: --apply emptied the live log (canary green)" || fail "A: live log not emptied: $(cat "$LOG_A")"
[ -s "${LOG_A%.log}.resolved.log" ] && pass "A: resolved archive written" || fail "A: no resolved archive"
grep -q "all-a" "${LOG_A%.log}.resolved.log" && pass "A: archive names the cleared cid" || fail "A: archive missing cid"

# case B: mixed → still-silent kept, resolved archived, exit 1
rm -f "$TMP"/fix/*; : > "$TMP/hub.log"
mkfix keep-me 3; mkfix clear-me 0 42
LOG_B="$TMP/b.log"; { frame keep-me dm:s:t 5; frame clear-me dm:u:v 7; } > "$LOG_B"
run "$LOG_B" --json --apply
echo "$RUN_OUT" | jq -e '.summary.resolved==1 and .summary.still_silent==1 and .ok==false' >/dev/null 2>&1 \
    && pass "B: mixed summary ok" || fail "B: summary wrong: $RUN_OUT"
[ "$RUN_RC" -eq 1 ] && pass "B: exit 1 when a silent entry remains" || fail "B: expected exit 1 got $RUN_RC"
grep -q "keep-me" "$LOG_B" && pass "B: still-silent entry KEPT in live log" || fail "B: silent entry lost"
grep -q "clear-me" "$LOG_B" && fail "B: resolved entry NOT removed" || pass "B: resolved entry removed from live log"

# case C: report-mode (no --apply) is non-mutating
rm -f "$TMP"/fix/*; mkfix r-a 0 1; mkfix r-b 0 2
LOG_C="$TMP/c.log"; { frame r-a dm:a:b 3; frame r-b dm:c:d 4; } > "$LOG_C"
before="$(md5sum < "$LOG_C")"
run "$LOG_C"     # no --apply
after="$(md5sum < "$LOG_C")"
[ "$before" = "$after" ] && pass "C: report-mode leaves live log byte-identical" || fail "C: report-mode mutated the log"
[ ! -e "${LOG_C%.log}.resolved.log" ] && pass "C: report-mode wrote no archive" || fail "C: report-mode wrote an archive"

# case D: malformed entry kept, never dropped
rm -f "$TMP"/fix/*; mkfix good 0 9
LOG_D="$TMP/d.log"; { printf '=== 2026-07-17T00:00:00Z ===\ngarbage line with no fields\n---\n'; frame good dm:g:h 8; } > "$LOG_D"
run "$LOG_D" --json --apply
echo "$RUN_OUT" | jq -e '.summary.inconclusive>=1' >/dev/null 2>&1 \
    && pass "D: malformed counted inconclusive" || fail "D: malformed not counted: $RUN_OUT"
grep -q "garbage line" "$LOG_D" && pass "D: malformed entry KEPT" || fail "D: malformed entry dropped"

# case E: --hub forwarded to the matcher
rm -f "$TMP"/fix/*; : > "$TMP/hub.log"; mkfix hub-cid 0 1
LOG_E="$TMP/e.log"; frame hub-cid dm:h:i 0 192.168.10.122:9100 > "$LOG_E"
run "$LOG_E" --apply
grep -q "hub-cid 192.168.10.122:9100" "$TMP/hub.log" && pass "E: --hub forwarded to matcher" || fail "E: --hub not forwarded (log: $(cat "$TMP/hub.log"))"

# case F: inconclusive (rc=2) kept, exit 1
rm -f "$TMP"/fix/*; mkfix inc 2
LOG_F="$TMP/f.log"; frame inc dm:j:k 0 > "$LOG_F"
run "$LOG_F" --json --apply
echo "$RUN_OUT" | jq -e '.summary.inconclusive==1 and .ok==false' >/dev/null 2>&1 \
    && pass "F: inconclusive kept, not-ok" || fail "F: inconclusive wrong: $RUN_OUT"
grep -q "inc" "$LOG_F" && pass "F: inconclusive entry KEPT (not re-verifiable → not cleared)" || fail "F: inconclusive entry cleared"

# case G: empty log → healthy exit 0
LOG_G="$TMP/g.log"; : > "$LOG_G"
run "$LOG_G" --json
echo "$RUN_OUT" | jq -e '.ok==true and .summary.total==0' >/dev/null 2>&1 \
    && pass "G: empty log healthy" || fail "G: empty log wrong: $RUN_OUT"
[ "$RUN_RC" -eq 0 ] && pass "G: empty log exit 0" || fail "G: empty log expected exit 0 got $RUN_RC"

# case H: heartbeat/mtime ordering — a KEPT still-silent entry must leave the
# log NEWER than the heartbeat (else canary-status masks it as HEALTHY). Runs
# WITHOUT --no-heartbeat so the real heartbeat logic exercises.
rm -f "$TMP"/fix/*; mkfix h-keep 3
LOG_H="$TMP/h.log"; HB_H="$TMP/h.heartbeat"; frame h-keep dm:h:h 0 > "$LOG_H"
env WOKEN_TRIAGE_CONFIRM_CMD="bash $STUB" STUB_DIR="$TMP/fix" STUB_HUB_LOG="$TMP/hub.log" \
    TERMLINK_WOKEN_SILENT_LOG="$LOG_H" bash "$SCRIPT" --apply --quiet >/dev/null 2>&1
if [ -f "$HB_H" ] && [ -s "$LOG_H" ]; then
    lm=$(stat -c %Y "$LOG_H"); hm=$(stat -c %Y "$HB_H")
    [ "$lm" -gt "$hm" ] && pass "H: kept entry leaves log NEWER than heartbeat (stays FIRING)" \
                        || fail "H: log($lm) not newer than heartbeat($hm) — would mask firing"
else
    fail "H: expected non-empty log + heartbeat after --apply with a kept entry"
fi
# case I: all-cleared leaves an EMPTY log + fresh heartbeat (canary HEALTHY)
rm -f "$TMP"/fix/*; mkfix i-clear 0 5
LOG_I="$TMP/i.log"; HB_I="$TMP/i.heartbeat"; frame i-clear dm:i:i 0 > "$LOG_I"
env WOKEN_TRIAGE_CONFIRM_CMD="bash $STUB" STUB_DIR="$TMP/fix" STUB_HUB_LOG="$TMP/hub.log" \
    TERMLINK_WOKEN_SILENT_LOG="$LOG_I" bash "$SCRIPT" --apply --quiet >/dev/null 2>&1
[ ! -s "$LOG_I" ] && [ -f "$HB_I" ] && pass "I: all-cleared → empty log + heartbeat (HEALTHY)" \
                                    || fail "I: expected empty log + heartbeat"
# case J: report mode touches NO heartbeat (non-mutating preview)
rm -f "$TMP"/fix/*; mkfix j 0 1
LOG_J="$TMP/j.log"; HB_J="$TMP/j.heartbeat"; frame j dm:j:j 0 > "$LOG_J"
env WOKEN_TRIAGE_CONFIRM_CMD="bash $STUB" STUB_DIR="$TMP/fix" \
    TERMLINK_WOKEN_SILENT_LOG="$LOG_J" bash "$SCRIPT" --quiet >/dev/null 2>&1
[ ! -e "$HB_J" ] && pass "J: report mode writes no heartbeat" || fail "J: report mode touched heartbeat"

# ── syntax ───────────────────────────────────────────────────────────────────
bash -n "$SCRIPT" 2>/dev/null && pass "bash -n scripts/woken-silent-triage.sh clean" || fail "bash -n triage FAILED"
bash -n "$ROOT/scripts/wake-confirm.sh" 2>/dev/null && pass "bash -n scripts/wake-confirm.sh clean (untouched)" || fail "wake-confirm regressed"

echo ""
if [ "$fails" -eq 0 ]; then echo "woken-silent-triage: ALL PASS"; exit 0
else echo "woken-silent-triage: $fails FAIL"; exit 1; fi
