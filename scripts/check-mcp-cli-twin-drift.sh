#!/usr/bin/env bash
# T-2999 (value-review C-26): one-sided changes to _mcp/CLI helper twins.
#
# The T-2069 convention duplicates small pure helpers into termlink-mcp as
# `fn <name>_mcp` rather than sharing them across the crate boundary. 63 of the 67
# `_mcp` helpers have a same-named CLI twin. Duplication already produced parity
# drift that went unnoticed for weeks: one twin gains a field or a fix and the
# other does not.
#
# This lists every commit in a range that changed exactly ONE side of a twin pair.
# It is a REVIEW LIST, not a gate: a CLI-only presentation change is legitimate, so
# a hit means "look at the other twin", not "this is wrong". Its verdict depends on
# the range you give it, so it is deliberately not a `guard-layer: source` member.
#
# How a change is attributed: every changed line (added lines in the new version,
# removed lines in the old one) is mapped to the nearest preceding `fn` in that
# file version. Code between two fns is attributed to the earlier one, so read a
# hit as a pointer, not a proof.
#
# Usage: check-mcp-cli-twin-drift.sh [--range A..B] [--json]
#   default range: HEAD~50..HEAD
# Exit: 0 no one-sided change · 1 one-sided change(s) listed · 2 tooling error
#       (not a git repo, bad range) — never a vacuous clean.
# Test seams: TWIN_MCP_DIR / TWIN_CLI_DIR (default crates/termlink-{mcp,cli}/src)
set -uo pipefail

RANGE="HEAD~50..HEAD"
FORMAT=text
while [ $# -gt 0 ]; do
    case "$1" in
        --range) [ $# -ge 2 ] || { echo "check-mcp-cli-twin-drift: --range needs A..B" >&2; exit 2; }
                 RANGE="$2"; shift 2 ;;
        --json)  FORMAT=json; shift ;;
        -h|--help) sed -n '2,25p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-mcp-cli-twin-drift: unknown arg: $1" >&2; exit 2 ;;
    esac
done

git rev-parse --git-dir >/dev/null 2>&1 || { echo "check-mcp-cli-twin-drift: not a git repository" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "check-mcp-cli-twin-drift: python3 not found" >&2; exit 2; }
git rev-list --reverse "$RANGE" >/dev/null 2>&1 || { echo "check-mcp-cli-twin-drift: bad range: $RANGE" >&2; exit 2; }

MCP_DIR="${TWIN_MCP_DIR:-crates/termlink-mcp/src}" CLI_DIR="${TWIN_CLI_DIR:-crates/termlink-cli/src}" \
RANGE="$RANGE" FORMAT="$FORMAT" python3 - <<'PY'
import json, os, re, subprocess, sys

mcp_dir, cli_dir = os.environ["MCP_DIR"].rstrip("/"), os.environ["CLI_DIR"].rstrip("/")
FN = re.compile(r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+([A-Za-z0-9_]+)')
HUNK = re.compile(r'^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@')

def git(*args):
    r = subprocess.run(["git", *args], capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None

def fn_names(text):
    return {m.group(1) for m in (FN.match(l) for l in text.splitlines()) if m}

def tree_fns(rev, d):
    names = set()
    ls = git("ls-tree", "-r", "--name-only", rev, "--", d) or ""
    for path in ls.split():
        if path.endswith(".rs"):
            names |= fn_names(git("show", f"{rev}:{path}") or "")
    return names

def enclosing(lines, lineno):
    for i in range(min(lineno, len(lines)) - 1, -1, -1):
        m = FN.match(lines[i])
        if m:
            return m.group(1)
    return None

def touched(sha, d):
    """fn names whose bodies a commit changed under directory d."""
    out = set()
    parent = f"{sha}^"
    has_parent = git("rev-parse", "--verify", "-q", parent) is not None
    diff = git("show", "-U0", "--format=", sha, "--", d) or ""
    path = oldpath = None
    new_cache, old_cache = {}, {}
    for line in diff.splitlines():
        if line.startswith("+++ "):
            p = line[4:]
            path = p[2:] if p.startswith("b/") else None
            continue
        if line.startswith("--- "):
            p = line[4:]
            oldpath = p[2:] if p.startswith("a/") else None
            continue
        m = HUNK.match(line)
        if not m:
            continue
        o_start, o_cnt = int(m.group(1)), int(m.group(2) if m.group(2) is not None else 1)
        n_start, n_cnt = int(m.group(3)), int(m.group(4) if m.group(4) is not None else 1)
        if path and path.endswith(".rs") and n_cnt > 0:
            if path not in new_cache:
                new_cache[path] = (git("show", f"{sha}:{path}") or "").splitlines()
            for ln in range(n_start, n_start + n_cnt):
                f = enclosing(new_cache[path], ln)
                if f: out.add(f)
        if has_parent and oldpath and oldpath.endswith(".rs") and o_cnt > 0:
            if oldpath not in old_cache:
                old_cache[oldpath] = (git("show", f"{parent}:{oldpath}") or "").splitlines()
            for ln in range(o_start, o_start + o_cnt):
                f = enclosing(old_cache[oldpath], ln)
                if f: out.add(f)
    return out

commits = (git("rev-list", "--reverse", os.environ["RANGE"]) or "").split()
head = commits[-1] if commits else "HEAD"
mcp_bases = {n[:-4] for n in tree_fns(head, mcp_dir) if n.endswith("_mcp")}
twins = sorted(mcp_bases & tree_fns(head, cli_dir))

hits = []
for sha in commits:
    t_mcp, t_cli = touched(sha, mcp_dir), touched(sha, cli_dir)
    if not t_mcp and not t_cli:
        continue
    for b in twins:
        m, c = (b + "_mcp") in t_mcp, b in t_cli
        if m != c:
            # Only an ESTABLISHED pair can drift: the changed twin must already
            # exist in the parent (a modification, not a creation), and the other
            # twin must exist at this commit. Otherwise this is the normal
            # "CLI first, MCP parity next commit" creation pattern.
            ch_name, ch_dir = (b + "_mcp", mcp_dir) if m else (b, cli_dir)
            ot_name, ot_dir = (b, cli_dir) if m else (b + "_mcp", mcp_dir)
            pat = lambda name: r"fn\s+" + name + r"\b"
            if subprocess.run(["git", "grep", "-qE", pat(ch_name), f"{sha}^", "--", ch_dir]).returncode != 0:
                continue
            if subprocess.run(["git", "grep", "-qE", pat(ot_name), sha, "--", ot_dir]).returncode != 0:
                continue
            subj = (git("log", "-1", "--format=%s", sha) or "").strip()
            hits.append({"commit": sha[:9], "twin": b, "changed": "mcp" if m else "cli",
                         "unchanged": "cli" if m else "mcp", "subject": subj[:100]})

scope = ("one-sided changes to _mcp/CLI twin helpers in the range; a REVIEW list, "
         "not a verdict: a hit means look at the other twin, not that it is wrong")
if os.environ["FORMAT"] == "json":
    print(json.dumps({"ok": not hits, "range": os.environ["RANGE"], "commits_scanned": len(commits),
                      "twin_pairs": len(twins), "one_sided": hits, "scope": scope}))
else:
    print(f"mcp/cli twin drift: {len(commits)} commit(s) in {os.environ['RANGE']}, {len(twins)} twin pair(s)")
    for h in hits:
        print(f"  ONE-SIDED  {h['commit']}  {h['twin']}: {h['changed']} changed, {h['unchanged']} not — {h['subject']}")
    if not hits:
        print("  no one-sided twin changes in range")
    print(f"  scope: {scope}")
sys.exit(1 if hits else 0)
PY
