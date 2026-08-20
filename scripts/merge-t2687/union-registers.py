#!/usr/bin/env python3
"""Union the append-only registers WITHOUT losing a record to an id collision.

Both branches allocate register ids by max+1 against their own copy, so the same
id can name two different records — exactly the task-id defect (T-2800), in a
register nothing checks. Measured here: PL-328 means two different learnings, and
ten PD-* ids name two different decisions each.

Deduping on id would silently keep one and drop the other. That is losing the
thing the register exists to preserve, so:

  * a record present on both sides with IDENTICAL content  -> kept once
  * a record only on one side                              -> kept
  * SAME id, DIFFERENT content                             -> BOTH kept; ours is
    renumbered above the global max, and stamped renumbered_from so the old id
    stays traceable

main-first ordering is preserved so existing references stay roughly stable.
"""
import os, re, subprocess, sys, yaml

os.chdir("/root/.claude/jobs/d638a35c/tmp/merge-trial")
APPLY = "--apply" in sys.argv

SPECS = [
    (".context/project/learnings.yaml", "learnings", "PL-"),
    (".context/project/decisions.yaml", "decisions", "PD-"),
]


def raw(stage, path):
    r = subprocess.run(["git", "show", "%s:%s" % (stage, path)],
                       capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else ""


def num(i, prefix):
    m = re.match(re.escape(prefix) + r"(\d+)$", str(i or ""))
    return int(m.group(1)) if m else -1


for path, key, prefix in SPECS:
    ours = yaml.safe_load(raw(":2", path)) or {}
    main = yaml.safe_load(raw(":3", path)) or {}
    o_recs = [r for r in (ours.get(key) or []) if isinstance(r, dict)]
    m_recs = [r for r in (main.get(key) or []) if isinstance(r, dict)]
    m_by_id = {r.get("id"): r for r in m_recs}

    ceiling = max([num(r.get("id"), prefix) for r in m_recs + o_recs] + [0])
    merged = list(m_recs)
    renumbered, added, skipped = [], 0, 0

    for r in o_recs:
        rid = r.get("id")
        if rid not in m_by_id:
            merged.append(r)
            added += 1
        elif m_by_id[rid] == r:
            skipped += 1                       # identical, already present
        else:
            ceiling += 1
            new_id = "%s%03d" % (prefix, ceiling)
            r = dict(r)
            r["renumbered_from"] = rid
            r["id"] = new_id
            merged.append(r)
            renumbered.append((rid, new_id))
            added += 1

    print("%s" % path)
    print("   main=%d  ours=%d  ->  merged=%d   (new from ours: %d, identical skipped: %d)"
          % (len(m_recs), len(o_recs), len(merged), added, skipped))
    for old, new in renumbered:
        print("     renumbered %s -> %s (same id, different content)" % (old, new))

    if APPLY:
        out = dict(main)
        out[key] = merged
        with open(path, "w") as f:
            yaml.safe_dump(out, f, sort_keys=False, default_flow_style=False,
                           allow_unicode=True, width=100)
        subprocess.run(["git", "add", "--", path], capture_output=True)
    print()

# metrics-history is a pure time series with no id field; main carries the far
# longer run (6104 lines vs 304). Take it rather than invent a merge.
if APPLY:
    open(".context/project/metrics-history.yaml", "w").write(
        raw(":3", ".context/project/metrics-history.yaml"))
    subprocess.run(["git", "add", "--", ".context/project/metrics-history.yaml"],
                   capture_output=True)
print(".context/project/metrics-history.yaml  take-main (time series, main has the longer run)")
print("---")
print("applied" if APPLY else "(dry run — pass --apply)")
