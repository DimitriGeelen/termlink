#!/usr/bin/env bash
# guard-layer: source
# T-3071 — fixtures for scripts/journal-mirror.sh.
#
# There were none before this task. The mirror had been treated as plumbing, but
# T-3071 gave it a job that needs guarding: it now reads a PEER-SUPPLIED field
# (metadata.priority) and writes it into the column the injector sorts on. That is
# caller-controlled input reaching an ordering decision, so the clamp is not a
# nicety — without it a peer posts `priority: 1e9` once and pins itself to the head
# of every queue forever. A denial of attention that needs no exploit, only a large
# integer.
#
# THE LOAD-BEARING CASES:
#   C4  an out-of-range value is CLAMPED, not honoured
#   C6  a BOOLEAN is not silently read as 1 (bool is a subclass of int in Python,
#       so `true` would otherwise promote a message whose sender set a flag)
#   C7  an UNPARSEABLE value sorts as NORMAL, never as urgent — a garbled field
#       must not outrank a real one
#   M2  the migration is idempotent BY CONSTRUCTION, and rewrites no existing row
#
# Hermetic: temp databases, stub `termlink`, no hub, no network.
set -u

MIRROR="${MIRROR:-scripts/journal-mirror.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

command -v sqlite3 >/dev/null 2>&1 || { echo "SKIP: sqlite3 not available"; exit 0; }
command -v python3 >/dev/null 2>&1 || { echo "SKIP: python3 not available"; exit 0; }

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/bin"

# stub termlink: the mirror calls `channel subscribe` / `channel list`; return nothing
# so the run exercises schema + migration without needing a hub.
cat > "$WORK/bin/termlink" <<'EOS'
#!/usr/bin/env bash
exit 0
EOS
chmod +x "$WORK/bin/termlink"

# Extract the inserter from the script rather than restating it. Two copies of a
# coercion rule drift, and the copy that drifts is the one that stops clamping.
INS="$WORK/insert.py"
awk "/^insert_py='/{f=1;next} /^'\$/{f=0} f" "$MIRROR" > "$INS"

echo "C0 [ANTI-DRIFT]: the inserter extracts and compiles"
if [ -s "$INS" ] && python3 -m py_compile "$INS" 2>/dev/null; then
    pass "C0 inserter body recovered from $MIRROR"
else
    fail "C0 could not extract/compile the inserter — the awk seam or the quoting broke"
    echo; echo "journal-mirror fixtures: $PASS passed, $FAIL failed"; exit 1
fi

# A stray apostrophe inside the single-quoted insert_py string silently ends it and
# the remainder runs as SHELL. That happened once during T-3071 and cost a debug
# cycle; the tell was `declared: command not found`.
echo "C0b [REGRESSION]: no stray apostrophe inside the single-quoted inserter"
q="$(awk "/^insert_py='/,/^'\$/" "$MIRROR" | grep -c "'")"
[ "$q" -eq 2 ] && pass "C0b exactly the 2 delimiters" \
               || fail "C0b found $q quotes — an apostrophe leaked into the python body"

newdb() {
    rm -f "$1"
    sqlite3 "$1" "CREATE TABLE messages (topic TEXT NOT NULL, offset INTEGER NOT NULL, conversation_id TEXT NOT NULL DEFAULT '', sender_id TEXT NOT NULL DEFAULT '', msg_type TEXT NOT NULL DEFAULT '', ts INTEGER NOT NULL DEFAULT 0, payload TEXT NOT NULL DEFAULT '', observed_addr TEXT NOT NULL DEFAULT '', priority INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (topic, offset));"
}
# feed <db> <json-priority-literal>  -> echoes the stored priority
feed() {
    local db="$1" lit="$2"
    printf '{"topic":"t","offset":1,"ts":1,"metadata":{%s}}\n' "$lit" \
        | python3 "$INS" "$db" >/dev/null 2>&1
    sqlite3 "$db" "SELECT priority FROM messages WHERE offset=1;"
}
check() { # check <label> <json> <want>
    local db="$WORK/c.sqlite"; newdb "$db"
    local got; got="$(feed "$db" "$2")"
    [ "$got" = "$3" ] && pass "$1 -> $got" || fail "$1 -> got '$got' want '$3'"
}

echo "C1: an absent priority is the normal band"
check "C1 absent"        ''                      0
echo "C2: an integer band is kept"
check "C2 int 3"         '"priority":3'          3
echo "C3: a NUMERIC STRING is accepted (senders serialise loosely)"
check "C3 string 5"      '"priority":"5"'        5
echo "C4 [CLAMP]: an absurd value is clamped to the ceiling, not honoured"
check "C4 1e9"           '"priority":1000000000' 9
echo "C5 [CLAMP]: the floor holds too"
check "C5 -1000"         '"priority":-1000'      -9
echo "C6 [BOOL]: true is NOT read as band 1"
check "C6 true"          '"priority":true'       0
echo "C7 [FAIL-SAFE]: an unparseable band sorts as NORMAL, never as urgent"
check "C7 \"high\""      '"priority":"high"'     0
check "C7 list"          '"priority":[9]'        0
check "C7 null"          '"priority":null'       0
echo "C8: a float truncates toward the band below, never rounds up"
check "C8 2.9"           '"priority":2.9'        2

echo "C9: the pre-existing metadata fields still land (no regression)"
db="$WORK/c9.sqlite"; newdb "$db"
printf '{"topic":"t","offset":1,"ts":1,"sender_id":"abc","msg_type":"note","payload_b64":"aGk=","metadata":{"conversation_id":"cid-1","observed_addr":"1.2.3.4","priority":4}}\n' \
    | python3 "$INS" "$db" >/dev/null 2>&1
got="$(sqlite3 -separator '|' "$db" "SELECT conversation_id,observed_addr,payload,priority FROM messages;")"
[ "$got" = "cid-1|1.2.3.4|hi|4" ] && pass "C9 $got" || fail "C9 got '$got'"

echo "M1 [MIGRATION]: a journal predating the column gains it"
OLD="$WORK/old.sqlite"
sqlite3 "$OLD" "CREATE TABLE messages (topic TEXT NOT NULL, offset INTEGER NOT NULL, conversation_id TEXT NOT NULL DEFAULT '', sender_id TEXT NOT NULL DEFAULT '', msg_type TEXT NOT NULL DEFAULT '', ts INTEGER NOT NULL DEFAULT 0, payload TEXT NOT NULL DEFAULT '', observed_addr TEXT NOT NULL DEFAULT '', PRIMARY KEY (topic, offset));"
sqlite3 "$OLD" "INSERT INTO messages VALUES ('t',1,'','s','note',10,'body','');"
before="$(sqlite3 "$OLD" "SELECT COUNT(*)||':'||SUM(LENGTH(payload)) FROM messages;")"
PATH="$WORK/bin:$PATH" TERMLINK="$WORK/bin/termlink" \
    bash "$MIRROR" --journal "$OLD" --topic 'no-such-topic' >/dev/null 2>&1
cols="$(sqlite3 "$OLD" "SELECT COUNT(*) FROM pragma_table_info('messages') WHERE name='priority';")"
[ "$cols" = "1" ] && pass "M1 priority column added to a legacy journal" \
                  || fail "M1 column not added"

echo "M2 [IDEMPOTENT + NON-DESTRUCTIVE]: repeat runs add nothing and rewrite nothing"
for _ in 1 2 3; do
    PATH="$WORK/bin:$PATH" TERMLINK="$WORK/bin/termlink" \
        bash "$MIRROR" --journal "$OLD" --topic 'no-such-topic' >/dev/null 2>&1 || {
            fail "M2 a repeat run exited non-zero"; break; }
done
after="$(sqlite3 "$OLD" "SELECT COUNT(*)||':'||SUM(LENGTH(payload)) FROM messages;")"
cols="$(sqlite3 "$OLD" "SELECT COUNT(*) FROM pragma_table_info('messages') WHERE name='priority';")"
if [ "$before" = "$after" ] && [ "$cols" = "1" ]; then
    pass "M2 content fingerprint unchanged ($after), column still singular"
else
    fail "M2 before=$before after=$after cols=$cols"
fi

echo "M3 [BACKFILL]: every migrated row sits in the normal band, so nothing reorders"
n="$(sqlite3 "$OLD" "SELECT COUNT(*) FROM messages WHERE priority<>0;")"
[ "$n" = "0" ] && pass "M3 no legacy row was promoted or demoted by migrating" \
               || fail "M3 $n row(s) left the normal band"

echo
echo "journal-mirror fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
