#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
#
# check-go-propagation.sh — GO-recorded inceptions that never propagated their scope.
#
# Origin: T-3003 (investigation) -> T-3038 (this check). The decide path in vendored
# lib/inception.sh:775 prints "Next: Create build tasks for implementation" and writes
# NOTHING — no related_tasks on either side. Linkage therefore depends on an agent
# hand-editing frontmatter after GO, an undocumented step gated by nothing. T-3003
# measured 80 of 167 GO-recorded inceptions strictly unlinked, and the leak is ongoing.
#
# This is a DEPLOY-TIME / ad-hoc check, NOT a cron canary (T-2800 tier, sibling of
# check-cron-install-drift.sh). --no-heartbeat is accepted for guard-layer symmetry and
# is a no-op: nothing here heartbeats, because nothing schedules it.
#
# SCOPE — read a green narrowly (T-2680). It answers exactly one question: does every
# GO-recorded inception past its grace period carry a forward or backward machine-readable
# link? It does NOT audit whether the follow-on work is adequate, whether it was ever
# finished, or whether the GO was the right call.
#
# Exit: 0 = no unacknowledged leak · 1 = a leak · 2 = tooling (fail-closed).
set -uo pipefail

TASKS_DIR="${GO_PROP_TASKS_DIR:-.tasks}"
ALLOWLIST="${GO_PROP_ALLOWLIST:-.context/checks/go-propagation-allowlist}"
GRACE_DAYS="${GO_PROP_GRACE_DAYS:-7}"
JSON=0; QUIET=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --quiet) QUIET=1 ;;
    --no-heartbeat) : ;;   # accepted, no-op — see header
    --tasks-dir) TASKS_DIR="${2:-}"; shift ;;
    --allowlist) ALLOWLIST="${2:-}"; shift ;;
    --grace-days) GRACE_DAYS="${2:-}"; shift ;;
    -h|--help) sed -n '3,20p' "$0"; exit 0 ;;
    *) echo "check-go-propagation: unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

command -v python3 >/dev/null 2>&1 || {
  echo "check-go-propagation: python3 not found — cannot scan (fail-closed)" >&2; exit 2; }
[ -d "$TASKS_DIR" ] || {
  echo "check-go-propagation: tasks dir not found: $TASKS_DIR (fail-closed)" >&2; exit 2; }

GO_PROP_TASKS_DIR="$TASKS_DIR" GO_PROP_ALLOWLIST="$ALLOWLIST" \
GO_PROP_GRACE_DAYS="$GRACE_DAYS" GO_PROP_JSON="$JSON" GO_PROP_QUIET="$QUIET" \
python3 - <<'PYEOF'
import os, re, sys, json, glob, datetime
from collections import defaultdict

tasks_dir = os.environ["GO_PROP_TASKS_DIR"]
allow_path = os.environ["GO_PROP_ALLOWLIST"]
as_json = os.environ["GO_PROP_JSON"] == "1"
quiet = os.environ["GO_PROP_QUIET"] == "1"
try:
    grace_days = int(os.environ["GO_PROP_GRACE_DAYS"])
    if grace_days < 0: raise ValueError
except ValueError:
    print("check-go-propagation: --grace-days must be a non-negative integer (fail-closed)", file=sys.stderr)
    sys.exit(2)

files = sorted(glob.glob(os.path.join(tasks_dir, "active", "*.md")) +
               glob.glob(os.path.join(tasks_dir, "completed", "*.md")))
if not files:
    print("check-go-propagation: no task files under %s (fail-closed — a corpus of zero "
          "is never a clean census)" % tasks_dir, file=sys.stderr)
    sys.exit(2)

# ---- allowlist: counted and reported, never silently excluded (T-2483) ----
acknowledged = {}
if os.path.exists(allow_path):
    try:
        for raw in open(allow_path, encoding="utf-8"):
            line = raw.strip()
            if not line or line.startswith("#"): continue
            tid, _, reason = line.partition("#")
            acknowledged[tid.strip()] = reason.strip() or "(no reason cited)"
    except OSError as e:
        print("check-go-propagation: allowlist unreadable: %s (fail-closed)" % e, file=sys.stderr)
        sys.exit(2)

FM = re.compile(r"\A---\n(.*?)\n---\n", re.S)
ID_TOKEN = re.compile(r"\bT-\d+\b")

def field(fm, name):
    m = re.search(r"^%s:[ \t]*(.*)$" % re.escape(name), fm, re.M)
    if not m: return ""
    # Frontmatter timestamps are sometimes emitted quoted ('2026-09-08T21:28:26Z') and
    # sometimes bare. An unstripped quote makes fromisoformat fail, the age read as
    # unknown, and the record reach the firing branch by falling through rather than by
    # being old — a today-decided task would fire. Strip before anything parses it.
    return m.group(1).strip().strip("'\"")

def verdict(body):
    """GO / NO-GO / DEFER / None. NO-GO must never read as GO."""
    for pat in (r"^\*\*Recommendation:\*\*[ \t]*(.+)$",
                r"^[-*][ \t]*\*\*Decision:?\*\*:?[ \t]*(.+)$",
                r"^\*\*Decision\*\*:[ \t]*(.+)$",
                r"Inception decision:[ \t]*(\S+)"):
        for m in re.finditer(pat, body, re.M):
            v = m.group(1).strip().upper()
            if v.startswith("NO-GO"): return "NO-GO"
            if v.startswith("DEFER"): return "DEFER"
            if v.startswith("GO"): return "GO"
    return None

records, backrefs, mentions = [], set(), defaultdict(set)
for path in files:
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as e:
        print("check-go-propagation: unreadable task file %s: %s (fail-closed)" % (path, e), file=sys.stderr)
        sys.exit(2)
    m = FM.match(text)
    fm, body = (m.group(1), text[m.end():]) if m else ("", text)
    tid = field(fm, "id")
    if not tid: continue

    rel = field(fm, "related_tasks")
    rel_ids = ID_TOKEN.findall(rel)
    backrefs.update(rel_ids)
    for other in set(ID_TOKEN.findall(text)) - {tid}:
        mentions[other].add(os.path.basename(path))

    if field(fm, "workflow_type") != "inception": continue
    v = verdict(body)
    if v != "GO": continue
    records.append({
        "id": tid, "file": os.path.basename(path),
        "decided": field(fm, "date_finished") or field(fm, "last_update"),
        "forward": rel_ids,
    })

now = datetime.datetime.now(datetime.timezone.utc)
def age_days(stamp):
    if not stamp: return None
    try:
        return (now - datetime.datetime.fromisoformat(stamp.replace("Z", "+00:00"))).days
    except ValueError:
        return None

firing, in_grace, ack, linked, orphans = [], [], [], [], []
for r in records:
    if r["forward"] or r["id"] in backrefs:
        linked.append(r); continue
    # strictly unlinked from here down
    r["orphan"] = not mentions[r["id"]]          # loose predicate: nobody names it at all
    r["age_days"] = age_days(r["decided"])
    if r["id"] in acknowledged:
        r["reason"] = acknowledged[r["id"]]; ack.append(r)
    elif r["age_days"] is not None and r["age_days"] < grace_days:
        in_grace.append(r)
    else:
        firing.append(r)
    if r["orphan"]: orphans.append(r)

ok = not firing
census = {"go_inceptions": len(records), "linked": len(linked), "firing": len(firing),
          "in_grace": len(in_grace), "acknowledged": len(ack), "orphans": len(orphans),
          "grace_days": grace_days, "allowlist": allow_path}
SCOPE = ("detects GO-recorded inceptions with no machine-readable link in either direction; "
         "does NOT audit whether the follow-on work is adequate, finished, or correct")

if as_json:
    print(json.dumps({"ok": ok, "census": census, "scope": SCOPE, "firing": firing,
                      "in_grace": in_grace, "acknowledged": ack, "orphans": orphans}, indent=2))
    sys.exit(0 if ok else 1)

if not (quiet and ok):
    print("GO-propagation: %d GO inception(s) — %d linked, %d firing, %d in grace (<%dd), %d acknowledged"
          % (census["go_inceptions"], census["linked"], census["firing"],
             census["in_grace"], grace_days, census["acknowledged"]))
    print("Scope: %s" % SCOPE)
for r in firing:
    tag = "  [ORPHAN — no task mentions it at all]" if r["orphan"] else ""
    age = "%dd ago" % r["age_days"] if r["age_days"] is not None else "age unknown — fires by default"
    print("  FIRING  %s  decided %s (%s)  %s%s"
          % (r["id"], r["decided"] or "unknown", age, r["file"], tag))
if firing and not quiet:
    print("\nRemediation: file the slices the GO approved, then write both directions —")
    print("  <inception>.related_tasks: [T-AAAA, ...]   and each slice's related_tasks: [<inception>]")
    print("Or, if the approval genuinely lapsed, acknowledge it in %s with a cited reason." % allow_path)
sys.exit(0 if ok else 1)
PYEOF
