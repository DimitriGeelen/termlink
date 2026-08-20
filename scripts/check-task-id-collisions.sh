#!/usr/bin/env bash
# T-2800 — cross-branch task-ID collision + duplicate-work check.
#
# Every worktree allocates task IDs by scanning its OWN .tasks/ for the highest ID
# and incrementing. Worktrees materialise only their own branch's files, so no branch
# can see another's allocations. The framework creates those worktrees itself, so
# this is a framework-made race, not an operator oversight.
#
# It was understood in March 2026. T-229 renumbered six colliding tasks by hand and
# recorded the cause verbatim — "task counter not safe for concurrent work" — routing
# the fix upstream as G-007. Nothing landed. By August the collision count was twelve,
# and three agents had independently implemented fixes for the same two defects.
#
# The duplicate-ID check T-229 left behind reads a single working tree:
#     cat .tasks/active/*.md .tasks/completed/*.md | grep '^id:' | sort | uniq -d
# It is structurally incapable of seeing the cross-branch case, and has been passing
# cleanly throughout.
#
# TWO AXES, because each is blind exactly where the other fires:
#
#   Axis A — COLLIDING IDS. An ID claimed by two or more branches. FIRES (exit 1).
#            Differing filenames are printed, so "same task on two branches"
#            (a cherry-pick — harmless, and NOT reported) is distinguishable from
#            "two different tasks, one ID" (the defect).
#
#   Axis B — NEAR-DUPLICATE TITLES. Task titles across DIFFERENT branches whose
#            normalised word sets overlap above a threshold. WARNS, never fires.
#            This is the axis that matters: renumbering twelve tasks is mechanical,
#            three agents solving the same problem is not recoverable. But title
#            similarity is a judgement, and a heuristic that blocks on a judgement
#            gets disabled the first time it is wrong.
#
# This is a DEPLOY-TIME / ad-hoc check, NOT a cron canary — same tier as
# check-cron-install-drift.sh (T-2561). Run it before starting work, and before a
# merge. It does not fix allocation; that is vendored framework code and is filed
# upstream per G-062 (framework:pickup offsets 15-16).
#
# Exit codes: 0 no collisions · 1 colliding ID(s) · 2 tooling error
set -u

BASE_REF="${TASK_COLLISION_BASE:-main}"
BRANCH_GLOB="${TASK_COLLISION_BRANCHES:-}"
THRESHOLD="${TASK_COLLISION_SIMILARITY:-0.72}"
QUIET=0
FORMAT=human
NO_TITLES=0

usage() {
    sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: check-task-id-collisions.sh [OPTIONS]
  --base REF        Common base to diff branches against (default: main)
  --threshold N     Axis B similarity threshold, 0..1 (default: 0.72)
  --no-titles       Skip axis B entirely
  --json            Emit a JSON envelope
  --quiet           Print only when axis A fires
  -h, --help        This help

Test hooks: TASK_COLLISION_BASE, TASK_COLLISION_BRANCHES (space-separated branch
list, overrides discovery), TASK_COLLISION_SIMILARITY.

Exit: 0 no collisions · 1 colliding ID(s) · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --base)      shift; [ $# -ge 1 ] || { echo "check-task-id-collisions: --base requires a value" >&2; exit 2; }; BASE_REF="$1" ;;
        --threshold) shift; [ $# -ge 1 ] || { echo "check-task-id-collisions: --threshold requires a value" >&2; exit 2; }; THRESHOLD="$1" ;;
        --no-titles) NO_TITLES=1 ;;
        --json)      FORMAT=json ;;
        --quiet)     QUIET=1 ;;
        -h|--help)   usage; exit 0 ;;
        *) echo "check-task-id-collisions: unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

git rev-parse --git-dir >/dev/null 2>&1 || {
    echo "check-task-id-collisions: not a git repository" >&2; exit 2; }

git rev-parse --verify --quiet "$BASE_REF" >/dev/null || {
    echo "check-task-id-collisions: base ref not found: $BASE_REF" >&2; exit 2; }

export TASK_COLLISION_BASE="$BASE_REF"
export TASK_COLLISION_BRANCHES="$BRANCH_GLOB"
export TASK_COLLISION_SIMILARITY="$THRESHOLD"
export TASK_COLLISION_FORMAT="$FORMAT"
export TASK_COLLISION_QUIET="$QUIET"
export TASK_COLLISION_NO_TITLES="$NO_TITLES"

python3 - <<'PYEOF'
import json, os, re, subprocess, sys

BASE      = os.environ["TASK_COLLISION_BASE"]
BRANCHES  = os.environ.get("TASK_COLLISION_BRANCHES", "").split()
THRESHOLD = float(os.environ.get("TASK_COLLISION_SIMILARITY") or 0.72)
FORMAT    = os.environ.get("TASK_COLLISION_FORMAT", "human")
QUIET     = os.environ.get("TASK_COLLISION_QUIET") == "1"
NO_TITLES = os.environ.get("TASK_COLLISION_NO_TITLES") == "1"


def git(*args):
    return subprocess.run(("git",) + args, capture_output=True, text=True).stdout


def task_files(ref):
    out = git("ls-tree", "-r", "--name-only", ref, ".tasks/")
    return [p for p in out.splitlines() if p.endswith(".md")]


ID_RE = re.compile(r"(?:^|/)(T-\d+)-")


def id_of(path):
    m = ID_RE.search(path)
    return m.group(1) if m else None


if not BRANCHES:
    BRANCHES = [b for b in git("branch", "--format=%(refname:short)").split()
                if b != BASE and not b.endswith("-backup")]

base_ids = {}
for p in task_files(BASE):
    i = id_of(p)
    if i:
        base_ids.setdefault(i, p)

# id -> {branch: filename}   (only IDs NOT already in the base)
claims = {}
per_branch_new = {}
for b in BRANCHES:
    files = task_files(b)
    if not files:
        continue
    new_here = []
    for p in files:
        i = id_of(p)
        if not i or i in base_ids:
            continue
        claims.setdefault(i, {})[b] = os.path.basename(p)
        new_here.append(i)
    per_branch_new[b] = sorted(set(new_here))

# ---- Axis A: an ID claimed by 2+ branches for DIFFERENT files -------------
collisions = []
for i, byb in sorted(claims.items()):
    if len(byb) < 2:
        continue
    if len(set(byb.values())) < 2:
        # Same filename on every branch: shared history / cherry-pick, not a clash.
        continue
    collisions.append({"id": i, "claims": [{"branch": b, "file": f}
                                          for b, f in sorted(byb.items())]})

# ---- Axis B: near-duplicate titles across DIFFERENT branches --------------
# Scored on shared RARE words, not on set overlap. Two reasons.
#
# First, the obvious source — the filename slug — is truncated at ~40 chars, which
# is exactly where the discriminating words live. "blanket-contextworking-gitignore-
# makes-s" and "static-check-allowlists-live-under-a-git" describe the same defect
# and share not one token. So titles are read from the `name:` frontmatter instead.
#
# Second, plain Jaccard is the wrong measure even on full titles. The real pair
# found in August scores ~0.19: "Blanket .context/working gitignore makes static-
# check allowlists untrackable" vs "static-check allowlists live under a gitignored
# path". Any threshold low enough to catch it drowns in noise. What actually
# identifies the pair is that they share ALLOWLISTS and STATIC — terms that are rare
# across the corpus. Common words (canary, check, fix) carry no signal; rare ones
# nearly always mean "same subject".
STOP = set("""a an the and or of to for in on at by with from is are be was were
that this it its as into via not no non pre post re run runs ran use used uses
using make makes made fix fixes fixed add adds added new old set sets than then
when where which while who whom whose what why how all any both each few more most
other some such only own same so too very can will just should now does did done
task tasks work works working""".split())

RARE_MAX_DF = int(os.environ.get("TASK_COLLISION_RARE_DF") or 4)
MIN_SHARED_RARE = int(os.environ.get("TASK_COLLISION_MIN_SHARED") or 2)


def words(text):
    toks = re.split(r"[^a-z0-9]+", text.lower())
    return {t for t in toks if t and t not in STOP and not t.isdigit() and len(t) > 2}


def titles_for(ref):
    """path -> title, read from each task's `name:` frontmatter."""
    out = git("grep", "-n", "--no-color", "^name:", ref, "--", ".tasks/")
    res = {}
    for line in out.splitlines():
        parts = line.split(":", 3)
        if len(parts) < 4:
            continue
        _, path, _, content = parts
        title = content[len("name:"):].strip().strip('"').strip("'")
        if title:
            res[path] = title
    return res


entries = []
if not NO_TITLES:
    for b, ids in per_branch_new.items():
        tmap = titles_for(b)
        idset = set(ids)
        for path, title in tmap.items():
            i = id_of(path)
            if i in idset:
                entries.append((b, i, os.path.basename(path), title, words(title)))

    # Document frequency over the titles actually in play.
    df = {}
    for _, _, _, _, w in entries:
        for t in w:
            df[t] = df.get(t, 0) + 1

colliding_ids = {c["id"] for c in collisions}
dupes = []
if not NO_TITLES:
    for x in range(len(entries)):
        bx, ix, fx, tx, wx = entries[x]
        if not wx:
            continue
        for y in range(x + 1, len(entries)):
            by, iy, fy, ty, wy = entries[y]
            if bx == by or not wy:
                continue
            if ix == iy and ix in colliding_ids:
                continue  # axis A already owns this pair
            shared = wx & wy
            if not shared:
                continue
            rare = sorted(t for t in shared if df.get(t, 0) <= RARE_MAX_DF)
            if len(rare) < MIN_SHARED_RARE:
                continue
            sim = len(shared) / float(len(wx | wy))
            dupes.append({"shared_rare": rare,
                          "rare_count": len(rare),
                          "similarity": round(sim, 3),
                          "a": {"branch": bx, "id": ix, "file": fx, "title": tx},
                          "b": {"branch": by, "id": iy, "file": fy, "title": ty}})
    dupes.sort(key=lambda d: (-d["rare_count"], -d["similarity"]))

fire = len(collisions) > 0

if FORMAT == "json":
    print(json.dumps({
        "ok": not fire,
        "base": BASE,
        "branches_scanned": sorted(per_branch_new.keys()),
        "collision_count": len(collisions),
        "collisions": collisions,
        "duplicate_title_count": len(dupes),
        "duplicate_titles": dupes,
        "similarity_threshold": THRESHOLD,
    }))
    sys.exit(1 if fire else 0)

if QUIET and not fire:
    sys.exit(0)

if collisions:
    print("check-task-id-collisions: %d task ID(s) claimed by more than one branch "
          "— FIRING:" % len(collisions))
    for c in collisions:
        holders = " ".join(x["branch"] for x in c["claims"])
        print("  COLLISION: %s  claimed by: %s" % (c["id"], holders))
        for x in c["claims"]:
            print("      %-42s %s" % (x["branch"], x["file"]))
    print("  These are DIFFERENT tasks sharing an identifier. Merging any two of these")
    print("  branches produces duplicate IDs, by which point the ID is already embedded")
    print("  in commit messages, episodic memory, related_tasks: fields and handovers.")
    print("  Remediation: renumber before merging (T-229 is the worked example), and")
    print("  pick the new IDs above the highest claimed on ANY branch — `fw task create`")
    print("  has no --id flag, so the rename is manual.")

if dupes:
    print("")
    print("check-task-id-collisions: %d near-duplicate task title(s) across branches "
          "— WARNING (not firing):" % len(dupes))
    for d in dupes:
        print("  shared: %s" % ", ".join(d["shared_rare"]))
        print("     %-38s %s" % (d["a"]["branch"], d["a"]["id"]))
        print("        %s" % d["a"]["title"][:110])
        print("     %-38s %s" % (d["b"]["branch"], d["b"]["id"]))
        print("        %s" % d["b"]["title"][:110])
        print("")
    print("  Two branches may be solving the same problem. Renumbering is mechanical;")
    print("  duplicated work is not recoverable. Read the other branch before continuing.")

if not fire and not dupes:
    # Do not claim the title axis is clean when it was never run — that is the
    # same "healthy while nobody looked" wording this session spent its time on.
    titles_note = "axis B skipped (--no-titles)" if NO_TITLES else "no near-duplicate titles"
    print("check-task-id-collisions: clean (%d branch(es) scanned against %s, "
          "no colliding IDs, %s)" % (len(per_branch_new), BASE, titles_note))
elif not fire:
    print("")
    print("check-task-id-collisions: no colliding IDs (%d branch(es) scanned against %s)"
          % (len(per_branch_new), BASE))

sys.exit(1 if fire else 0)
PYEOF
