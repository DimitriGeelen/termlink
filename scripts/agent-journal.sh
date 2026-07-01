#!/usr/bin/env bash
# T-2298 (arc-003 reliable-comms, V6 slice S1) — per-conversation journal query verb.
#
# Reads the durable per-conversation journal written by journal-mirror.sh (NOT the
# hub firehose). Answers "show me this conversation's history" from local sqlite —
# no hub, no network, no auth. This is the mineable read surface (T-2296 AC4).
#
# <conversation> matches EITHER a topic name (dm:a:b) OR a metadata conversation_id,
# so you can query by peer-pair or by thread.
#
# Exit: 0 ok (rows may be empty) · 2 usage / tooling error
set -uo pipefail

JOURNAL="${TERMLINK_JOURNAL_PATH:-$HOME/.termlink/journals/journal.sqlite}"
CONVO=""
SINCE_OFFSET=""
LIMIT=""
FORMAT=human

die() { echo "agent-journal: $*" >&2; exit 2; }

usage() {
    sed -n '2,11p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: agent-journal.sh <conversation> [OPTIONS]
  <conversation>       a dm: topic name OR a conversation_id
  --since-offset N     only rows with offset >= N
  --limit N            cap the number of rows returned (default: all)
  --journal PATH       sqlite journal path (default: ~/.termlink/journals/journal.sqlite,
                       or $TERMLINK_JOURNAL_PATH)
  --json               emit a JSON array envelope
  -h, --help           this help

Exit: 0 ok (rows may be empty) · 2 usage / tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --since-offset) SINCE_OFFSET="${2:-}"; shift 2 ;;
        --limit)        LIMIT="${2:-}"; shift 2 ;;
        --journal)      JOURNAL="${2:-}"; shift 2 ;;
        --json)         FORMAT=json; shift ;;
        -h|--help)      usage; exit 0 ;;
        --*)            die "unknown arg: $1 (try --help)" ;;
        *)              [ -z "$CONVO" ] && CONVO="$1" || die "unexpected extra arg: $1"; shift ;;
    esac
done

[ -n "$CONVO" ] || { usage >&2; die "missing <conversation> (a dm: topic or conversation_id)"; }
command -v sqlite3 >/dev/null 2>&1 || die "sqlite3 not available"
command -v python3 >/dev/null 2>&1 || die "python3 not available"
[ -f "$JOURNAL" ] || die "journal not found at $JOURNAL (run journal-mirror.sh first)"
if [ -n "$SINCE_OFFSET" ]; then
    case "$SINCE_OFFSET" in ''|*[!0-9]*) die "--since-offset must be a non-negative integer" ;; esac
fi
if [ -n "$LIMIT" ]; then
    case "$LIMIT" in ''|*[!0-9]*) die "--limit must be a positive integer" ;; esac
fi

# Query via python (parameterized — robust for arbitrary convo strings/payloads).
query_py='
import sys, json, sqlite3
db, convo, since, limit, fmt = sys.argv[1:6]
con = sqlite3.connect(db)
con.row_factory = sqlite3.Row
sql = ("SELECT topic,offset,conversation_id,sender_id,msg_type,ts,payload,observed_addr"
       " FROM messages WHERE (topic=? OR conversation_id=?)")
params = [convo, convo]
if since:
    sql += " AND offset >= ?"; params.append(int(since))
sql += " ORDER BY topic, offset"
if limit:
    sql += " LIMIT ?"; params.append(int(limit))
rows = [dict(r) for r in con.execute(sql, params).fetchall()]
if fmt == "json":
    print(json.dumps({"ok": True, "conversation": convo, "count": len(rows), "messages": rows}))
else:
    if not rows:
        print("agent-journal: no messages for %r in %s" % (convo, db)); sys.exit(0)
    print("agent-journal: %d message(s) for %s" % (len(rows), convo))
    for r in rows:
        preview = (r["payload"] or "").replace("\n", " ")
        if len(preview) > 80:
            preview = preview[:77] + "..."
        print("  [%s@%d] %s %s: %s" % (r["topic"], r["offset"], r["sender_id"][:12], r["msg_type"], preview))
'

python3 -c "$query_py" "$JOURNAL" "$CONVO" "$SINCE_OFFSET" "$LIMIT" "$FORMAT" || die "journal query failed"
exit 0
