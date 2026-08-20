#!/usr/bin/env python3
"""Union CLAUDE.md: main has 7 sections ours lacks, ours has 8 main lacks.

Take-ours drops seven of main's sections; take-main drops eight of ours. So the
resolution is: start from OURS (it carries this branch's work in place, with
surrounding edits) and splice in each `###` section that exists only on main,
inserted before the first `## ` heading that follows its neighbours on main —
in practice all seven are guard-layer sections that belong in the same run of
`###` blocks, so they are appended after the last such block in ours.

Deliberately conservative: nothing is deleted, nothing is reordered, and the
result is checked for both heading sets afterwards.
"""
import os, re, subprocess, sys

os.chdir("/root/.claude/jobs/d638a35c/tmp/merge-trial")
APPLY = "--apply" in sys.argv


def raw(stage):
    r = subprocess.run(["git", "show", "%s:CLAUDE.md" % stage],
                       capture_output=True, text=True)
    return r.stdout


ours, main = raw(":2"), raw(":3")


def sections(txt):
    """Split into (heading, body) for '### ' blocks, keeping everything else."""
    out, cur, buf = [], None, []
    for line in txt.split("\n"):
        if line.startswith("### ") or line.startswith("## "):
            if cur is not None:
                out.append((cur, "\n".join(buf)))
            cur, buf = line, [line]
        else:
            buf.append(line)
    if cur is not None:
        out.append((cur, "\n".join(buf)))
    return out


o_secs, m_secs = sections(ours), sections(main)
o_heads = {h.strip() for h, _ in o_secs}
missing = [(h, b) for h, b in m_secs if h.strip() not in o_heads and h.startswith("### ")]

print("sections only on main (to splice in): %d" % len(missing))
for h, _ in missing:
    print("   %s" % h[:78])

# Insert after the last '### ' block in ours that precedes '## Project-Specific
# Rules' — that is where this file keeps its guard/check sections.
anchor = "## Project-Specific Rules"
idx = ours.find(anchor)
if idx < 0:
    print("!! anchor '%s' not found — refusing to guess placement" % anchor)
    sys.exit(2)

addition = "\n".join(b.rstrip() + "\n" for _, b in missing)
merged = ours[:idx] + addition + "\n" + ours[idx:]

new_heads = {l.strip() for l in merged.split("\n") if l.startswith(("## ", "### "))}
lost_from_main = [h.strip() for h, _ in m_secs if h.strip() not in new_heads]
lost_from_ours = [h.strip() for h, _ in o_secs if h.strip() not in new_heads]
print()
print("after union: headings=%d  lost-from-main=%d  lost-from-ours=%d"
      % (len(new_heads), len(lost_from_main), len(lost_from_ours)))
for h in lost_from_main[:5]:
    print("   STILL MISSING (main): %s" % h[:70])

if APPLY and not lost_from_main and not lost_from_ours:
    open("CLAUDE.md", "w").write(merged)
    subprocess.run(["git", "add", "--", "CLAUDE.md"], capture_output=True)
    print("applied")
elif APPLY:
    print("NOT applied — union would still lose headings")
else:
    print("(dry run — pass --apply)")
