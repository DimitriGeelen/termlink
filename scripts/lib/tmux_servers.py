"""T-3552: enumerate the tmux servers TermLink sessions can live on.

Since T-3552 each project's sessions run on their own server (`tmux -L tl-<slug>-<hash8>`, see
crates/termlink-session/src/tmux_server.rs), so a detector that asks only the DEFAULT server
(`tmux list-panes -a`) no longer sees them. This module lists the default server plus every
socket in this uid's tmux directory whose name passes the same validation the Rust side applies
before any `-L`. A name that fails it is never passed to tmux.
"""
import os
import re
import subprocess

VALID = re.compile(r"^(tl-host|tl-[a-z0-9-]{1,24}-[0-9a-f]{8})$")


def socket_dir(uid=None):
    base = os.environ.get("TMUX_TMPDIR") or "/tmp"
    return os.path.join(base, f"tmux-{os.getuid() if uid is None else uid}")


def termlink_servers(sdir=None):
    """Sorted `-L` names of TermLink servers present in `sdir` (default: this uid's tmux dir)."""
    sdir = sdir or socket_dir()
    try:
        names = os.listdir(sdir)
    except OSError:
        return []
    return sorted(n for n in names if VALID.match(n))


def run_on_each(args, sdir=None, timeout=10):
    """Run `tmux [-L name] <args>` on the default server and every TermLink server.

    Returns [(server, stdout or None)], server None for the default. A server that does not
    answer yields None, so a caller can report it rather than treat it as empty."""
    out = []
    for server in [None] + termlink_servers(sdir):
        cmd = ["tmux"] + (["-L", server] if server else []) + list(args)
        try:
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
            out.append((server, r.stdout if r.returncode == 0 else ("" if server is None else None)))
        except (OSError, subprocess.SubprocessError):
            out.append((server, None))
    return out
