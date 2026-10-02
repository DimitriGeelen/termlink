#!/usr/bin/env python3
"""T-3325 — five-level circuit addressing for the notify sidecar.

Every agent on a host signs as one TermLink identity unless it has its own key
(T-1448), so `dm:<fp>` and the project inbox are shared. The sidecar used to wake
its agent on ANY unread message — including mail meant for a co-resident agent
(framework:pickup 257 item 4). This module decides, per message, whether it is
for us.

ADDRESS (operator ruling T-3325 Q1 = C, 2026-10-03). The agreed five-level circuit
(AEF D-599 / T-3433): host / hub / project / session / agent. Carried in
`metadata.to_circuit`, mirroring the `from_circuit` peers already send. Two
grammars are READ:

  path form   //host/hub/project[/session][/@agent]      (from_circuit as sent today)
              hub/project[/session][/@agent]              (no leading // = starts at hub)
  V9 form     aef::host=H::hub=X::project=P::session=S::@agent::

The agent segment is marked with `@` in both forms; that is what keeps
`project/session` and `project/@agent` apart in the path form.
When `to_circuit` is absent, a bare `metadata.to_project` (what
`termlink agent contact name:project` sets) is read as a project-level address.

DECISION, level by level from level 1:
  - no address                         -> unaddressed  (wakes: old senders keep working)
  - host / hub / project named and different from ours -> foreign (does not wake)
  - agent named and equal to ours      -> mine
  - agent named, different, and LIVE here (fresh sidecar heartbeat)  -> foreign
  - agent named, different, NOT live   -> fallback (wakes: the ladder falls back to the
                                          deepest level that resolves, the project, so
                                          nothing is dropped silently)
  - address stops above the agent and every named level matches -> mine
The session level is not used: the sidecar serves an agent, not a session, and has
no session identity to compare; a named session never makes a message foreign.
An address that cannot be parsed is treated as unaddressed: a spurious wake is
cheap, a silent drop is the defect this exists to remove.

USAGE (the sidecar pipes one topic's unread envelopes as NDJSON on stdin):
  circuit.py classify --first-unread N --self-host H --self-hub X \
      --self-project P --self-agent A [--live a,b,c]
prints one JSON object:
  {"mine":n,"unaddressed":n,"fallback":n,"foreign":n,"wake":n,
   "foreign_min_offset":int|null,"newest_wake_offset":int|null,"unparseable":n}
Exit 0 on success, 2 on bad usage. The sidecar treats any failure as "classifier
unavailable" and falls back to the raw unread count.
"""
import argparse
import json
import sys

LEVELS = ("host", "hub", "project", "session", "agent")
# Mirror of termlink-cli UNREAD_META_TYPES (channel.rs) so our count agrees with
# `channel unread`'s unread_count.
META_TYPES = {"receipt", "reaction", "redaction", "edit", "topic_metadata"}


def parse(addr):
    """Return {level: value} or None when unaddressed/unparseable."""
    if not isinstance(addr, str):
        return None
    s = addr.strip()
    if not s:
        return None
    d = {}
    if "::" in s:
        for piece in s.split("::"):
            piece = piece.strip()
            if not piece or piece == "aef":
                continue
            if piece.startswith("@"):
                if "agent" in d or len(piece) < 2:
                    return None
                d["agent"] = piece[1:]
                continue
            if "=" in piece:
                k, v = piece.split("=", 1)
                k, v = k.strip(), v.strip()
                if k == "agent":
                    v = v.lstrip("@")
                if k in LEVELS and v and k not in d:
                    d[k] = v
                    continue
            return None
        return d or None
    if s.startswith("//"):
        parts, keys = s[2:].split("/"), ["host", "hub", "project"]
    else:
        parts, keys = s.split("/"), ["hub", "project"]
    while parts and parts[-1] == "":
        parts.pop()
    if not parts or "" in parts:
        return None
    i = 0
    for p in parts:
        if p.startswith("@"):
            if "agent" in d or len(p) < 2:
                return None
            d["agent"] = p[1:]
            continue
        if "agent" in d:
            return None  # nothing may follow the agent
        if i < len(keys):
            d[keys[i]] = p
        elif "session" not in d:
            d["session"] = p
        else:
            return None
        i += 1
    return d or None


def _same(level, a, b):
    a, b = a.strip().lower(), b.strip().lower()
    if level == "host":
        return a == b or a.split(".")[0] == b.split(".")[0]
    if level == "hub":
        a, b = a.replace("sha256:", ""), b.replace("sha256:", "")
        return len(min(a, b, key=len)) >= 8 and (a.startswith(b) or b.startswith(a))
    return a == b


def decide(addr, me, live):
    """addr: parsed dict or None. me: {level: value}. live: set of agent ids."""
    if not addr:
        return "unaddressed"
    for level in ("host", "hub", "project"):
        if level in addr and me.get(level) and not _same(level, addr[level], me[level]):
            return "foreign"
    if "agent" in addr:
        if me.get("agent") and _same("agent", addr["agent"], me["agent"]):
            return "mine"
        if addr["agent"] in live:
            return "foreign"
        return "fallback"
    return "mine"


def address_of(meta):
    meta = meta or {}
    tc = meta.get("to_circuit")
    if isinstance(tc, str) and tc.strip():
        parsed = parse(tc)
        return parsed, parsed is None
    tp = meta.get("to_project")
    if isinstance(tp, str) and tp.strip():
        return {"project": tp.strip()}, False
    return None, False


def classify(lines, first_unread, me, live):
    out = {"mine": 0, "unaddressed": 0, "fallback": 0, "foreign": 0, "wake": 0,
           "foreign_min_offset": None, "newest_wake_offset": None, "unparseable": 0}
    for line in lines:
        line = line.strip()
        if not line:
            continue
        try:
            env = json.loads(line)
        except ValueError:
            continue
        if not isinstance(env, dict):
            continue
        off = env.get("offset")
        if not isinstance(off, int) or off < first_unread:
            continue
        if env.get("msg_type") in META_TYPES:
            continue
        addr, bad = address_of(env.get("metadata"))
        if bad:
            out["unparseable"] += 1
        verdict = decide(addr, me, live)
        out[verdict] += 1
        if verdict == "foreign":
            if out["foreign_min_offset"] is None or off < out["foreign_min_offset"]:
                out["foreign_min_offset"] = off
        else:
            out["wake"] += 1
            if out["newest_wake_offset"] is None or off > out["newest_wake_offset"]:
                out["newest_wake_offset"] = off
    return out


def main(argv):
    ap = argparse.ArgumentParser(prog="circuit.py")
    sub = ap.add_subparsers(dest="cmd")
    c = sub.add_parser("classify")
    c.add_argument("--first-unread", type=int, required=True)
    for level in ("host", "hub", "project", "agent"):
        c.add_argument("--self-" + level, default="")
    c.add_argument("--live", default="")
    p = sub.add_parser("parse")
    p.add_argument("address")
    try:
        a = ap.parse_args(argv)
    except SystemExit:
        return 2
    if a.cmd == "parse":
        print(json.dumps(parse(a.address)))
        return 0
    if a.cmd != "classify":
        ap.print_usage(sys.stderr)
        return 2
    me = {k: getattr(a, "self_" + k) for k in ("host", "hub", "project", "agent")}
    me = {k: v for k, v in me.items() if v}
    live = {x for x in a.live.split(",") if x} - {me.get("agent", "")}
    print(json.dumps(classify(sys.stdin, a.first_unread, me, live)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
