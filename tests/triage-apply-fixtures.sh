#!/usr/bin/env bash
# T-3649 — hermetic fixtures for scripts/triage-apply.sh.
# Temp dirs and a fake fw only; never a real task file, never the real fw.
# Run: bash tests/triage-apply-fixtures.sh

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/triage-apply.sh"
WORK="$(mktemp -d "${TMPDIR:-/var/tmp}/triage-apply-fx.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

PASS=0
FAIL=0
ok()   { PASS=$((PASS + 1)); echo "ok   $1"; }
bad()  { FAIL=$((FAIL + 1)); echo "FAIL $1"; [ -n "${2:-}" ] && sed 's/^/     /' "$2" | head -20; }

# Fake fw: logs argv, exits FAKE_FW_RC_<verb> (close/horizon), default 0.
FW="$WORK/fw"
cat > "$FW" <<'EOF'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_FW_LOG"
case "$*" in
  *work-completed*) [ "${FAKE_FW_CLOSE_RC:-0}" = 0 ] || { echo "ERROR: Verification gate failed"; echo "  detail line"; exit "$FAKE_FW_CLOSE_RC"; } ;;
  *--horizon*)      [ "${FAKE_FW_HORIZON_RC:-0}" = 0 ] || { echo "no such task" >&2; exit "$FAKE_FW_HORIZON_RC"; } ;;
esac
exit 0
EOF
chmod +x "$FW"

setup() {  # $1 = case name
    local d="$WORK/$1"
    mkdir -p "$d/triage" "$d/tasks/completed"
    cat > "$d/triage/batch.json" <<'EOF'
{"batch": "b1", "rows": [
 {"task": "T-1", "title": "a", "proposed": "keep"},
 {"task": "T-2", "title": "b", "proposed": "park"},
 {"task": "T-3", "title": "c", "proposed": "close"},
 {"task": "T-4", "title": "d", "proposed": "drop"},
 {"task": "T-5", "title": "e", "proposed": "more"},
 {"task": "T-6", "title": "f", "proposed": "prioritize"},
 {"task": "T-7", "title": "g", "proposed": "keep"}
]}
EOF
    cat > "$d/triage/answers.jsonl" <<'EOF'
{"ts": "t", "batch": "b1", "task": "T-1", "choice": "keep"}
{"ts": "t", "batch": "b1", "task": "T-2", "choice": "prioritize"}
{"ts": "t", "batch": "b1", "task": "T-2", "choice": "park"}
{"ts": "t", "batch": "b1", "task": "T-3", "choice": "close"}
{"ts": "t", "batch": "b1", "task": "T-4", "choice": "drop"}
{"ts": "t", "batch": "b1", "task": "T-5", "choice": "more"}
{"ts": "t", "batch": "b1", "task": "T-6", "choice": "prioritize"}
{"ts": "t", "batch": "old", "task": "T-7", "choice": "close"}
not json
{"ts": "t", "batch": "b1", "task": "T-99", "choice": "close"}
EOF
    : > "$d/fw.log"
    echo "$d"
}

run() {  # $1 = case dir, rest = args
    local d="$1"; shift
    TRIAGE_DIR="$d/triage" TRIAGE_FW="$FW" TRIAGE_TASKS_DIR="$d/tasks" FAKE_FW_LOG="$d/fw.log" \
        bash "$SCRIPT" "$@" > "$d/out" 2>&1
}

# 1. Dry run is the default: prints the plan, calls nothing, writes nothing.
d=$(setup dry)
run "$d"; rc=$?
if [ "$rc" = 0 ] && grep -q "WOULD.*T-2.*park.*--horizon later" "$d/out" \
   && grep -q "WOULD.*T-6.*--horizon now" "$d/out" && grep -q "WOULD.*T-3.*work-completed" "$d/out" \
   && [ ! -s "$d/fw.log" ] && [ ! -e "$d/triage/applied.jsonl" ] && [ ! -e "$d/triage/drop-pending.jsonl" ]; then
    ok "dry-run default prints plan, touches nothing"
else bad "dry-run default" "$d/out"; fi

# 2. Only this batch's answers, last line wins, junk and unknown tasks ignored.
if grep -q "6 answered, 1 unanswered" "$d/out" && ! grep -q "T-7\|T-99" "$d/out" \
   && ! grep -q "T-2 *prioritize" "$d/out"; then
    ok "last answer wins; other batch, bad lines, unknown tasks ignored"
else bad "answer selection" "$d/out"; fi

# 3. --apply maps each choice; never --force.
d=$(setup apply)
run "$d" --apply; rc=$?
if [ "$rc" = 0 ] \
   && grep -qx "task update T-2 --horizon later" "$d/fw.log" \
   && grep -qx "task update T-6 --horizon now" "$d/fw.log" \
   && grep -qx "task update T-3 --status work-completed" "$d/fw.log" \
   && [ "$(wc -l < "$d/fw.log")" = 3 ] && ! grep -q -- "--force" "$d/fw.log"; then
    ok "--apply: prioritize/park/close mapped, keep/drop/more call no fw, no --force"
else bad "--apply mapping (rc=$rc)" "$d/fw.log"; fi

# 4. Drop does not delete or close: it is listed for the operator.
if grep -q '"task": "T-4"' "$d/triage/drop-pending.jsonl" && grep -q '"task": "T-5"' "$d/triage/more-pending.jsonl" \
   && ! grep -q "T-4" "$d/fw.log"; then
    ok "drop -> drop-pending.jsonl, more -> more-pending.jsonl"
else bad "drop/more lists" "$d/out"; fi

# 5. Idempotent: a second --apply does nothing.
: > "$d/fw.log"
run "$d" --apply; rc=$?
if [ "$rc" = 0 ] && [ ! -s "$d/fw.log" ] && [ "$(grep -c "^ALREADY" "$d/out")" = 6 ] \
   && [ "$(grep -c T-4 "$d/triage/drop-pending.jsonl")" = 1 ]; then
    ok "rerun is idempotent"
else bad "idempotence (rc=$rc)" "$d/out"; fi

# 6. A changed answer is applied again.
echo '{"ts": "t", "batch": "b1", "task": "T-1", "choice": "park"}' >> "$d/triage/answers.jsonl"
run "$d" --apply
if grep -qx "task update T-1 --horizon later" "$d/fw.log"; then
    ok "changed answer re-applied"
else bad "changed answer" "$d/out"; fi

# 7. A gate refusing close is SKIPPED with its reason, exit 0, retried next run.
d=$(setup gate)
FAKE_FW_CLOSE_RC=1 run "$d" --apply; rc=$?
if [ "$rc" = 0 ] && grep -q "SKIPPED.*T-3.*Verification gate failed" "$d/out" \
   && ! grep -q '"task": "T-3"' "$d/triage/applied.jsonl"; then
    ok "gate refusal -> SKIPPED with reason, not recorded"
else bad "gate refusal (rc=$rc)" "$d/out"; fi
: > "$d/fw.log"
run "$d" --apply
if grep -qx "task update T-3 --status work-completed" "$d/fw.log"; then
    ok "skipped close retried on the next run"
else bad "close retry" "$d/out"; fi

# 8. A failing horizon update is FAILED, exit 1.
d=$(setup failh)
FAKE_FW_HORIZON_RC=3 run "$d" --apply; rc=$?
if [ "$rc" = 1 ] && grep -q "FAILED.*T-2" "$d/out"; then
    ok "horizon failure -> FAILED, exit 1"
else bad "horizon failure (rc=$rc)" "$d/out"; fi

# 9. Close on an already-completed task does not call fw.
d=$(setup done)
: > "$d/tasks/completed/T-3-c.md"
run "$d" --apply
if ! grep -q "T-3" "$d/fw.log" && grep -q "OK.*T-3.*already completed" "$d/out"; then
    ok "already-completed close is a no-op"
else bad "already-completed" "$d/out"; fi

# 10. Tooling: missing batch -> exit 2, never clean.
d=$(setup tool)
rm "$d/triage/batch.json"
run "$d"; rc=$?
if [ "$rc" = 2 ] && grep -q "fail-closed" "$d/out"; then
    ok "missing batch -> exit 2"
else bad "missing batch (rc=$rc)" "$d/out"; fi

# 11. Tooling: --apply with no executable fw -> exit 2.
d=$(setup nofw)
TRIAGE_DIR="$d/triage" TRIAGE_FW="$d/none" TRIAGE_TASKS_DIR="$d/tasks" bash "$SCRIPT" --apply > "$d/out" 2>&1; rc=$?
if [ "$rc" = 2 ]; then ok "missing fw -> exit 2"; else bad "missing fw (rc=$rc)" "$d/out"; fi

# 12. --json is parseable.
d=$(setup json)
run "$d" --json
if python3 -c "import json,sys; d=json.load(open(sys.argv[1])); assert d['summary']['answers']==6" "$d/out"; then
    ok "--json parses"
else bad "--json" "$d/out"; fi

echo "triage-apply fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
