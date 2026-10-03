#!/usr/bin/env bash
# guard-layer: source
#
# T-3336 — fixtures for scripts/check-vector-index-freshness.sh. Hermetic: a tiny
# sqlite "index" and a fake .tasks tree in a temp dir; the import probe is canned.
set -uo pipefail
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/check-vector-index-freshness.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
command -v python3 >/dev/null || { echo "TOOLING: python3 missing" >&2; exit 2; }
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/tasks/active" "$TMP/tasks/completed"
mkdb() { # newest indexed task id
    rm -f "$TMP/idx.db"
    python3 -c "
import sqlite3; c=sqlite3.connect('$TMP/idx.db'); c.execute('create table documents(id integer primary key, path text)')
for i in (1,2,$1): c.execute('insert into documents(path) values (?)',('.tasks/completed/T-%d-x.md'%i,))
c.commit()"
}
mktasks() { rm -f "$TMP"/tasks/*/T-*.md; for i in "$@"; do : > "$TMP/tasks/completed/T-$i-x.md"; done; }
run() { VEC_INDEX_DB="$TMP/idx.db" VEC_TASKS_DIR="$TMP/tasks" VEC_IMPORT_RC="${IMP:-0}" \
        VEC_HEARTBEAT_FILE="$TMP/hb" bash "$SCRIPT" "$@"; }

mktasks 1 2 100; mkdb 100
out="$(run)"; rc=$?
[ "$rc" = 0 ] && printf '%s' "$out" | grep -q 'covers through T-100' && ok "current index: healthy, exit 0" || bad "healthy (rc=$rc $out)"
[ -f "$TMP/hb" ] && ok "heartbeat written on exit" || bad "heartbeat"

mktasks 1 2 100 400; mkdb 100
out="$(run --quiet)"; rc=$?
[ "$rc" = 1 ] && printf '%s' "$out" | grep -q 'B: index stops at T-100, newest task is T-400 (lag 300 > 50)' \
    && ok "index lagging 300 tasks fires (the real 2026-10-03 shape)" || bad "lag (rc=$rc $out)"
out="$(run --quiet --max-task-lag 500)"; rc=$?
[ "$rc" = 0 ] && ok "--max-task-lag raises the bar" || bad "lag threshold rc=$rc"

mktasks 1 2 100; mkdb 100
out="$(IMP=1 run --quiet)"; rc=$?
[ "$rc" = 1 ] && printf '%s' "$out" | grep -q 'A: reindex import path broken' \
    && ok "broken import path fires even when coverage looks current" || bad "import (rc=$rc $out)"

j="$(mktasks 1 2 100 400; mkdb 100; run --json)"
printf '%s' "$j" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d["ok"] is False and d["index_max"]==100 and d["disk_max"]==400' \
    && ok "--json carries ok/firing/index_max/disk_max" || bad "json: $j"

rm -f "$TMP/idx.db"; run --quiet >/dev/null 2>&1; rc=$?
[ "$rc" = 2 ] && ok "no index file: exit 2 (tooling), never a pass" || bad "missing index rc=$rc"
echo 'not sqlite' > "$TMP/idx.db"; run --quiet >/dev/null 2>&1; rc=$?
[ "$rc" = 2 ] && ok "unreadable index: exit 2" || bad "unreadable rc=$rc"

echo; echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
