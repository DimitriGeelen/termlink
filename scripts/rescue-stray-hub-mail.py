#!/usr/bin/env python3
"""T-3343 — re-post mail stranded on a stray hub into the canonical hub, once.

Operator ruling B (2026-10-04): a second hub at /tmp/termlink-0 (T-3340) held mail
that never reached its recipients. For each message on the given topics of the stray
hub that the canonical hub lacks (by client_msg_id, else by payload sha256), post it
to the same topic on the canonical hub:
  - payload prefixed with one line naming the rescue, the original sender and time;
  - msg_type kept, so consumers handle it as before;
  - metadata: the original metadata, plus rescued_from, rescued_by, original_ts,
    original_offset, original_client_msg_id, rescue_key.
Idempotent: a message whose rescue_key (or original id/hash) is already on the
canonical topic is skipped, so a re-run posts nothing. Noise is never rescued:
test-fixture projects (origin_project / from_project starting "tmp."), receipts,
nudges.

Usage: rescue-stray-hub-mail.py --stray-dir /tmp/termlink-0 --topic T [--topic T ...]
                                [--dry-run] [--json]
The canonical hub is whatever this process resolves with TERMLINK_RUNTIME_DIR set to
--canonical-dir (default /var/lib/termlink). Exit 0 ok, 1 a post failed, 2 tooling.
"""
import argparse, base64, hashlib, json, os, subprocess, sys

def sub(topic, runtime_dir):
    env = dict(os.environ, TERMLINK_RUNTIME_DIR=runtime_dir)
    r = subprocess.run(["termlink", "channel", "subscribe", topic, "--cursor", "0",
                        "--limit", "5000", "--json"], env=env, capture_output=True,
                       text=True, timeout=60)
    out = []
    for line in r.stdout.splitlines():
        try:
            e = json.loads(line)
        except ValueError:
            continue
        if "payload_b64" in e:
            out.append(e)
    return out

def topic_exists(topic, runtime_dir):
    env = dict(os.environ, TERMLINK_RUNTIME_DIR=runtime_dir)
    r = subprocess.run(["termlink", "channel", "info", topic, "--json"], env=env,
                       capture_output=True, text=True, timeout=30)
    return r.returncode == 0

def is_noise(e, text):
    m = e.get("metadata") or {}
    if str(m.get("from_project", "")).startswith("tmp.") or '"origin_project":"tmp.' in text:
        return "test-fixture project"
    if e.get("msg_type") in ("receipt", "sidecar.receipt") or text.startswith("[nudge]") \
            or text.startswith("[sidecar receipt]"):
        return "receipt/nudge"
    return None

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--stray-dir", required=True)
    ap.add_argument("--canonical-dir", default="/var/lib/termlink")
    ap.add_argument("--topic", action="append", required=True)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    rows, failed = [], 0
    for topic in a.topic:
        stray = sub(topic, a.stray_dir)
        canon = sub(topic, a.canonical_dir)
        seen = set()
        for e in canon:
            m = e.get("metadata") or {}
            for k in ("client_msg_id", "original_client_msg_id", "rescue_key"):
                if m.get(k):
                    seen.add(m[k])
            seen.add(hashlib.sha256(base64.b64decode(e["payload_b64"])).hexdigest())
        exists = topic_exists(topic, a.canonical_dir)
        for e in stray:
            raw = base64.b64decode(e["payload_b64"])
            text = raw.decode(errors="replace")
            m = dict(e.get("metadata") or {})
            h = hashlib.sha256(raw).hexdigest()
            cid = m.get("client_msg_id")
            key = f"rescue-{cid or h[:32]}"
            row = {"topic": topic, "offset": e["offset"], "ts": e.get("ts"),
                   "from": m.get("from_project") or m.get("from_agent") or e.get("sender_id", "")[:16],
                   "msg_type": e.get("msg_type"), "rescue_key": key}
            noise = is_noise(e, text)
            if noise:
                row["action"] = f"skip-noise ({noise})"
            elif (cid and cid in seen) or h in seen or key in seen:
                row["action"] = "skip-present"
            else:
                row["action"] = "post" if not a.dry_run else "would-post"
                if not a.dry_run:
                    env = dict(os.environ, TERMLINK_RUNTIME_DIR=a.canonical_dir)
                    if not exists:
                        subprocess.run(["termlink", "channel", "create", topic], env=env,
                                       capture_output=True, text=True, timeout=30)
                        exists = True
                    prefix = (f"[RESCUED by 010-termlink on 2026-10-04 from stray hub {a.stray_dir} "
                              f"(T-3340/T-3343): originally posted by {row['from']} at ts={e.get('ts')}, "
                              f"never delivered to this hub]\n")
                    meta = {k: v for k, v in m.items() if k not in ("client_msg_id", "cv_key")}
                    meta.update({"rescued_from": a.stray_dir, "rescued_by": "010-termlink",
                                 "original_ts": str(e.get("ts")), "original_offset": str(e["offset"]),
                                 "original_client_msg_id": cid or "", "rescue_key": key})
                    cmd = ["termlink", "channel", "post", topic, "--msg-type", e.get("msg_type") or "note",
                           "--payload", prefix + text, "--client-msg-id", key]
                    for k, v in meta.items():
                        cmd += ["--metadata", f"{k}={v}"]
                    r = subprocess.run(cmd, env=env, capture_output=True, text=True, timeout=60)
                    if r.returncode != 0:
                        row["action"] = "FAILED: " + (r.stderr or r.stdout).strip()[:200]
                        failed += 1
                    else:
                        seen.add(key)
            rows.append(row)
    if a.json:
        print(json.dumps({"ok": failed == 0, "rows": rows}, indent=1))
    else:
        for r in rows:
            print(f"{r['action']:34} {r['topic']:42} off={r['offset']:<4} from={r['from']}")
        n = lambda p: sum(1 for r in rows if r["action"].startswith(p))
        print(f"\nposted/would-post: {n('post') + n('would-post')}   present: {n('skip-present')}   "
              f"noise: {n('skip-noise')}   failed: {failed}")
    sys.exit(1 if failed else 0)

if __name__ == "__main__":
    main()
