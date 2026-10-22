#!/usr/bin/env python3
"""T-3551 (SEV-1 T-3549): find processes that carry another project's variables.

A project variable (PROJECT_VARS) is FOREIGN when its value lies in project B while the session it
belongs to works in a different project A (the session's registered cwd). A project root is the nearest ancestor
holding `.framework.yaml`, else `.git`; a git worktree counts as its main checkout. Unknown is never a finding: a value outside every project
(a test temp dir), a relative value, or a process outside every project is not judged. This is
the same rule as `termlink_session::child_env::foreign_project_vars`, which `termlink register`
enforces at start; this script finds the processes that started before it existed.

Scanned: every registered session's root process (pid from `termlink list --json`), judged
against the session's registered cwd, plus the GLOBAL environment of the default tmux server and of every per-project TermLink server
(T-3552), which must hold no project variable at all
(every session started on that server inherits it).

Output: one JSON object on stdout. Exit 0 = scanned (findings or not), 2 = could not scan
(session list or process table unreadable). Firing is decided by the calling canary.
Seams: --list-json FILE, --proc DIR, --tmux-env FILE (use an empty file for "no server").
"""
import argparse
import json
import os
import subprocess
import sys

import tmux_servers  # T-3552: per-project servers (same directory)

PROJECT_VARS = ["PROJECT_ROOT", "TASKS_DIR", "CONTEXT_DIR", "CLAUDE_PROJECT_DIR", "_FW_PATHS_DERIVED_BY"]
# The server's global environment must not name any project at all, FRAMEWORK_ROOT included.
SERVER_VARS = PROJECT_VARS + ["FRAMEWORK_ROOT", "FW_ROUTED_FROM"]


def main_checkout(root):
    """A git worktree belongs to the project of its main checkout: its `.git` is a file
    `gitdir: <main>/.git/worktrees/<name>`."""
    g = os.path.join(root, ".git")
    if os.path.isfile(g):
        try:
            with open(g) as f:
                line = f.readline().strip()
        except OSError:
            return root
        if line.startswith("gitdir:") and "/.git/worktrees/" in line:
            return line.split(":", 1)[1].strip().split("/.git/worktrees/", 1)[0]
    return root


def project_of(path):
    if not path or not path.startswith("/"):
        return None
    for marker in (".framework.yaml", ".git"):
        p = os.path.normpath(path)
        while True:
            if os.path.exists(os.path.join(p, marker)):
                return main_checkout(p)
            parent = os.path.dirname(p)
            if parent == p:
                break
            p = parent
    return None


def read_cwd(proc, pid):
    p = os.path.join(proc, str(pid), "cwd")
    if os.path.islink(p):
        return os.readlink(p)
    with open(p) as f:  # fixture form: a text file holding the path
        return f.read().strip()


def read_environ(proc, pid):
    with open(os.path.join(proc, str(pid), "environ"), "rb") as f:
        raw = f.read()
    env = {}
    for item in raw.split(b"\0"):
        if b"=" in item:
            k, v = item.split(b"=", 1)
            env[k.decode(errors="replace")] = v.decode(errors="replace")
    return env


def foreign(env, cwd):
    own = project_of(cwd)
    if own is None:
        return []
    out = []
    for k in PROJECT_VARS:
        v = env.get(k)
        vp = project_of(v) if v else None
        if vp is not None and vp != own:
            out.append({"name": k, "value": v, "value_project": vp, "own_project": own})
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--list-json")
    ap.add_argument("--proc", default="/proc")
    ap.add_argument("--tmux-env")
    a = ap.parse_args()

    try:
        if a.list_json:
            with open(a.list_json) as f:
                sessions = json.load(f)
        else:
            r = subprocess.run(["termlink", "list", "--json"], capture_output=True, text=True, timeout=60)
            if r.returncode != 0:
                raise RuntimeError(r.stderr.strip()[:200])
            sessions = json.loads(r.stdout)
        if isinstance(sessions, dict):
            sessions = sessions.get("sessions", [])
        if not os.path.isdir(a.proc):
            raise RuntimeError(f"process table {a.proc} not readable")
    except Exception as e:  # noqa: BLE001 — any failure here means we could not look
        print(f"env-leak-scan: cannot scan: {e}", file=sys.stderr)
        return 2

    findings, scanned, unreadable = [], 0, 0
    for s in sessions:
        pid = s.get("pid")
        if not isinstance(pid, int):
            continue
        # Only the session's ROOT process is judged, against the session's registered cwd.
        # A leak arrives through it (the register process a polluted tmux server or shell
        # started), and every dispatched worker registers as a session of its own. Descendants
        # are not judged: a test harness deliberately points its children at a fixture
        # project, and a language server runs from its install dir with the session's
        # PROJECT_ROOT; both are set on purpose, not inherited.
        session_cwd = (s.get("metadata") or {}).get("cwd")
        try:
            env = read_environ(a.proc, pid)
            if not session_cwd:
                session_cwd = read_cwd(a.proc, pid)
        except OSError:
            unreadable += 1
            continue
        scanned += 1
        for f in foreign(env, session_cwd):
            f.update({"pid": pid, "session": s.get("display_name") or s.get("id"), "session_pid": pid})
            findings.append(f)

    server = []
    if a.tmux_env is not None:
        with open(a.tmux_env) as f:
            texts = [(None, f.read())]
    else:
        # T-3552: the default server AND every per-project TermLink server. No server, or a
        # server that does not answer, holds nothing global (and the zombie check reports it).
        texts = [(srv, t or "") for srv, t in tmux_servers.run_on_each(["show-environment", "-g"])]
    for srv, text in texts:
        for line in text.splitlines():
            if "=" in line and not line.startswith("-"):
                k, v = line.split("=", 1)
                if k in SERVER_VARS:
                    server.append({"name": k, "value": v, "server": srv or "default"})

    print(json.dumps({
        "sessions": len(sessions),
        "processes_scanned": scanned,
        "processes_unreadable": unreadable,
        "foreign": findings,
        "server_global": server,
    }))
    return 0


if __name__ == "__main__":
    sys.exit(main())
