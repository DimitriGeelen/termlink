#!/usr/bin/env python3
"""T-3296 (T-3291 S4) — shared detector for the session lifecycle leak.

Used by BOTH the daily canary (scripts/check-session-leak-freshness.sh) and the
operator's one-time reap (runme.sh, T-3291 S3b), so what is detected and what
is terminated cannot drift apart (the T-2404/T-2405 pattern).

Classifies every live `termlink register --shell` process:
  zombie    a tmux PANE process, older than --min-age, whose PTY shell has NO
            child processes and whose tmux session has no client attached
  unmanaged not a tmux pane (systemd-supervised agents, launcher children) —
            an idle shell is how an always-on endpoint waits; never a zombie
  busy      its shell has a child (an agent, a command) — never a zombie
  attached  a human/client is attached to its tmux session — never a zombie
  young     younger than --min-age — too soon to judge
  launcher_alive  named <prefix>-<pid> where <pid> is a live claude/claude-fw
            (claude-fw --termlink's claude-master-$$) — part of a live agent
Non-shell registers (`register --self`, plain endpoints) are counted and
never classified: they have no shell whose exit could end them.

Also counts orphaned data-plane sockets per sessions dir: `<id>.sock.data`
with no `<id>.json`, older than --orphan-age. After T-3293 the hub reaps
these within 10 minutes, so any found here means that reap is not running.

Inputs (seams for fixtures, PL-213):
  --ps FILE      output of `ps -eo pid,ppid,etimes,args --no-headers`
                 (default: run ps)
  --tmux FILE    output of `tmux list-panes -a -F '#{pane_pid} #{session_name}
                 #{session_attached}'` (default: run tmux; no server = no panes)
  --dir DIR      sessions dir to scan for orphans (repeatable)
Output: one JSON object on stdout. Exit 2 if ps cannot be read (fail-closed).
"""
import argparse
import json
import os
import subprocess
import sys
import time


def read_table(path, cmd):
    if path:
        with open(path) as f:
            return f.read()
    try:
        return subprocess.run(cmd, capture_output=True, text=True, timeout=30).stdout
    except (OSError, subprocess.SubprocessError):
        return None


def arg_after(args, flag):
    try:
        return args[args.index(flag) + 1]
    except (ValueError, IndexError):
        return None


def launcher_alive(name, procs):
    """True when `name` ends in -<pid> and that pid is a live claude/claude-fw."""
    tail = name.rsplit("-", 1)[-1]
    if not tail.isdigit():
        return False
    p = procs.get(int(tail))
    return bool(p) and "claude" in p[2]


def classify(ps_text, tmux_text, min_age):
    procs = {}
    children = {}
    for line in ps_text.splitlines():
        parts = line.split(None, 3)
        if len(parts) < 4 or not parts[0].isdigit():
            continue
        pid, ppid, etimes, args = int(parts[0]), int(parts[1]), int(parts[2]), parts[3]
        procs[pid] = (ppid, etimes, args)
        children.setdefault(ppid, []).append(pid)

    panes = {}
    for line in (tmux_text or "").splitlines():
        parts = line.split()
        if len(parts) >= 3 and parts[0].isdigit():
            panes[int(parts[0])] = (parts[1], int(parts[2]) if parts[2].isdigit() else 0)

    out = {"zombie": [], "busy": 0, "attached": 0, "young": 0, "non_shell": 0, "unmanaged": [], "launcher_alive": 0}
    for pid, (ppid, etimes, args) in sorted(procs.items()):
        argv = args.split()
        if len(argv) < 2 or not argv[0].endswith("termlink") or argv[1] != "register":
            continue
        if "--shell" not in argv:
            out["non_shell"] += 1
            continue
        shells = children.get(pid, [])
        pane = panes.get(pid)
        entry = {
            "pid": pid,
            "name": arg_after(argv, "--name") or "?",
            "age_s": etimes,
            "tmux_session": pane[0] if pane else None,
        }
        if not pane:
            # Not a tmux pane: a systemd-supervised agent, or a launcher's own
            # child. An idle shell there is how an always-on endpoint waits for
            # work (framework-agent-systemd, termlink-agent, email-archive), so
            # it is NEVER a zombie. Listed for visibility, never flagged or reaped.
            out["unmanaged"].append(entry)
        elif any(children.get(s) for s in shells):
            out["busy"] += 1
        elif pane[1] > 0:
            out["attached"] += 1
        elif launcher_alive(entry["name"], procs):
            # `claude-fw --termlink` names its session claude-master-<its pid>;
            # while that launcher lives the session is how its agent is reached,
            # even with the shell idle. Never a zombie.
            out["launcher_alive"] += 1
        elif etimes < min_age:
            out["young"] += 1
        else:
            out["zombie"].append(entry)
    return out


def orphans(dirs, orphan_age, now):
    result = {}
    for d in dirs:
        n = 0
        try:
            names = os.listdir(d)
        except OSError:
            result[d] = None  # unreadable/absent: reported, not counted as clean
            continue
        present = set(names)
        for name in names:
            if not name.endswith(".sock.data"):
                continue
            sid = name[: -len(".sock.data")]
            if not sid or f"{sid}.json" in present:
                continue
            try:
                if now - os.stat(os.path.join(d, name)).st_mtime >= orphan_age:
                    n += 1
            except OSError:
                pass
        result[d] = n
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ps")
    ap.add_argument("--tmux")
    ap.add_argument("--dir", action="append", default=[])
    ap.add_argument("--min-age", type=int, default=86400)
    ap.add_argument("--orphan-age", type=int, default=3600)
    a = ap.parse_args()

    ps_text = read_table(a.ps, ["ps", "-eo", "pid,ppid,etimes,args", "--no-headers"])
    if not ps_text:
        print("session-zombies: could not read the process table", file=sys.stderr)
        return 2
    tmux_text = read_table(
        a.tmux,
        ["tmux", "list-panes", "-a", "-F", "#{pane_pid} #{session_name} #{session_attached}"],
    )
    res = classify(ps_text, tmux_text, a.min_age)
    res["orphans"] = orphans(a.dir, a.orphan_age, time.time())
    res["min_age_s"] = a.min_age
    res["orphan_age_s"] = a.orphan_age
    print(json.dumps(res))
    return 0


if __name__ == "__main__":
    sys.exit(main())
