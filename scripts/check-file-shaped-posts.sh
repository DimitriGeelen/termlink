#!/usr/bin/env bash
# check-file-shaped-posts.sh — files sent as ordinary messages, outside the artifact path (T-3432)
#
# WHY: arc-011 FX-5 ruling C (2026-10-17). Measured on .107: peers already move files through
# channel.post — `document` messages with a `filename`, and files cut into 40 KB `artifact-chunk`
# pieces with `part`/`sha256`. That content lands in the message body, a plaintext field the hub
# keeps readable, and no per-message size cap can stop it (the pieces are small). So the hub
# keeps a 512 KB backstop cap (PLAINTEXT_FIELD_TOO_LARGE), and this canary makes the chunked
# bypass visible: which topics, which message types, which senders. Refusing such posts is a
# later decision, once PH-2 row P15 gives an encrypted way to send a file.
#
# FILE-SHAPED: msg_type document | artifact-chunk | attachment | file-chunk, OR metadata carries
# `filename`, OR both `part` and `sha256`. NOT file-shaped: msg_type `artifact`, the sanctioned
# `termlink file send` path (its writers are held by check-artifact-writers.sh).
#
# FIRES (exit 1) on a file-shaped post NEWER than the acknowledged offset for its topic. The
# existing ones are acknowledged with --ack (recorded per topic in the state file), so the log
# stays meaningful: it fills only when someone sends a new file this way.
#
# SCOPE (T-2680): reads the local hub only, and only the shapes above; a file disguised as a plain
# `note` is not seen. Exit 0 healthy · 1 new file-shaped post(s) · 2 tooling (hub unreachable,
# unparseable output, or a topic read the hub refused or failed; never a false clean). A refused
# read is named on stderr and the scan continues; a finding elsewhere still exits 1 (T-3843).
# --ack keeps the stored offset of a topic it did not scan (T-3843).
# INBOXES (T-3843 ruling B, 2026-11-24): `inbox:*` topics are NOT read. Their file-shaped posts come
# from the hub's post-time record (`file-shaped-post` rows in the admin log, T-3844), so this canary
# is no second reader of any inbox (P-12). No readable admin log, or a hub that never started
# watching, is a coverage gap (exit 2), never clean. Rollback: FILE_SHAPED_INBOX_SCAN=1 restores the
# inbox scan with the hub/scan parity check.
# Usage: check-file-shaped-posts.sh [--json] [--quiet] [--ack] [--no-heartbeat]
# Seams: FILE_SHAPED_TEST_DIR (list.json + <topic-slug>.ndjson [+ admin.jsonl]), FILE_SHAPED_STATE,
#   FILE_SHAPED_HEARTBEAT, FILE_SHAPED_ADMIN_LOG, FILE_SHAPED_INBOX_SCAN
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JSON=0 QUIET=0 ACK=0 HB=1
for a in "$@"; do
    case "$a" in
        --json) JSON=1 ;; --quiet) QUIET=1 ;; --ack) ACK=1 ;; --no-heartbeat) HB=0 ;;
        -h|--help) sed -n 2,24p "$0"; exit 0 ;;
        *) echo "check-file-shaped-posts: unknown arg '$a'" >&2; exit 2 ;;
    esac
done
STATE="${FILE_SHAPED_STATE:-$ROOT/.context/working/.file-shaped-posts.seen}"
HBF="${FILE_SHAPED_HEARTBEAT:-$ROOT/.context/working/.file-shaped-posts-canary.heartbeat}"
# The heartbeat proves the run completed (T-2843), so it is written on exit, not at start.
[ "$HB" = 1 ] && trap 'touch "$HBF" 2>/dev/null' EXIT

FILE_SHAPED_JSON="$JSON" FILE_SHAPED_QUIET="$QUIET" FILE_SHAPED_ACK="$ACK" FILE_SHAPED_STATE="$STATE" python3 - <<'PY'
import json, os, re, subprocess, sys

test_dir = os.environ.get("FILE_SHAPED_TEST_DIR")
state_path = os.environ["FILE_SHAPED_STATE"]
# T-3843 ruling B: inboxes come from the hub's record unless the rollback flag restores the scan.
inbox_scan = os.environ.get("FILE_SHAPED_INBOX_SCAN") == "1"
FILE_TYPES = {"document", "artifact-chunk", "attachment", "file-chunk"}

def tooling(msg):
    print(f"check-file-shaped-posts: TOOLING: {msg}", file=sys.stderr); sys.exit(2)

def slug(t): return re.sub(r"[^A-Za-z0-9_.-]", "_", t)

def topics():
    if test_dir:
        raw = open(os.path.join(test_dir, "list.json")).read()
    else:
        try:
            raw = subprocess.run(["termlink", "channel", "list", "--json"], capture_output=True,
                                 text=True, timeout=120).stdout
        except Exception as e: tooling(f"channel list: {e}")
    try: d = json.loads(raw)
    except Exception: tooling("channel list output is not JSON")
    ts = d.get("topics", d) if isinstance(d, dict) else d
    return [(t["name"], t.get("count", 0)) for t in ts if isinstance(t, dict) and "name" in t]

# T-3843: a read the hub refused or failed (non-zero exit, an error object, output that is not
# JSON) is a coverage gap for that topic, never an empty topic. The scan goes on; the run ends 2.
problems = []

def page(topic, out, rc, err=""):
    """Rows of one subscribe page, or None (and a recorded problem) when the read did not succeed."""
    if rc != 0:
        problems.append(f"subscribe {topic}: exit {rc}: {(err or out).strip()[:160]}")
        return None
    rows = []
    for line in out.splitlines():
        if not line.strip(): continue
        try: r = json.loads(line)
        except Exception:
            problems.append(f"subscribe {topic}: output is not JSON: {line.strip()[:160]}")
            return None
        if not isinstance(r, dict) or "error" in r:
            problems.append(f"subscribe {topic}: refused: {json.dumps(r)[:160]}")
            return None
        rows.append(r)
    return rows

def envelopes(topic):
    if test_dir:
        raw = os.path.join(test_dir, slug(topic) + ".raw")
        if os.path.exists(raw):
            rcf = os.path.join(test_dir, slug(topic) + ".rc")
            rc = int(open(rcf).read().strip()) if os.path.exists(rcf) else 0
            for r in page(topic, open(raw).read(), rc) or []:
                if isinstance(r.get("offset"), int): yield r
            return
        p = os.path.join(test_dir, slug(topic) + ".ndjson")
        if os.path.exists(p):
            for line in open(p):
                if line.strip(): yield json.loads(line)
        return
    cursor = 0
    while True:
        try:
            res = subprocess.run(["termlink", "channel", "subscribe", topic, "--json", "--cursor",
                                  str(cursor), "--limit", "1000"], capture_output=True, text=True,
                                 timeout=120)
        except Exception as e:
            problems.append(f"subscribe {topic}: {e}")
            return
        rows = page(topic, res.stdout, res.returncode, res.stderr)
        if rows is None: return
        for r in rows:
            if isinstance(r.get("offset"), int): yield r
        offs = [r["offset"] for r in rows if isinstance(r.get("offset"), int)]
        if len(rows) < 1000 or not offs: return
        cursor = max(offs) + 1

def file_shaped(env):
    t = env.get("msg_type") or ""
    if t == "artifact": return None
    md = env.get("metadata") or {}
    if t in FILE_TYPES: return t
    if "filename" in md: return f"{t}+filename"
    if "part" in md and "sha256" in md: return f"{t}+part/sha256"
    return None

try: acked = json.load(open(state_path))
except FileNotFoundError: acked = {}
except Exception: tooling(f"state file unreadable: {state_path}")

# T-3844 (T-3843 ruling H): the hub records file-shaped inbox posts at post time in its admin log. The
# earliest watch-on time is kept in <state>.parity, because the admin log prunes rows older than 30 days.
admin_path = os.environ.get("FILE_SHAPED_ADMIN_LOG") or (os.path.join(test_dir, "admin.jsonl") if test_dir else "/var/lib/termlink/rpc-admin.jsonl")
parity_path = state_path + ".parity"
parity = {"state": "unknown", "since": None, "matched": 0, "hub_only": [], "scan_only": []}
hub_rows, watch_on = None, []
try:
    hub_rows = []
    for line in open(admin_path):
        try: r = json.loads(line)
        except Exception: continue
        if r.get("event") == "file-shaped-watch-on" and isinstance(r.get("ts"), int): watch_on.append(r["ts"])
        elif r.get("event") == "file-shaped-post" and isinstance(r.get("offset"), int) and isinstance(r.get("topic"), str): hub_rows.append(r)
except OSError as e:
    hub_rows = None
    parity["why"] = f"admin log unreadable: {admin_path}: {e.strerror}"
try: since = json.load(open(parity_path)).get("since")
except Exception: since = None
if watch_on: since = min([since] + watch_on) if since else min(watch_on)
if hub_rows is not None and since is None:
    parity["why"] = "the hub has not started watching (no file-shaped-watch-on row)"
# T-3843 ruling B: without the scan, the hub record IS the inbox coverage, so its absence is a gap.
if not inbox_scan and (hub_rows is None or since is None):
    problems.append(f"inbox coverage: {parity['why']} (inboxes are not read; the hub record is their only source)")

# T-3843: --ack starts from the stored offsets, so a topic this run did not scan (or could not
# read) keeps its acknowledgement instead of having it erased.
report, new_acked = [], dict(acked)
# T-3844: inbox findings of this scan, and the first offset scanned per inbox, for the hub parity check.
scan_inbox, first_off = {}, {}

def add_report(name, found):
    if not found: return
    new_acked[name] = max(f["offset"] for f in found)
    fresh = [f for f in found if f["offset"] > acked.get(name, -1)]
    report.append({"topic": name, "total": len(found), "new": len(fresh),
                   "shapes": sorted({f["shape"] for f in found}),
                   "senders": sorted({str(f["from"]) for f in found})})

for name, count in topics():
    if not count: continue
    if name.startswith("inbox:") and not inbox_scan: continue  # never read an inbox (P-12)
    found = []
    for env in envelopes(name):
        if name.startswith("inbox:"):
            first_off[name] = min(first_off.get(name, env["offset"]), env["offset"])
        shape = file_shaped(env)
        if shape:
            md = env.get("metadata") or {}
            found.append({"offset": env["offset"], "shape": shape,
                          "from": md.get("from_project") or env.get("sender_id", "?")[:16]})
            if name.startswith("inbox:"):
                scan_inbox[(name, env["offset"])] = env.get("ts")
    add_report(name, found)

# T-3843 ruling B: inbox findings from the hub's post-time record, one row per post.
if not inbox_scan and hub_rows:
    by_topic = {}
    for r in hub_rows:
        if not r["topic"].startswith("inbox:"): continue
        by_topic.setdefault(r["topic"], {})[r["offset"]] = {"offset": r["offset"], "shape": str(r.get("shape") or "?"),
                                                             "from": r.get("sender") or "?"}
    for name in sorted(by_topic):
        add_report(name, list(by_topic[name].values()))

firing = [r for r in report if r["new"]]

if not inbox_scan and hub_rows is not None and since is not None:
    try:
        json.dump({"since": since}, open(parity_path, "w"))
    except OSError: pass
    parity.update(state="hub-record", since=since,
                  matched=sum(1 for r in hub_rows if r["topic"].startswith("inbox:")))
if inbox_scan and hub_rows is not None and since is not None:
    try:
        json.dump({"since": since}, open(parity_path, "w"))
    except OSError: pass
    hub = {(r["topic"], r["offset"]) for r in hub_rows if isinstance(r.get("post_ts"), int) and r["post_ts"] >= since}
    scan = {k for k, ts in scan_inbox.items() if isinstance(ts, int) and ts >= since}
    # a hub row for an offset the inbox no longer holds (capped or swept) cannot be compared
    # (a topic whose read failed has no first offset, so it is never compared: that gap is already exit 2)
    hub_cmp = {k for k in hub if k[0] in first_off and k[1] >= first_off[k[0]]}
    parity.update(state="ok", since=since, matched=len(hub_cmp & scan),
                  hub_only=sorted(f"{t}@{o}" for t, o in hub_cmp - scan),
                  scan_only=sorted(f"{t}@{o}" for t, o in scan - hub))
    if parity["hub_only"] or parity["scan_only"]: parity["state"] = "mismatch"
parity_firing = parity["state"] == "mismatch"
if os.environ["FILE_SHAPED_ACK"] == "1":
    os.makedirs(os.path.dirname(state_path) or ".", exist_ok=True)
    json.dump(new_acked, open(state_path, "w"), indent=1, sort_keys=True)
if os.environ["FILE_SHAPED_JSON"] == "1":
    print(json.dumps({"ok": not firing and not problems and not parity_firing, "acked_now": os.environ["FILE_SHAPED_ACK"] == "1",
                      "topics": report, "firing": [r["topic"] for r in firing], "tooling": problems, "parity": parity}))
else:
    quiet = os.environ["FILE_SHAPED_QUIET"] == "1"
    if firing and os.environ["FILE_SHAPED_ACK"] != "1":
        print(f"file-shaped-posts canary: FIRING — new file-shaped message(s) in {len(firing)} topic(s), "
              "outside the artifact path (plaintext on the hub, FX-5):")
        for r in firing:
            print(f"  {r['topic']}: {r['new']} new ({r['total']} total)  shapes={','.join(r['shapes'])}  from={','.join(r['senders'])}")
        print("  Send files with `termlink file send` (the artifact path), and once seen and judged,")
        print("  acknowledge with: bash scripts/check-file-shaped-posts.sh --ack")
    elif not quiet:
        tot = sum(r["total"] for r in report)
        verb = "acknowledged now" if os.environ["FILE_SHAPED_ACK"] == "1" else "all acknowledged"
        print(f"file-shaped-posts canary: healthy — {tot} file-shaped message(s) in {len(report)} topic(s), {verb}")
        for r in report:
            print(f"  {r['topic']}: {r['total']}  shapes={','.join(r['shapes'])}  from={','.join(r['senders'])}")
        print("  Scope: local hub; detects the listed shapes only, not a file disguised as a plain note.")
    # T-3844: the hub-side record of inbox posts, compared with this scan.
    if parity_firing:
        print(f"file-shaped-posts canary: PARITY MISMATCH — the hub's post-time record and this scan disagree on inbox posts "
              f"(hub only: {', '.join(parity['hub_only']) or 'none'}; scan only: {', '.join(parity['scan_only']) or 'none'})")
    elif not quiet:
        if parity["state"] == "hub-record":
            print(f"  Inboxes (T-3843): not read; from the hub's post-time record, {parity['matched']} file-shaped inbox post(s) since {parity['since']}")
        elif parity["state"] == "ok":
            print(f"  Parity (T-3844): hub record and scan agree on {parity['matched']} inbox post(s) since {parity['since']}")
        else:
            print(f"  Parity (T-3844): unknown — {parity.get('why', 'not compared')} (not a pass)")
for p in problems:
    print(f"check-file-shaped-posts: TOOLING: {p}", file=sys.stderr)
# A finding outranks a gap (it is the actionable signal); a gap is never reported clean (T-3843).
if firing and os.environ["FILE_SHAPED_ACK"] != "1": sys.exit(1)
if parity_firing: sys.exit(1)
sys.exit(2 if problems else 0)
PY
