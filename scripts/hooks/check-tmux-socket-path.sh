#!/usr/bin/env bash
# T-3425 — PreToolUse(Bash) guard: refuse `tmux -S <path>` when <path> is not a socket.
#
# On 2026-10-12 an agent ran `tmux -S /proc/2908/cwd` to "look at" a tmux server.
# -S names a SOCKET PATH; /proc/<pid>/cwd resolved to `/`, tmux failed to create
# its socket there and still applied its socket chmod, clearing the execute bits:
# `/` went 0755 -> 0644, no non-root process could open any file, and the host
# would not boot (docs/reports/T-3425-root-chmod-incident.md).
#
# Refuses (exit 2, reason on stderr) when a tmux invocation in the command has
#   -S <path>  with <path> under /proc, or an existing path that is not a socket;
#   -L <name>  where <name> contains a `/` (-L is a socket NAME, never a path).
# A path that cannot be resolved statically ($VAR, `cmd`) is allowed unless it
# mentions /proc: refusing those would block every scratch-socket test here.
# Fail-open on unparseable input: this guard narrows one footgun, it is not a
# shell parser, and a guard that blocks on its own bugs gets switched off.
#
# Test seam: TMUX_GUARD_INPUT=<json> replaces stdin.
set -u
input="${TMUX_GUARD_INPUT:-$(cat)}"
TMUX_GUARD_JSON="$input" python3 - <<'PY'
import json, os, shlex, stat, sys

try:
    cmd = json.loads(os.environ["TMUX_GUARD_JSON"]).get("tool_input", {}).get("command", "")
except Exception:
    sys.exit(0)
if "tmux" not in cmd:
    sys.exit(0)
try:
    lex = shlex.shlex(cmd, posix=True, punctuation_chars=";&|()")
    lex.whitespace_split = True
    toks = list(lex)
except ValueError:
    sys.exit(0)

SEP = {";", "&&", "||", "|", "&", "(", ")", ";;"}
problems = []

def check_socket(path):
    if "/proc" in path:
        return f"-S {path}: a /proc path resolves to a real directory (/proc/<pid>/cwd is often /)"
    if "$" in path or "`" in path:
        return None
    if os.path.lexists(path):
        try:
            st = os.stat(path)
        except OSError:
            return None
        if not stat.S_ISSOCK(st.st_mode):
            kind = "directory" if stat.S_ISDIR(st.st_mode) else "file"
            return f"-S {path}: exists and is a {kind}, not a socket"
    return None

i = 0
while i < len(toks):
    if os.path.basename(toks[i]) == "tmux":
        j = i + 1
        while j < len(toks) and toks[j] not in SEP:
            t = toks[j]
            val = None
            if t in ("-S", "-L") and j + 1 < len(toks):
                opt, val = t, toks[j + 1]
                j += 1
            elif t.startswith("-S") and len(t) > 2:
                opt, val = "-S", t[2:]
            elif t.startswith("-L") and len(t) > 2:
                opt, val = "-L", t[2:]
            if val is not None:
                if opt == "-S":
                    p = check_socket(val)
                    if p:
                        problems.append(p)
                elif "/" in val:
                    problems.append(f"-L {val}: -L takes a socket name, not a path")
            j += 1
        i = j
    else:
        i += 1

if problems:
    sys.stderr.write(
        "BLOCKED (T-3425): tmux would create or chmod a path that is not its socket.\n"
        + "".join(f"  {p}\n" for p in problems)
        + "  On 2026-10-12, `tmux -S /proc/<pid>/cwd` stripped the execute bits from `/` and the host\n"
        "  could not boot. To read a process's environment use /proc/<pid>/environ; to talk to a\n"
        "  tmux server pass its real socket (see `ls -l /proc/<pid>/fd`).\n"
    )
    sys.exit(2)
sys.exit(0)
PY
