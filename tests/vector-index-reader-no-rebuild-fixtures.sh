#!/usr/bin/env bash
# guard-layer: source
#
# T-3337 — a vector-index READER must never rebuild (and so never delete) the index.
# Port of AEF T-3786: a reader that hit a transient open error fell through to
# build_index(), which unlinks the live file first and rebuilds for hours under the
# caller's timeout; the killed rebuild left an almost empty index (2.5 GB -> 45 KB).
#
# Hermetic: DB_PATH is pointed at a temp dir and build_index is replaced by a
# tripwire, so the real index and Ollama are never touched.
set -uo pipefail
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
FW="$PROJECT_ROOT/.agentic-framework"
[ -r "$FW/web/embeddings.py" ] || { echo "TOOLING: no $FW/web/embeddings.py" >&2; exit 2; }
command -v python3 >/dev/null || { echo "TOOLING: python3 missing" >&2; exit 2; }
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

cd "$PROJECT_ROOT" && PYTHONPATH="$FW" T3337_TMP="$TMP" python3 - <<'PY'
import os, sqlite3, sys
from pathlib import Path
try:
    import web.embeddings as e
except Exception as ex:
    print(f"TOOLING: cannot import web.embeddings: {ex}", file=sys.stderr); sys.exit(2)

tmp = Path(os.environ["T3337_TMP"])
calls = []
e.build_index = lambda *a, **k: calls.append(1)   # tripwire
passed = failed = 0

def case(name, setup):
    global passed, failed
    db = tmp / f"{name}.db"
    setup(db)
    before = db.stat().st_size if db.exists() else None
    e.DB_PATH = db; e._db = None; calls.clear()
    try:
        e._get_db(); raised = None
    except RuntimeError as ex:
        raised = str(ex)
    after = db.stat().st_size if db.exists() else None
    ok = raised and "fw index reindex" in raised and not calls and (before is None or (after or 0) >= before)
    if ok: passed += 1; print(f"  ok   {name}: raised, no rebuild, file not shrunk")
    else:  failed += 1; print(f"  FAIL {name}: raised={raised!r} rebuild_calls={len(calls)} size {before}->{after}")

case("missing", lambda p: None)
case("corrupt", lambda p: p.write_bytes(b"not a sqlite file " * 400))   # >4096 bytes, open fails
def empty(p):
    c = sqlite3.connect(p); c.execute("create table documents(id integer primary key, path text)")
    c.execute("create table pad(x)"); c.executemany("insert into pad values (?)", [("x"*100,)]*80)
    c.commit(); c.close()
case("empty", empty)

# discovery page must not start a rebuild either
src = open(os.path.join(os.environ["PYTHONPATH"], "web/blueprints/discovery.py")).read()
if "build_index()" in src: failed += 1; print("  FAIL discovery: still calls build_index()")
else: passed += 1; print("  ok   discovery: no background build_index()")

print(f"\n  passed: {passed}   failed: {failed}")
sys.exit(1 if failed else 0)
PY
