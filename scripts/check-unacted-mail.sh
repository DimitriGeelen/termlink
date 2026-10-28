#!/usr/bin/env bash
# Not a guard-layer source member: it reads the live hub. Its fixtures
# (tests/unacted-mail-fixtures.sh) are, by naming convention.
# T-3618 — mail delivered but never acted on, on every rail, by name.
#
# WHY THIS EXISTS
# ---------------
# Operator 2026-10-28: "messages get lost". Every loss on record had the same shape:
# the message arrived, something recorded it as delivered, and no agent ever acted on
# it (T-3608: a dead session was rung, 14 consults sat in a nickname inbox; a 055 bug
# report sat 8 days on dm:3bba…:d1993c…, receipted "delivered"; INC-a 2026-10-03).
# AEF already measures most of this (`fw sidecar dm-stale`, `inbox-stale`,
# `held-unsurfaced`, `orphan-inboxes`), but only as facts for `fw doctor` and one
# audit WARN among many, which nobody acted on either. This script composes them into
# one list that scripts/session-start-alerts.sh shows by name.
#
# Rails (inventory in T-3618 Updates):
#   dm      every dm:* topic on the local hub whose last real message went unanswered
#           (rule below). AEF's dm-stale is NOT used: it measures "unread by this
#           identity" from its own cursor and counts our own and receipt posts, so a
#           test rail we only wrote to reads as unread while the 055 report, signed with
#           our shared root key, reads as ours.
#   inbox   consult inboxes holding an unread consult (inbox-stale).
#   held    receiver mail held that no session was ever shown (held-unsurfaced).
#   orphan  project-level inboxes nothing reads: mail sent to a nickname (orphan-inboxes).
# The termlink sidecar's own unsurfaced mail is raised by the sidecar itself into its
# needs-attention list, which session-start-alerts.sh section 1d already shows.
#
# Output: one line per finding:  UNACTED  <rail>  <topic or id>  <detail>
#         and NOT CHECKED <rail>: <reason> for a rail that could not be read.
# Exit: 0 nothing found, 1 found, 2 nothing found but a rail could not be read
#       (never "clean" on a read error). A finding wins over a read error (exit 1,
#       the NOT CHECKED line is still printed).
#
# Usage: check-unacted-mail.sh [--threshold-hours N]   (default 24)
# Topics that are known test rails are listed, with a reason, in
# scripts/unacted-mail-ignore.txt (one topic per line, then the reason).
# A message dealt with out of band (forwarded, answered elsewhere, turned into a task)
# is recorded in .context/project/unacted-mail-handled.txt as "<topic> <offset> <date>
# <what was done>"; only that offset is skipped, so a later message re-flags the topic.
# Seams (fixtures): UNACTED_FW (fw launcher), UNACTED_TL (termlink CLI),
#   UNACTED_IGNORE, UNACTED_HANDLED, UNACTED_WINDOW_DAYS (default 14).
set -u

THRESHOLD=24
while [ $# -gt 0 ]; do
    case "$1" in
        --threshold-hours) THRESHOLD="${2:-}"; shift 2 ;;
        -h|--help) sed -n '2,35p' "$0"; exit 0 ;;
        *) echo "check-unacted-mail: unknown argument $1" >&2; exit 2 ;;
    esac
done
case "$THRESHOLD" in ''|*[!0-9.]*) echo "check-unacted-mail: --threshold-hours wants a number" >&2; exit 2 ;; esac

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FW="${UNACTED_FW:-$ROOT/.agentic-framework/bin/fw}"
TL="${UNACTED_TL:-termlink}"
IGNORE="${UNACTED_IGNORE:-$ROOT/scripts/unacted-mail-ignore.txt}"
HANDLED="${UNACTED_HANDLED:-$ROOT/.context/project/unacted-mail-handled.txt}"
WINDOW_DAYS="${UNACTED_WINDOW_DAYS:-14}"   # the hub's default retention window (T-3304 IW-2)

FOUND=0; BLIND=0
SCRATCH="$(mktemp -d)"; trap 'rm -rf "$SCRATCH"' EXIT

fact() {  # <verb> <outfile> : run one AEF fact verb as JSON; rc 0 ok, 1 could not read
    local verb="$1" out="$2" rc
    if [ "$verb" = orphan-inboxes ]; then
        timeout 120 "$FW" sidecar "$verb" --json > "$out" 2>"$out.err"; rc=$?
        [ "$rc" = 0 ] || [ "$rc" = 1 ] || return 1
    else
        timeout 120 "$FW" sidecar "$verb" --json --threshold-hours "$THRESHOLD" > "$out" 2>"$out.err" || return 1
    fi
    python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); sys.exit(0 if isinstance(d,list) else 1)' "$out" 2>/dev/null
}

blind() { echo "NOT CHECKED $1: $2"; BLIND=1; }

# ---- dm rails --------------------------------------------------------------------
# Rule, per message (older than the threshold, younger than the window, not a reply or a
# notice): it is OPEN when nobody it was meant for posted after it: its `to_project`
# addressee, or without one anyone else on the topic. A message posted within 72 h after
# the other side's message is that message's answer, not a new question (a closing
# "thanks, fixed" is not listed). Per message, not per topic: .107's root key DMs itself
# as a shared mailbox, many projects posting with a to_project tag. Sender ids
# cannot say who is "us" here: every project on a host may sign with the host's root key
# (the 055 report was signed d1993c… and addressed to AEF's 3bba…), and receipts are
# posted by auto-confirming sidecars on the recipient's behalf. So the test is "the
# conversation's last word went unanswered", not "unread by this identity".
if ! timeout 60 $TL channel list --prefix dm: --json > "$SCRATCH/dm.json" 2>"$SCRATCH/dm.err" \
   || ! python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); sys.exit(0 if isinstance(d,(list,dict)) else 1)' "$SCRATCH/dm.json" 2>/dev/null; then
    blind dm "termlink channel list --prefix dm: failed ($(head -c 200 "$SCRATCH/dm.err" 2>/dev/null))"
else
    for topic in $(python3 -c 'import json,sys
d=json.load(open(sys.argv[1])); d=d if isinstance(d,list) else d.get("topics",[])
[print(t["name"]) for t in d if t.get("name","").startswith("dm:")]' "$SCRATCH/dm.json"); do
        grep -qxF "$topic" <(grep -v '^[[:space:]]*#' "$IGNORE" 2>/dev/null | awk '{print $1}') && continue
        handled="$(grep -v '^[[:space:]]*#' "$HANDLED" 2>/dev/null | awk -v t="$topic" '$1 == t {print $2}' | tr '\n' ' ')"
        if ! timeout 60 $TL channel subscribe "$topic" --cursor 0 --limit 1000 --json > "$SCRATCH/posts" 2>/dev/null; then
            blind dm "$topic could not be read"; continue
        fi
        line="$(python3 - "$SCRATCH/posts" "$topic" "$THRESHOLD" "$WINDOW_DAYS" "$handled" <<'PY'
import json, sys, base64, time
path, topic, thr, window_d = sys.argv[1], sys.argv[2], float(sys.argv[3]), float(sys.argv[4])
handled = set(sys.argv[5].split())
now = time.time() * 1000
posts = []
for line in open(path):
    line = line.strip()
    if not line.startswith("{"):
        continue
    m = json.loads(line)
    meta = m.get("metadata") or {}
    if m.get("msg_type") != "chat" or meta.get("stage") or meta.get("up_to"):
        continue  # topic metadata, receipts, reactions: not mail
    posts.append(m)
ANSWER_H = 72  # a message within this many hours of the other side's message answers it
def party(m):
    meta = m.get("metadata") or {}
    return meta.get("from_project") or meta.get("from") or m.get("sender_id")
for i, m in enumerate(posts):
    meta = m.get("metadata") or {}
    ts = m.get("ts") or 0
    age_h = (now - ts) / 3.6e6
    if not (thr <= age_h <= window_d * 24) or str(m.get("offset")) in handled:
        continue
    if any(k in meta for k in ("in_reply_to", "reply_to", "_reply_to", "parent")):
        continue
    if bool(meta.get("notice")) or meta.get("kind") == "notice":
        continue
    me, to = party(m), meta.get("to_project")
    prev = posts[i - 1] if i else None
    if prev is not None and party(prev) != me and not to and (ts - (prev.get("ts") or 0)) / 3.6e6 <= ANSWER_H:
        continue  # it answers the other side's message just before it
    # Open unless its addressee (to_project) or, without one, anyone else posted later.
    later = posts[i + 1:]
    if any((party(n) == to) if to else (party(n) != me) for n in later):
        continue
    body = base64.b64decode(m.get("payload_b64") or b"").decode("utf-8", "replace").replace("\n", " ")
    dest = f"  to={to}" if to else ""
    print(f"UNACTED  dm  {topic}  offset {m.get('offset')}  from={me}{dest}  age={age_h:.0f}h  {body[:90]}")
PY
)"
        if [ -n "$line" ]; then printf '%s\n' "$line"; FOUND=1; fi
    done
fi

# ---- inbox, held, orphan ---------------------------------------------------------
for spec in "inbox:inbox-stale" "held:held-unsurfaced" "orphan:orphan-inboxes"; do
    rail="${spec%%:*}"; verb="${spec#*:}"
    if ! fact "$verb" "$SCRATCH/$rail.json"; then
        blind "$rail" "fw sidecar $verb failed or did not answer JSON ($(head -c 200 "$SCRATCH/$rail.json.err" 2>/dev/null))"; continue
    fi
    lines="$(python3 - "$SCRATCH/$rail.json" "$rail" <<'PY'
import json, sys
rows, rail = json.load(open(sys.argv[1])), sys.argv[2]
for r in rows:
    if not isinstance(r, dict):
        r = {"topic": str(r)}
    name = r.get("topic") or r.get("msg_id") or r.get("inbox") or "?"
    bits = [f"{k}={r[k]}" for k in ("unread", "age_hours", "from", "oldest_from", "readers") if r.get(k) not in (None, "")]
    print(f"UNACTED  {rail}  {name}  " + "  ".join(bits))
PY
)"
    if [ -n "$lines" ]; then printf '%s\n' "$lines"; FOUND=1; fi
done

[ "$FOUND" = 1 ] && exit 1
[ "$BLIND" = 1 ] && exit 2
exit 0
