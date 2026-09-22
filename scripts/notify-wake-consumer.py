#!/usr/bin/env python3
"""T-3061 — notify-rail wake consumer for ALLOWLIST-CONSTRAINED agents.

Why a second implementation instead of just using notify-wake-consumer.sh:
`termlink register --shell --allowed-commands` is a real security boundary, and
the AEF agent's is "fw,cat,ls,head,grep,tail,wc,date,python3". `bash` is not on
it, so the shell consumer literally cannot be launched there (JSON-RPC -32005).
`python3` is, so this is the form that runs inside that sandbox.

WHAT THIS DELIBERATELY DOES NOT DO
----------------------------------
It does not shell out to `termlink` to post a reply. python3 could trivially do
that via subprocess, and it would "work" — by routing around the very allowlist
the operator configured on a remotely-reachable agent. An agent that uses an
allowed interpreter to run disallowed commands has defeated the control, not
satisfied it. Whether the AEF agent may speak on the bus is the operator's call
(it means adding `termlink` to --allowed-commands), not something a test script
should decide by being clever.

So the wake action here is a FILE WRITE into a path the unit already grants via
ReadWritePaths. That is enough to prove the property under test — mail lands, the
consumer notices, something observable happens — without widening anyone's
privileges to prove it.

TRIGGER DISCIPLINE (the same lesson the shell sibling records)
-------------------------------------------------------------
`latest_topic` is STICKY: it keeps naming a topic long after that mail was
handled. Triggering on its presence re-fires forever on stale state. The trigger
is the flag's own `ts` advancing past our start, which proves the sidecar cycled
after we began watching.

A stale producer heartbeat exits 3 (DEAF) rather than waiting quietly, because a
consumer blocked on a dead producer is indistinguishable from one with nothing
to do — the exact ambiguity the rail's DEAF verdict exists to remove.

Exit: 0 woke · 2 usage · 3 producer DEAF · 4 timed out with no wake
"""
import argparse
import json
import os
import sys
import time


def read_kv(path):
    out = {}
    try:
        with open(path) as fh:
            for line in fh:
                line = line.strip()
                if "=" in line:
                    k, v = line.split("=", 1)
                    out[k] = v
    except OSError:
        return None
    return out


def read_int(path):
    try:
        with open(path) as fh:
            digits = "".join(c for c in fh.read() if c.isdigit())
        return int(digits) if digits else None
    except OSError:
        return None


def main():
    ap = argparse.ArgumentParser(add_help=True)
    ap.add_argument("--agent-id", required=True)
    ap.add_argument("--notify-dir", default=os.path.expanduser("~/.termlink/notify"))
    ap.add_argument("--topic-filter", default="")
    ap.add_argument("--marker", required=True,
                    help="file to write on wake (must be under the unit's ReadWritePaths)")
    ap.add_argument("--timeout", type=int, default=120)
    ap.add_argument("--interval", type=int, default=2)
    ap.add_argument("--hb-max-age", type=int, default=90)
    args = ap.parse_args()

    if args.interval < 1:
        print("notify-wake-consumer: --interval must be >= 1", file=sys.stderr)
        return 2

    flag = os.path.join(args.notify_dir, f"{args.agent_id}.flag")
    beat = os.path.join(args.notify_dir, f"{args.agent_id}.heartbeat")

    start_ms = int(time.time() * 1000)
    deadline = time.time() + args.timeout
    print(f"watching {flag} since={start_ms} timeout={args.timeout}s", flush=True)

    while time.time() < deadline:
        hb = read_int(beat)
        if hb is None:
            print(f"DEAF — no heartbeat at {beat}", file=sys.stderr)
            return 3
        age = (int(time.time() * 1000) - hb) / 1000.0
        if age > args.hb_max_age:
            print(f"DEAF — producer heartbeat {age:.0f}s stale (>{args.hb_max_age}s)",
                  file=sys.stderr)
            return 3

        kv = read_kv(flag)
        if kv:
            try:
                ts = int(kv.get("ts", "0"))
                pending = int(kv.get("pending", "0"))
            except ValueError:
                ts, pending = 0, 0
            topic = kv.get("latest_topic", "")
            fresh = ts > start_ms
            wanted = (not args.topic_filter) or (topic == args.topic_filter)
            if fresh and pending > 0 and topic and wanted:
                payload = {
                    "woke_at_ms": int(time.time() * 1000),
                    "by": args.agent_id,
                    "topic": topic,
                    "pending": pending,
                    "flag_ts": ts,
                    "latency_ms": int(time.time() * 1000) - start_ms,
                }
                try:
                    with open(args.marker, "w") as fh:
                        json.dump(payload, fh)
                except OSError as exc:
                    print(f"wake fired but marker unwritable: {exc}", file=sys.stderr)
                    return 2
                print("WOKE " + json.dumps(payload), flush=True)
                return 0
        time.sleep(args.interval)

    print(f"no wake within {args.timeout}s", flush=True)
    return 4


if __name__ == "__main__":
    sys.exit(main())
