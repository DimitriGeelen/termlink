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
# unparseable output; never a false clean).
# Usage: check-file-shaped-posts.sh [--json] [--quiet] [--ack] [--no-heartbeat]
# Seams: FILE_SHAPED_TEST_DIR (list.json + <topic-slug>.ndjson), FILE_SHAPED_STATE, FILE_SHAPED_HEARTBEAT
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

def envelopes(topic):
    if test_dir:
        p = os.path.join(test_dir, slug(topic) + ".ndjson")
        if os.path.exists(p):
            for line in open(p):
                if line.strip(): yield json.loads(line)
        return
    cursor = 0
    while True:
        try:
            out = subprocess.run(["termlink", "channel", "subscribe", topic, "--json", "--cursor",
                                  str(cursor), "--limit", "1000"], capture_output=True, text=True,
                                 timeout=120).stdout
        except Exception as e: tooling(f"subscribe {topic}: {e}")
        rows = []
        for line in out.splitlines():
            try: rows.append(json.loads(line))
            except Exception: continue
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

report, new_acked = [], {}
for name, count in topics():
    if not count: continue
    found = []
    for env in envelopes(name):
        shape = file_shaped(env)
        if shape:
            md = env.get("metadata") or {}
            found.append({"offset": env["offset"], "shape": shape,
                          "from": md.get("from_project") or env.get("sender_id", "?")[:16]})
    if not found: continue
    newest = max(f["offset"] for f in found)
    new_acked[name] = newest
    fresh = [f for f in found if f["offset"] > acked.get(name, -1)]
    report.append({"topic": name, "total": len(found), "new": len(fresh),
                   "shapes": sorted({f["shape"] for f in found}),
                   "senders": sorted({str(f["from"]) for f in found})})

firing = [r for r in report if r["new"]]
if os.environ["FILE_SHAPED_ACK"] == "1":
    os.makedirs(os.path.dirname(state_path) or ".", exist_ok=True)
    json.dump(new_acked, open(state_path, "w"), indent=1, sort_keys=True)
if os.environ["FILE_SHAPED_JSON"] == "1":
    print(json.dumps({"ok": not firing, "acked_now": os.environ["FILE_SHAPED_ACK"] == "1",
                      "topics": report, "firing": [r["topic"] for r in firing]}))
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
sys.exit(1 if firing and os.environ["FILE_SHAPED_ACK"] != "1" else 0)
PY
