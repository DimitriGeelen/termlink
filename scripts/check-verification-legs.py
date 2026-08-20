#!/usr/bin/env python3
"""How many verification legs would the P-011 gate actually run, per task?

Reported by 832-Workflow-designer (framework:pickup offset 30): the gate extracts
the Verification block with a COLUMN-0 sed anchor —

    sed -n '/^## Verification/,/^## /p'   ... then   [ -z "$verify_cmds" ] && return 0

so a `## Verification` heading that is present but NOT at column 0 yields an empty
extraction and the gate returns 0 SILENTLY. Completion output is byte-identical to
a task whose legs all passed.

This mimics the same extraction and prints the leg count, which is what the filing
argues should be unconditional. Classes:

  OK        heading at column 0, N >= 1 legs      -> the gate ran N commands
  NO-BLOCK  no `## Verification` anywhere         -> documented pass-through
  MALFORMED heading text present but NOT anchored -> the silent-zero case
  EMPTY     anchored but no runnable lines        -> passes having run nothing
"""
import glob, os, re, sys

SHOW_ALL = "--all" in sys.argv


def legs(path):
    txt = open(path, errors="replace").read()
    lines = txt.split("\n")
    anchored = [i for i, l in enumerate(lines) if l.startswith("## Verification")]
    mentioned = "## Verification" in txt

    if not anchored:
        return ("MALFORMED" if mentioned else "NO-BLOCK"), 0

    start = anchored[0] + 1
    end = len(lines)
    for i in range(start, len(lines)):
        if lines[i].startswith("## "):
            end = i
            break
    body = lines[start:end]
    cmds = [l for l in body
            if l.strip() and not l.strip().startswith("#") and not l.strip().startswith("```")]
    return ("OK" if cmds else "EMPTY"), len(cmds)


rows = []
for f in sorted(glob.glob(".tasks/completed/*.md")):
    head = open(f, errors="replace").read().split("\n---\n")[0]
    m = re.search(r"^date_finished:[ \t]*(\S+)", head, re.M)
    fin = m.group(1).strip().strip("'\"") if m else ""
    if not SHOW_ALL and not fin.startswith("2026-08-20") and not fin.startswith("2026-08-21"):
        continue
    tid = (re.search(r"^id:[ \t]*(\S+)", head, re.M) or [None, "?"])[1]
    cls, n = legs(f)
    rows.append((tid, cls, n, os.path.basename(f)[:44]))

bad = [r for r in rows if r[1] in ("MALFORMED", "EMPTY")]
print("tasks closed today: %d" % len(rows))
print("  ran >=1 leg : %d" % len([r for r in rows if r[1] == "OK"]))
print("  NO-BLOCK    : %d  (documented pass-through)" % len([r for r in rows if r[1] == "NO-BLOCK"]))
print("  MALFORMED   : %d  <-- silent-zero: heading present, not anchored" % len([r for r in rows if r[1] == "MALFORMED"]))
print("  EMPTY       : %d  <-- anchored but nothing runnable" % len([r for r in rows if r[1] == "EMPTY"]))
print()
for tid, cls, n, name in rows:
    flag = "  <<<" if cls in ("MALFORMED", "EMPTY") else ""
    print("  %-9s %-10s legs=%-3d %s%s" % (tid, cls, n, name, flag))
