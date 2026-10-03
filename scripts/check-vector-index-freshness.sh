#!/usr/bin/env bash
# T-3336 — is the vector index (fw ask / RAG memory) alive and current?
#
# WHY THIS EXISTS
# ---------------
# On 2026-10-03 the index was found frozen at T-2508 (early August) while the
# project had reached T-3336: about 830 tasks and every September/October design
# document were invisible to semantic recall. Three defects composed into silence:
#   1. .context/cron-registry.yaml never scheduled `index-reindex-hourly`;
#   2. `fw index reindex` could not import the framework's web/ module in a vendored
#      checkout, and reported that as exit 0;
#   3. `fw ask` crashed on the same import.
# Nothing fired for two months. This check is the detection that was missing.
#
# FIRES (exit 1) when any of:
#   A. the reindex import path is broken (`fw index reindex` would report unimportable);
#   B. the newest task id in the index lags the newest task file by more than
#      --max-task-lag (default 50);
#   C. the index file is older than --max-age-hours (default 48) while newer corpus
#      files exist.
# Exit 0 healthy · 1 firing · 2 tooling (no index file, no python3). Never a vacuous
# pass: an index it cannot read is exit 2.
#
# Usage: check-vector-index-freshness.sh [--json] [--quiet] [--no-heartbeat]
#        [--max-task-lag N] [--max-age-hours N]
# Seams (fixtures): VEC_INDEX_DB, VEC_TASKS_DIR, VEC_IMPORT_RC (canned import rc),
#   VEC_HEARTBEAT_FILE, VEC_NOW (epoch seconds).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DB="${VEC_INDEX_DB:-$ROOT/.context/working/fw-vec-index.db}"
TASKS="${VEC_TASKS_DIR:-$ROOT/.tasks}"
HB="${VEC_HEARTBEAT_FILE:-$ROOT/.context/working/.vector-index-canary.heartbeat}"
json=0 quiet=0 hb=1 lag_max=50 age_max=48
while [ $# -gt 0 ]; do
    case "$1" in
        --json) json=1; shift ;;
        --quiet) quiet=1; shift ;;
        --no-heartbeat) hb=0; shift ;;
        --max-task-lag) lag_max="$2"; shift 2 ;;
        --max-age-hours) age_max="$2"; shift 2 ;;
        -h|--help) sed -n '2,29p' "$0"; exit 0 ;;
        *) echo "check-vector-index-freshness: unknown arg $1" >&2; exit 2 ;;
    esac
done
_hb() { mkdir -p "$(dirname "$HB")" 2>/dev/null; date +%s > "$HB" 2>/dev/null || true; }
[ "$hb" -eq 1 ] && trap _hb EXIT
command -v python3 >/dev/null || { echo "check-vector-index-freshness: python3 missing" >&2; exit 2; }
[ -r "$DB" ] || { echo "check-vector-index-freshness: no index at $DB" >&2; exit 2; }

# A. importability, exactly as the cron runs it (FRAMEWORK_ROOT on the path).
if [ -n "${VEC_IMPORT_RC:-}" ]; then import_rc="$VEC_IMPORT_RC"
else
    (cd "$ROOT" && PYTHONPATH="$ROOT/.agentic-framework" timeout 60 python3 -c 'import web.embeddings' >/dev/null 2>&1); import_rc=$?
fi

res="$(VEC_DB="$DB" VEC_TASKS="$TASKS" VEC_NOW="${VEC_NOW:-}" python3 - <<'PY'
import os, re, sqlite3, glob, json, time
db, tasks = os.environ["VEC_DB"], os.environ["VEC_TASKS"]
now = float(os.environ.get("VEC_NOW") or time.time())
try:
    c = sqlite3.connect(db)
    paths = [p for (p,) in c.execute("select distinct path from documents where path like '%T-%'")]
except Exception as e:
    print(json.dumps({"error": str(e)[:200]})); raise SystemExit
def tid(p):
    m = re.search(r"T-(\d+)", os.path.basename(p)); return int(m.group(1)) if m else None
idx = [t for t in map(tid, paths) if t is not None]
disk = [t for t in map(tid, glob.glob(os.path.join(tasks, "*", "T-*.md"))) if t is not None]
newest_files = max((os.path.getmtime(f) for f in glob.glob(os.path.join(tasks, "*", "T-*.md"))), default=0)
print(json.dumps({"index_max": max(idx) if idx else 0, "disk_max": max(disk) if disk else 0,
                  "age_hours": round((now - os.path.getmtime(db)) / 3600, 1),
                  "newer_corpus": newest_files > os.path.getmtime(db)}))
PY
)"
if printf '%s' "$res" | grep -q '"error"'; then
    echo "check-vector-index-freshness: cannot read index: $res" >&2; exit 2
fi
imax=$(printf '%s' "$res" | python3 -c 'import json,sys;print(json.load(sys.stdin)["index_max"])')
dmax=$(printf '%s' "$res" | python3 -c 'import json,sys;print(json.load(sys.stdin)["disk_max"])')
age=$(printf '%s' "$res" | python3 -c 'import json,sys;print(json.load(sys.stdin)["age_hours"])')
newer=$(printf '%s' "$res" | python3 -c 'import json,sys;print(str(json.load(sys.stdin)["newer_corpus"]).lower())')
lag=$((dmax - imax))
firing=()
[ "$import_rc" -ne 0 ] && firing+=("A: reindex import path broken (web.embeddings not importable with FRAMEWORK_ROOT on the path)")
[ "$lag" -gt "$lag_max" ] && firing+=("B: index stops at T-$imax, newest task is T-$dmax (lag $lag > $lag_max)")
python3 -c "import sys; sys.exit(0 if float('$age') > float('$age_max') else 1)" && [ "$newer" = "true" ] \
    && firing+=("C: index is ${age}h old (> ${age_max}h) and newer task files exist")

if [ "$json" -eq 1 ]; then
    printf '%s\n' "${firing[@]+"${firing[@]}"}" | python3 -c "
import json,sys
f=[l.strip() for l in sys.stdin if l.strip()]
print(json.dumps({'ok': not f, 'firing': f, 'index_max': $imax, 'disk_max': $dmax, 'age_hours': $age, 'import_rc': $import_rc}))"
elif [ "${#firing[@]}" -gt 0 ]; then
    for f in "${firing[@]}"; do echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) === VECTOR INDEX $f"; done
elif [ "$quiet" -eq 0 ]; then
    echo "vector index healthy: covers through T-$imax (newest T-$dmax), ${age}h old, reindex import ok"
fi
[ "${#firing[@]}" -gt 0 ] && exit 1
exit 0
