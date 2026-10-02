#!/usr/bin/env bash
# T-3310 fixtures for scripts/check-forever-owner-freshness.sh (D1 backstop canary).
# Hermetic: canned hub status JSON, a fake task-create command, a scratch tasks
# dir, a fake pending-commit recorder. Never touches a hub or real task files.
set -uo pipefail
cd "$(dirname "$0")/.."
S="$PWD/scripts/check-forever-owner-freshness.sh"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
pass=0 fail=0
ok()  { pass=$((pass+1)); echo "PASS  $1"; }
bad() { fail=$((fail+1)); echo "FAIL  $1"; }

gov() { # <mode> <enforced> → canned status
    cat > "$T/s.json" <<EOF
{"ok":true,"status":"running","governor":{"forever_requires_owner":"$1","forever_enforced":$2,
 "forever_enforce_at_ms":1790000000000,"forever_quiet_since_ms":1788000000000,
 "bare_forever_senders":[{"sender":"peer:192.168.10.121:40000","creates":3,"last_ms":1788000000000}]}}
EOF
}
BEFORE=$(date -u -d 2026-11-01 +%s); AFTER=$(date -u -d 2026-11-16 +%s)
mkdir -p "$T/.tasks/active"; ln -s "$T/.tasks/active" "$T/tasks"
cat > "$T/create" <<'EOF'
#!/usr/bin/env bash
f="$PWD/.tasks/active/T-9001-fake.md"
printf '## Context\n\n<!-- x -->\n\n## Acceptance Criteria\n\n- [ ] [First criterion]\n- [ ] [Second criterion]\n' > "$f"
echo "ID: T-9001"; echo "File: $f"; echo "$*" > "$FAKE_TASKS/../create-args"
EOF
printf '#!/usr/bin/env bash\necho "$@" >> "%s/pending.log"\n' "$T" > "$T/pending"
chmod +x "$T/create" "$T/pending"
# Runs from $T so a filed task lands at .tasks/active/ relative to the cwd,
# which is the only place the canary records into the pending manifest.
run() { ( cd "$T" && FOREVER_OWNER_TEST_JSON="$T/s.json" FOREVER_OWNER_TASKS_DIR=".tasks/active" \
        FAKE_TASKS="$T/.tasks/active" FOREVER_OWNER_CREATE_CMD="$T/create" FOREVER_OWNER_PENDING_CMD="$T/pending" \
        FOREVER_OWNER_HEARTBEAT_FILE="$T/hb" CI= bash "$S" "$@" > "$T/out" 2> "$T/err"; echo $? ); }

# 1. auto, not enforcing, before the backstop → healthy (the clock is still allowed to run)
gov auto false; rc=$(FOREVER_OWNER_TEST_NOW=$BEFORE run)
[ "$rc" = 0 ] && grep -q "healthy" "$T/out" && ok "auto before backstop is healthy" || bad "auto before backstop rc=$rc"
[ -s "$T/hb" ] && ok "heartbeat written on exit" || bad "no heartbeat"

# 2. auto, still not enforcing AFTER the backstop → fires, names the sender, files ONE task
gov auto false; rc=$(FOREVER_OWNER_TEST_NOW=$AFTER run)
[ "$rc" = 1 ] && ok "backstop passed, not enforcing → fires" || bad "backstop rc=$rc"
grep -q "192.168.10.121" "$T/out" && ok "names who still sends bare forever" || bad "sender not named"
grep -q "forever-owner-backstop" "$T/tasks/T-9001-fake.md" && grep -q "forever_enforced=true" "$T/tasks/T-9001-fake.md" \
    && ok "filed task carries marker + AC" || bad "filed task body wrong"
grep -q "T-9001" "$T/pending.log" && ok "recorded in pending-commit manifest" || bad "not recorded"

# 3. second firing run does NOT file again (de-dup by marker)
rc=$(FOREVER_OWNER_TEST_NOW=$AFTER run)
[ "$rc" = 1 ] && grep -q "already exists" "$T/out" && [ "$(wc -l < "$T/pending.log")" = 1 ] \
    && ok "no duplicate task on the next run" || bad "duplicate filing (rc=$rc)"
rm -f "$T/tasks/"*.md "$T/pending.log"

# 4. enforcing after the backstop → healthy
gov auto true; rc=$(FOREVER_OWNER_TEST_NOW=$AFTER run)
[ "$rc" = 0 ] && ok "enforcing is healthy after the backstop" || bad "enforcing rc=$rc"

# 5. opt-out 'never' fires on ANY date (layer 3: loud every day)
gov never false; rc=$(FOREVER_OWNER_TEST_NOW=$BEFORE run --no-file-task)
[ "$rc" = 1 ] && grep -q "opt-out" "$T/out" && ok "never fires before the backstop too" || bad "never rc=$rc"
[ ! -e "$T/tasks/T-9001-fake.md" ] && ok "--no-file-task files nothing" || bad "--no-file-task filed"

# 6. CI never files
gov auto false; FOREVER_OWNER_TEST_JSON="$T/s.json" FOREVER_OWNER_TEST_NOW=$AFTER FOREVER_OWNER_TASKS_DIR="$T/tasks" \
  FAKE_TASKS="$T/tasks" FOREVER_OWNER_CREATE_CMD="$T/create" FOREVER_OWNER_HEARTBEAT_FILE="$T/hb" CI=1 \
  bash "$S" > "$T/out" 2>&1; rc=$?
[ "$rc" = 1 ] && [ ! -e "$T/tasks/T-9001-fake.md" ] && grep -q "skip (CI)" "$T/out" && ok "CI fires but never files" || bad "CI rc=$rc"

# 7. fail-closed: hub predating T-3310, no governor block, garbage → exit 2, never 0
echo '{"ok":true,"governor":{"connections_active":1}}' > "$T/s.json"; rc=$(FOREVER_OWNER_TEST_NOW=$AFTER run)
[ "$rc" = 2 ] && grep -q "predates T-3310" "$T/err" && ok "old hub → tooling (2)" || bad "old hub rc=$rc"
echo '{"ok":true,"status":"not_running"}' > "$T/s.json"; rc=$(run)
[ "$rc" = 2 ] && ok "hub not running → tooling (2)" || bad "not running rc=$rc"
echo 'garbage' > "$T/s.json"; rc=$(run)
[ "$rc" = 2 ] && ok "unparseable → tooling (2)" || bad "garbage rc=$rc"

# 8. --json carries the verdict fields
gov auto false; FOREVER_OWNER_TEST_NOW=$AFTER run --json --no-file-task >/dev/null
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d["rc"]==1 and d["mode"]=="auto" and d["senders"]' "$T/out" \
    && ok "--json verdict" || bad "--json shape"

# 9. MUTANT: a canary that ignores the backstop date must turn case 2 red
sed 's/if now >= backstop and not enforced:/if False:/' "$S" > "$T/mut.sh"
gov auto false; FOREVER_OWNER_TEST_JSON="$T/s.json" FOREVER_OWNER_TEST_NOW=$AFTER FOREVER_OWNER_HEARTBEAT_FILE="$T/hb" \
  bash "$T/mut.sh" --no-file-task > /dev/null 2>&1; mrc=$?
[ "$mrc" = 0 ] && ok "mutant (backstop removed) reads healthy → case 2 would catch it" || bad "mutant not distinguishable (rc=$mrc)"

echo "forever-owner canary fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
