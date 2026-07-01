#!/usr/bin/env bash
# T-2298 (arc-003 reliable-comms, V6 slice S1) — per-conversation journal mirror.
#
# The smallest-safe-first step of the V6 apex (T-2296): a PURE READ-SIDE mirror of
# dm: conversation turns into a durable per-conversation SQLite journal under
# ~/.termlink/journals/. The hub firehose stays authoritative and untouched — this
# script only reads (`channel subscribe`) and writes its own sqlite. Moving dm:
# turns OFF the firehose is S5 (out of scope here).
#
# Ships script-first (no Rust rebuild), mirroring the V3a notify-sidecar precedent.
# Idempotent: re-running over the same offsets inserts no duplicate rows (the
# (topic, offset) primary key + INSERT OR IGNORE). Run it periodically (cron / the
# V3a sidecar loop in S3) to keep the journal fresh.
#
# Exit: 0 ok · 2 usage / tooling error
set -uo pipefail

TERMLINK="${TERMLINK_BIN:-termlink}"

JOURNAL="${TERMLINK_JOURNAL_PATH:-$HOME/.termlink/journals/journal.sqlite}"
HUB=""
ONE_TOPIC=""
SINCE_OFFSET=0
LIMIT=100000
FORMAT=human

die() { echo "journal-mirror: $*" >&2; exit 2; }

usage() {
    sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: journal-mirror.sh [OPTIONS]
  --hub ADDR           mirror from this hub (default: local hub)
  --journal PATH       sqlite journal path (default: ~/.termlink/journals/journal.sqlite,
                       or $TERMLINK_JOURNAL_PATH)
  --topic T            mirror only this topic (default: every dm:* topic on the hub)
  --since-offset N     start subscribing at offset N (default 0; idempotency makes
                       a full re-scan cheap, so 0 is the safe default)
  --limit N            max envelopes per topic per pass (default 100000)
  --once               single pass (default; accepted for forward-compat)
  --json               emit a JSON summary envelope
  -h, --help           this help

Exit: 0 ok · 2 usage / tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --hub)          HUB="${2:-}"; shift 2 ;;
        --journal)      JOURNAL="${2:-}"; shift 2 ;;
        --topic)        ONE_TOPIC="${2:-}"; shift 2 ;;
        --since-offset) SINCE_OFFSET="${2:-}"; shift 2 ;;
        --limit)        LIMIT="${2:-}"; shift 2 ;;
        --once)         shift ;;
        --json)         FORMAT=json; shift ;;
        -h|--help)      usage; exit 0 ;;
        *)              die "unknown arg: $1 (try --help)" ;;
    esac
done

command -v "$TERMLINK" >/dev/null 2>&1 || die "termlink not on PATH"
command -v jq >/dev/null 2>&1           || die "jq not available"
command -v sqlite3 >/dev/null 2>&1      || die "sqlite3 not available"
command -v python3 >/dev/null 2>&1      || die "python3 not available"
case "$SINCE_OFFSET" in ''|*[!0-9]*) die "--since-offset must be a non-negative integer" ;; esac
case "$LIMIT" in ''|*[!0-9]*) die "--limit must be a positive integer" ;; esac

hub_args=()
[ -n "$HUB" ] && hub_args=(--hub "$HUB")

# Ensure the journal + schema exist. (topic, offset) PK gives idempotency.
mkdir -p "$(dirname "$JOURNAL")" 2>/dev/null || die "cannot create journal dir $(dirname "$JOURNAL")"
sqlite3 "$JOURNAL" <<'SQL' || die "cannot initialize journal schema at $JOURNAL"
CREATE TABLE IF NOT EXISTS messages (
    topic           TEXT    NOT NULL,
    offset          INTEGER NOT NULL,
    conversation_id TEXT    NOT NULL DEFAULT '',
    sender_id       TEXT    NOT NULL DEFAULT '',
    msg_type        TEXT    NOT NULL DEFAULT '',
    ts              INTEGER NOT NULL DEFAULT 0,
    payload         TEXT    NOT NULL DEFAULT '',
    observed_addr   TEXT    NOT NULL DEFAULT '',
    PRIMARY KEY (topic, offset)
);
CREATE INDEX IF NOT EXISTS idx_messages_convo ON messages(conversation_id, offset);
SQL

# Resolve the topic list.
topics=""
if [ -n "$ONE_TOPIC" ]; then
    topics="$ONE_TOPIC"
else
    # `channel list --json` is {"topics":[{"name":...}]} (object); tolerate a bare
    # array too — (.topics // .) handles both (the V3a probe-bug lesson).
    topics="$("$TERMLINK" channel list "${hub_args[@]+"${hub_args[@]}"}" --prefix "dm:" --json 2>/dev/null \
        | jq -r '(.topics // .)[]?.name // empty' 2>/dev/null)"
fi

# Python inserter: reads NDJSON envelopes on stdin, INSERT OR IGNORE per row,
# prints the number of NEWLY inserted rows (changes()). Parameterized — robust for
# arbitrary payloads (newlines, quotes, unicode).
insert_py='
import sys, json, sqlite3, base64
db = sys.argv[1]
con = sqlite3.connect(db)
cur = con.cursor()
before = con.total_changes
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        e = json.loads(line)
    except Exception:
        continue
    md = e.get("metadata") or {}
    try:
        payload = base64.b64decode(e.get("payload_b64") or "").decode("utf-8", "replace")
    except Exception:
        payload = ""
    cur.execute(
        "INSERT OR IGNORE INTO messages"
        "(topic,offset,conversation_id,sender_id,msg_type,ts,payload,observed_addr)"
        " VALUES(?,?,?,?,?,?,?,?)",
        (
            e.get("topic") or "",
            int(e.get("offset") or 0),
            str(md.get("conversation_id") or ""),
            e.get("sender_id") or "",
            e.get("msg_type") or "",
            int(e.get("ts") or 0),
            payload,
            str(md.get("observed_addr") or md.get("addr") or ""),
        ),
    )
con.commit()
print(con.total_changes - before)
'

total_topics=0
total_new=0
while IFS= read -r t; do
    [ -n "$t" ] || continue
    total_topics=$((total_topics + 1))
    new="$("$TERMLINK" channel subscribe "$t" "${hub_args[@]+"${hub_args[@]}"}" \
              --cursor "$SINCE_OFFSET" --limit "$LIMIT" --json 2>/dev/null \
           | python3 -c "$insert_py" "$JOURNAL" 2>/dev/null)"
    case "$new" in ''|*[!0-9]*) new=0 ;; esac
    total_new=$((total_new + new))
done <<EOF
$topics
EOF

if [ "$FORMAT" = json ]; then
    printf '{"ok":true,"journal":"%s","topics_scanned":%s,"rows_inserted":%s}\n' \
        "$JOURNAL" "$total_topics" "$total_new"
else
    echo "journal-mirror: ${total_topics} dm topic(s) scanned, ${total_new} new row(s) → $JOURNAL"
fi
exit 0
