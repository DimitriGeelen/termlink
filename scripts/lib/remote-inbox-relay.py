#!/usr/bin/env python3
"""T-3750 — relay peer mail on OUR inboxes on REMOTE hubs into the sidecar receiver.

Why: a peer replying from its own hub writes to our inbox ON ITS HUB (ring20 lesson 5.11).
remote-inbox-mail.py (T-3653) read those inboxes only at session start, so ring20's
operator-escalated bug report (.122, 2026-11-12, offset 23) sat about an hour unseen while
the main session ran. Live mail reaches a running session only through the AEF sidecar
receiver: its watcher drains LOCAL hub inbox topics into the receiver store, injects one line
when the session is ready, and the UserPromptSubmit hook surfaces the stored message.

What: put remote-hub mail on that same path. Each run reads every remote hub's inboxes
addressed to us (remote-inbox-mail.py --mode unseen --full-body, with this relay's OWN
cursor files) and stores each new peer post with lib/sidecar/receiver.store_message as a
`via: hub-topic` envelope, exactly the shape the watcher's own ingest writes. The receiver
does the rest (inject, surface, HANDED_OVER from transcript evidence). Nothing here claims a
message was seen: the cursor means "stored in the receiver", and it advances for a topic
only once every message on it was stored (store_message is idempotent by id, so a re-read
after a partial failure stores nothing twice).

First run per topic: the cursor is seeded from the session-start seen-marker
(.context/working/session-alerts/remote_*.seen) so history already shown at session start is
not replayed. A topic with no marker relays only posts younger than --window-days.

Fail loudly: a hub or topic that could not be read is NOT CHECKED in the output, the run exits
2, its cursor does not move, and after --notice-after consecutive failed runs ONE notice per
outage is stored in the receiver, so the running session is told mail there may be waiting.

Run by scripts/notify-sidecar-supervisor.sh every sweep (no cron of its own, DP-3).
Exit: 0 every hub read (mail relayed or not); 1 a store failed; 2 a hub/topic not checked,
or the reader failed (tooling).
Seams: --mail-cmd / REMOTE_RELAY_MAIL_CMD, --state-dir, --seed-dir, PROJECT_ROOT (receiver
location), FRAMEWORK_ROOT (where lib/sidecar lives), REMOTE_RELAY_NOW_MS, plus every
remote-inbox-mail.py seam (REMOTE_INBOX_TL, REMOTE_INBOX_HUBS_TOML, ...).
"""
import argparse
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = Path(os.environ.get("PROJECT_ROOT") or HERE.parent.parent)


def now_ms():
    v = os.environ.get("REMOTE_RELAY_NOW_MS")
    return int(v) if v else int(time.time() * 1000)


def stamp():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def load_receiver():
    fw = Path(os.environ.get("FRAMEWORK_ROOT") or ROOT / ".agentic-framework")
    os.environ.setdefault("PROJECT_ROOT", str(ROOT))
    sys.path.insert(0, str(fw))
    from lib.sidecar import receiver  # noqa: E402  (vendored AEF; read, never edited)
    return receiver


def sanitize(s):
    return "".join(c if c.isalnum() else "_" for c in s)


def msg_id(hub, topic, offset):
    return "remote-" + hashlib.sha256(f"{hub}|{topic}|{offset}".encode()).hexdigest()[:16]


def seed(state_dir, seed_dir):
    """Copy session-start seen-markers this relay has no cursor for yet."""
    if not seed_dir.is_dir():
        return
    for f in seed_dir.glob("remote_*.seen"):
        dst = state_dir / f.name
        if not dst.exists():
            shutil.copyfile(f, dst)


def envelope(row):
    hub, topic, off = row["hub"], row["topic"], row["offset"]
    origin = (f"(relayed from our inbox on remote hub {hub}, topic {topic}, offset {off}; "
              f"T-3750. The peer may read replies on that hub.)")
    return {
        "client_msg_id": msg_id(hub, topic, off),
        "from": row.get("from") or "?",
        "from_circuit": None,
        "conversation_id": row.get("conversation_id") or None,
        "body": origin + "\n\n" + str(row.get("body") or row.get("preview") or ""),
        "urgent": False,
        "via": "hub-topic",
        "hub_topic": topic,
        "hub_offset": off,
        "hub_ts": row.get("ts"),
        "remote_hub": hub,
    }


def outage_notice(hub, profiles, since, error):
    mid = "remote-outage-" + hashlib.sha256(f"{hub}|{since}".encode()).hexdigest()[:16]
    return mid, {
        "client_msg_id": mid,
        "from": "010-termlink-remote-inbox-relay",
        "conversation_id": None,
        "notice": True,
        "urgent": False,
        "via": "local-notice",
        "body": (f"NOT CHECKED: our inbox on remote hub {hub} ({','.join(profiles)}) could not be "
                 f"read since {since}: {error}. Peer mail there may be waiting and is NOT shown "
                 f"until the hub answers again. Check: termlink channel list --prefix inbox: "
                 f"--hub {hub} (T-3750 remote-inbox relay)"),
    }


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--state-dir", default=str(ROOT / ".context/working/remote-inbox-relay"))
    p.add_argument("--seed-dir", default=str(ROOT / ".context/working/session-alerts"))
    p.add_argument("--mail-cmd", default=os.environ.get(
        "REMOTE_RELAY_MAIL_CMD", f"python3 {HERE / 'remote-inbox-mail.py'}"))
    p.add_argument("--window-days", type=float, default=14)
    p.add_argument("--notice-after", type=int, default=2,
                   help="consecutive failed runs before the one outage notice (default 2)")
    p.add_argument("--quiet", action="store_true", help="print only when something happened")
    a = p.parse_args()

    state_dir = Path(a.state_dir)
    state_dir.mkdir(parents=True, exist_ok=True)
    seed(state_dir, Path(a.seed_dir))
    status_path = state_dir / "status.json"
    try:
        status = json.loads(status_path.read_text())
    except (OSError, ValueError):
        status = {}
    outages = status.get("outages") or {}

    cmd = shlex.split(a.mail_cmd) + ["--mode", "unseen", "--state-dir", str(state_dir), "--full-body"]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300, stdin=subprocess.DEVNULL)
        out = json.loads(proc.stdout)
        if not isinstance(out, dict) or not isinstance(out.get("hubs"), list):
            raise ValueError("unrecognised shape")
    except Exception as e:  # noqa: BLE001 - could not look is never "no mail"
        print(f"{stamp()} remote-inbox-relay: NOT CHECKED: reader failed ({type(e).__name__}: {e})"[:400])
        status.update(last_run=stamp(), last_rc=2, error=f"reader failed: {e}"[:300])
        status_path.write_text(json.dumps(status, indent=1))
        return 2

    receiver = load_receiver()
    cutoff = now_ms() - a.window_days * 86400000
    failed_topics = set()
    relayed, store_failed, lines = [], 0, []
    for row in out.get("mail") or []:
        if (row.get("ts") or 0) < cutoff:
            continue  # older than the window: the unacted-mail sweep's job, not live mail
        env = envelope(row)
        ok, err = receiver.store_message(env["client_msg_id"], env)
        if ok:
            receiver.record_event(env["client_msg_id"], "INGESTED_FROM_REMOTE_HUB", hub=row["hub"],
                                  topic=row["topic"], offset=row["offset"], hub_ts=row.get("ts"))
            relayed.append(row)
        else:
            store_failed += 1
            failed_topics.add((row["hub"], row["topic"]))
            lines.append(f"STORE FAILED {row['hub']} {row['topic']} offset {row['offset']}: {err}")

    # Cursor: only for topics whose every message was stored.
    for mk in out.get("marks") or []:
        key = mk.get("key", "")
        if any(key == "remote_" + sanitize(f"{h}_{t}") for h, t in failed_topics):
            continue
        tmp = state_dir / f"{key}.seen.tmp"
        tmp.write_text(f"{int(mk['offset'])}\n")
        os.replace(tmp, state_dir / f"{key}.seen")

    not_checked = [h for h in out["hubs"] if h.get("status") != "ok"]
    down = {h["hub"] for h in not_checked}
    for h in not_checked:
        o = outages.get(h["hub"]) or {"since": stamp(), "runs": 0, "notified": False}
        o["runs"] += 1
        o["error"] = (h.get("error") or "")[:300]
        if o["runs"] >= a.notice_after and not o["notified"]:
            mid, env = outage_notice(h["hub"], h.get("profiles") or [], o["since"], o["error"])
            ok, err = receiver.store_message(mid, env)
            o["notified"] = bool(ok)
            if not ok:
                lines.append(f"outage notice for {h['hub']} not stored: {err}")
        outages[h["hub"]] = o
        # Said on the first failed run, when the notice fires, then hourly at the 5-min cadence:
        # loud, but a hub down for a week does not write 2000 identical log lines.
        if a.quiet and o["runs"] not in (1, a.notice_after) and o["runs"] % 12:
            continue
        lines.append(f"NOT CHECKED {h['hub']} ({','.join(h.get('profiles') or [])}) run {o['runs']} "
                     f"since {o['since']}: {h.get('error')}"[:400])
    for hub in [k for k in outages if k not in down]:
        lines.append(f"recovered {hub} (was down since {outages[hub]['since']})")
        del outages[hub]
    if out.get("error"):
        lines.append(f"NOT CHECKED: {out['error']}")

    rc = 1 if store_failed else (2 if (not_checked or out.get("error")) else 0)
    for r in relayed:
        lines.insert(0, f"relayed {r['hub']} {r['topic']} offset {r['offset']} from {r.get('from')} "
                        f"as {msg_id(r['hub'], r['topic'], r['offset'])}")
    status.update(last_run=stamp(), last_rc=rc, outages=outages, error=out.get("error") or "",
                  hubs=[{k: h.get(k) for k in ("hub", "profiles", "status", "error")} for h in out["hubs"]],
                  relayed_total=int(status.get("relayed_total") or 0) + len(relayed))
    tmp = state_dir / "status.json.tmp"
    tmp.write_text(json.dumps(status, indent=1))
    os.replace(tmp, status_path)
    if lines or not a.quiet:
        print(f"{stamp()} remote-inbox-relay: rc={rc} hubs={len(out['hubs'])} relayed={len(relayed)}"
              f" not_checked={len(not_checked)}")
        for ln in lines:
            print("  " + ln)
    return rc


if __name__ == "__main__":
    sys.exit(main())
