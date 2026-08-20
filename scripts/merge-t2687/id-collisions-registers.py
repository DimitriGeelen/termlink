#!/usr/bin/env python3
"""Same-id / different-content collisions in the append-only registers.

The task-ID allocator collides across branches (T-2800). The LEARNINGS and
DECISIONS registers use the same max+1 scheme and have the same problem — and
nothing checks them, because T-2800 only walks .tasks/.

A naive union deduped on id silently keeps one and drops the other. That is
losing the thing the register exists to preserve.
"""
import os, subprocess, yaml

os.chdir("/root/.claude/jobs/d638a35c/tmp/merge-trial")


def load(stage, path):
    r = subprocess.run(["git", "show", "%s:%s" % (stage, path)],
                       capture_output=True, text=True)
    return yaml.safe_load(r.stdout) if r.returncode == 0 else {}


for path, key in ((".context/project/learnings.yaml", "learnings"),
                  (".context/project/decisions.yaml", "decisions")):
    ours, main = load(":2", path), load(":3", path)
    o = {r.get("id"): r for r in (ours.get(key) or []) if isinstance(r, dict)}
    m = {r.get("id"): r for r in (main.get(key) or []) if isinstance(r, dict)}
    shared = set(o) & set(m)
    clash = [i for i in shared if o[i] != m[i]]
    only_ours = sorted(set(o) - set(m))
    print("%s" % path)
    print("   main=%d  ours=%d  shared-id=%d  SAME-ID-DIFFERENT-CONTENT=%d"
          % (len(m), len(o), len(shared), len(clash)))
    print("   ids only in ours: %s" % (only_ours[:8] or "none"))
    for i in sorted(clash)[:6]:
        ot = str(o[i].get(key[:-1]) or o[i])[:70]
        mt = str(m[i].get(key[:-1]) or m[i])[:70]
        print("     %s" % i)
        print("       ours: %s..." % ot)
        print("       main: %s..." % mt)
    print()
