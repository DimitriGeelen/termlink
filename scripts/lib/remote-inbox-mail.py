#!/usr/bin/env python3
"""T-3653 — peer mail on OUR inboxes on REMOTE hubs.

Why: every mail check we had (session-start-alerts.sh, check-unacted-mail.sh, the W1
sidecars) read only the local hub. A peer replying from its own hub writes to our inbox
ON ITS HUB (ring20 lesson 5.11): `inbox:22c19fedafd73da2/010-termlink` on .122 held four
ring20 follow-ups for 5 days (2026-10-27..11-02) until the operator relayed them.

What: for every hub in ~/.termlink/hubs.toml that is not this host's own hub, list the
inbox topics addressed to us (a path segment after the hub id equals our project id or an
alias, or starts with an alias prefix: inbox:<hub>/010-termlink, inbox:<hub>/<sender>/
010-termlink, inbox:<hub>/010-termlink/<sender>, inbox:<hub>/claude-termlink*) and read
the peer posts on them (receipts, reactions and our own posts are not mail).

Modes:
  unseen   posts after this helper's own per-topic seen-marker (session start). Markers
           are returned in "marks"; the caller writes them on --mark-seen.
  unacted  posts older than --threshold-hours and younger than --window-days, unless the
           offset is recorded in the handled file as "<topic>@<hub> <offset> <date> <what>".

Output: one JSON object {hubs:[{hub, profiles, status, error, topics}], mail:[...], marks:[...]}.
A hub that cannot be listed, or a topic that cannot be read, is status "not-checked" with
the reason: never reported clean. Every termlink call is bounded by --timeout seconds and
hubs are read in parallel, so a down hub cannot hang session start.
Exit: 0 nothing found and every hub read; 1 mail found; 2 nothing found but a hub (or a
topic) could not be read, or the hubs file could not be read.

--full-body (T-3750) adds "body" to each mail row: scripts/lib/remote-inbox-relay.py stores it in the
sidecar receiver so the running session sees the message, not a 120-character preview.

Seams (fixtures): --tl / REMOTE_INBOX_TL, --hubs-toml / REMOTE_INBOX_HUBS_TOML,
REMOTE_INBOX_LOCAL_HOSTS (hosts that are this hub; default loopback + `hostname -I`),
REMOTE_INBOX_TIMEOUT (default 10), REMOTE_INBOX_ALIAS_PREFIXES (default claude-termlink).
"""
import argparse
import base64
import concurrent.futures as cf
import json
import os
import re
import signal
import subprocess
import sys
import time

try:
    import tomllib
except ImportError:  # pragma: no cover - python < 3.11
    tomllib = None

NOT_MAIL_TYPES = {"receipt", "reaction", "redaction", "edit", "topic_metadata"}


def sanitize(s):
    return re.sub(r"[^A-Za-z0-9]", "_", s)


def local_hosts():
    env = os.environ.get("REMOTE_INBOX_LOCAL_HOSTS")
    if env is not None:
        return set(env.split())
    hosts = {"127.0.0.1", "localhost", "::1", "0.0.0.0"}
    try:
        out = subprocess.run(["hostname", "-I"], capture_output=True, text=True, timeout=5).stdout
        hosts.update(out.split())
    except Exception:
        pass
    return hosts


def host_of(addr):
    if addr.startswith("["):
        return addr[1:].split("]")[0]
    return addr.rsplit(":", 1)[0] if ":" in addr else addr


def remote_hubs(path, locals_):
    """[(addr, [profile names])] for TCP hubs that are not this host. Raises on unreadable file."""
    with open(os.path.expanduser(path), "rb") as fh:
        data = tomllib.load(fh)
    hubs = {}
    for name, prof in (data.get("hubs") or {}).items():
        addr = (prof or {}).get("address") if isinstance(prof, dict) else None
        if not addr or "/" in addr or host_of(addr) in locals_:
            continue  # unix socket path or our own hub
        hubs.setdefault(addr, []).append(name)
    return sorted(hubs.items())


def is_ours(topic, names, prefixes):
    if not topic.startswith("inbox:"):
        return False
    parts = topic[len("inbox:"):].split("/")
    return len(parts) >= 2 and any(p in names or any(p.startswith(x) for x in prefixes) for p in parts[1:])


def is_mail(m, names):
    if not isinstance(m, dict) or m.get("offset") is None:
        return False
    t = str(m.get("msg_type") or "")
    meta = m.get("metadata") or {}
    if t in NOT_MAIL_TYPES or t.split(".")[-1] in NOT_MAIL_TYPES:
        return False
    if meta.get("kind") == "receipt" or meta.get("stage") or meta.get("up_to") or meta.get("receipt_for"):
        return False
    return (meta.get("from_project") or "") not in names


def run(cmd, timeout):
    # Own process group, killed whole on timeout: a wrapper's child holding the pipe
    # open must not stretch the bound (subprocess.run kills only the direct child).
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                         stdin=subprocess.DEVNULL, start_new_session=True)
    try:
        out, err = p.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except OSError:
            pass
        p.communicate()
        raise RuntimeError(f"no answer within {timeout:g}s")
    if p.returncode != 0:
        raise RuntimeError((err or out or f"exit {p.returncode}").strip().replace("\n", " ")[:200])
    return out


def read_hub(addr, profiles, a, names, prefixes):
    hub = {"hub": addr, "profiles": profiles, "status": "ok", "error": "", "topics": []}
    mail, marks = [], []
    try:
        listing = json.loads(run([a.tl, "channel", "list", "--prefix", "inbox:", "--hub", addr, "--json"], a.timeout))
    except Exception as e:  # noqa: BLE001 - every failure is "could not look"
        hub.update(status="not-checked", error=f"channel list failed: {e}"[:240])
        return hub, mail, marks
    topics = listing.get("topics", []) if isinstance(listing, dict) else listing
    ours = sorted(t.get("name", "") for t in topics if isinstance(t, dict) and is_ours(t.get("name", ""), names, prefixes))
    hub["topics"] = ours
    now = time.time() * 1000
    handled = a.handled_set
    errs = []
    for topic in ours:
        key = "remote_" + sanitize(f"{addr}_{topic}")
        cursor = 0
        if a.mode == "unseen":
            try:
                seen = int(re.sub(r"[^0-9]", "", open(os.path.join(a.state_dir, key + ".seen")).read()) or -1)
            except (OSError, ValueError):
                seen = -1
            cursor = seen + 1
        try:
            out = run([a.tl, "channel", "subscribe", topic, "--hub", addr, "--cursor", str(cursor),
                       "--limit", "1000", "--json"], a.timeout)
        except Exception as e:  # noqa: BLE001
            errs.append(f"{topic} could not be read: {e}"[:240])
            continue
        last = None
        for line in out.splitlines():
            line = line.strip()
            if not line.startswith("{"):
                continue
            try:
                m = json.loads(line)
            except ValueError:
                continue
            if isinstance(m, dict) and isinstance(m.get("offset"), int):
                last = m["offset"] if last is None else max(last, m["offset"])
            if not is_mail(m, names):
                continue
            meta = m.get("metadata") or {}
            ts = m.get("ts") or 0
            age_h = (now - ts) / 3.6e6
            if a.mode == "unacted":
                if not (a.threshold_hours <= age_h <= a.window_days * 24):
                    continue
                if (f"{topic}@{addr}", str(m["offset"])) in handled:
                    continue
            body = base64.b64decode(m.get("payload_b64") or b"").decode("utf-8", "replace")
            mail.append({"hub": addr, "topic": topic, "offset": m["offset"], "ts": ts,
                         "from": meta.get("from_project") or meta.get("from_agent") or m.get("sender_id") or "?",
                         "conversation_id": meta.get("conversation_id") or "",
                         "age_min": int((now - ts) / 60000), "age_hours": int(age_h),
                         "preview": " ".join(body.split())[:120],
                         "msg_type": str(m.get("msg_type") or ""),
                         **({"body": body} if a.full_body else {})})
        if last is not None:
            marks.append({"key": key, "offset": last})
    if errs:
        hub.update(status="not-checked", error="; ".join(errs))
    return hub, mail, marks


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--mode", choices=("unseen", "unacted"), required=True)
    p.add_argument("--tl", default=os.environ.get("REMOTE_INBOX_TL", "termlink"))
    p.add_argument("--hubs-toml", default=os.environ.get("REMOTE_INBOX_HUBS_TOML", "~/.termlink/hubs.toml"))
    p.add_argument("--self", dest="self_project", default=os.environ.get("FW_SIDECAR_SELF_PROJECT", "010-termlink"))
    p.add_argument("--aliases", default=os.environ.get("FW_SIDECAR_SELF_ALIASES", "termlink"))
    p.add_argument("--timeout", type=float, default=float(os.environ.get("REMOTE_INBOX_TIMEOUT", "10")))
    p.add_argument("--state-dir", default="")
    p.add_argument("--threshold-hours", type=float, default=24)
    p.add_argument("--window-days", type=float, default=14)
    p.add_argument("--handled", default="")
    p.add_argument("--full-body", action="store_true",
                   help="T-3750: add each message's full body (the live relay stores it)")
    a = p.parse_args()
    if a.mode == "unseen" and not a.state_dir:
        p.error("--mode unseen needs --state-dir")
    names = {a.self_project, *a.aliases.split()}
    prefixes = [x for x in os.environ.get("REMOTE_INBOX_ALIAS_PREFIXES", "claude-termlink").split() if x]
    a.handled_set = set()
    if a.handled:
        try:
            for line in open(a.handled):
                f = line.split()
                if len(f) >= 2 and not f[0].startswith("#"):
                    a.handled_set.add((f[0], f[1]))
        except OSError:
            pass  # no handled file yet: nothing dismissed
    result = {"hubs": [], "mail": [], "marks": [], "error": ""}
    try:
        if tomllib is None:
            raise RuntimeError("python tomllib unavailable (needs 3.11+)")
        hubs = remote_hubs(a.hubs_toml, local_hosts())
    except Exception as e:  # noqa: BLE001
        result["error"] = f"could not read hubs file {a.hubs_toml}: {e}"[:240]
        print(json.dumps(result))
        return 2
    if hubs:
        with cf.ThreadPoolExecutor(max_workers=min(8, len(hubs))) as ex:
            futs = [ex.submit(read_hub, addr, profs, a, names, prefixes) for addr, profs in hubs]
            for f in futs:
                hub, mail, marks = f.result()
                result["hubs"].append(hub)
                result["mail"].extend(mail)
                result["marks"].extend(marks)
    result["mail"].sort(key=lambda r: -r["ts"])
    print(json.dumps(result))
    if result["mail"]:
        return 1
    return 2 if any(h["status"] != "ok" for h in result["hubs"]) else 0


if __name__ == "__main__":
    sys.exit(main())
