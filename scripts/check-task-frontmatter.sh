#!/usr/bin/env bash
# guard-layer: source
#
# check-task-frontmatter.sh (T-2794)
#
# Every task file's YAML frontmatter must parse as a mapping.
#
# Why this exists. T-2222 records that the BVP estimator "corrupts anchor-less task
# frontmatter (orphaned proposed-score lists produce invalid YAML)" — a bulk writer that
# appends `bvp_scores_proposed:` / `cost_estimate_proposed:` to a file lacking the
# commented anchor block can leave frontmatter that no longer loads. T-2794 then ran both
# estimators across 2557 task files in one go. Nothing in the framework validated the
# result: `fw audit` parses the 15 project YAML files under `.context/project/`, and the
# task corpus — two orders of magnitude larger, and the thing bulk writers actually touch
# — was unchecked.
#
# The failure is quiet, which is what makes it worth a guard. A task whose frontmatter
# stops parsing does not error; it silently drops out of every consumer that walks the
# corpus — `fw bvp`, the audit's duplicate-ID scan, the handover's active-task list — so
# the symptom is a task that has simply gone missing from listings nobody re-counts.
#
# Scope: parse validity only. It does not check that fields are correct, that `status` is
# a legal value, or that required fields are present — a clean result means every task
# still LOADS, not that any of them is right.
#
# Exit codes: 0 = all parse, 1 = at least one does not, 2 = tooling error.
set -u

TASKS_DIRS=()
FORMAT=human
QUIET=0

die() {
    if [ "$FORMAT" = json ]; then printf '{"ok":false,"error":"%s"}\n' "$1"
    else echo "check-task-frontmatter: $1" >&2; fi
    exit 2
}

while [ $# -gt 0 ]; do
    case "$1" in
        --tasks-dir) TASKS_DIRS+=("${2:-}"); shift 2 ;;
        --json)      FORMAT=json; shift ;;
        --quiet)     QUIET=1; shift ;;
        -h|--help)   sed -n '3,28p' "$0"; exit 0 ;;
        *)           echo "check-task-frontmatter: unknown arg: $1" >&2; exit 2 ;;
    esac
done

[ ${#TASKS_DIRS[@]} -gt 0 ] || TASKS_DIRS=(".tasks")
command -v python3 >/dev/null 2>&1 || die "python3 not in PATH"

export CTF_DIRS="${TASKS_DIRS[*]}"
export CTF_FORMAT="$FORMAT"
export CTF_QUIET="$QUIET"

python3 - <<'PY'
import glob, json, os, sys

try:
    import yaml
except ImportError:
    print('{"ok":false,"error":"pyyaml not available"}' if os.environ.get("CTF_FORMAT") == "json"
          else "check-task-frontmatter: pyyaml not available", file=sys.stderr)
    sys.exit(2)

dirs = os.environ["CTF_DIRS"].split()
fmt = os.environ.get("CTF_FORMAT", "human")
quiet = os.environ.get("CTF_QUIET") == "1"

bad, ok = [], 0
files = []
for d in dirs:
    if not os.path.isdir(d):
        print(f"check-task-frontmatter: no such tasks dir: {d}", file=sys.stderr)
        sys.exit(2)
    files.extend(glob.glob(os.path.join(d, "**", "*.md"), recursive=True))

if not files:
    # An empty corpus is a tooling error, never a clean bill: "0 of 0 parse" is
    # vacuously true and would report green over a glob that stopped matching.
    print('{"ok":false,"error":"no task files found"}' if fmt == "json"
          else "check-task-frontmatter: no task files found — refusing to report clean",
          file=sys.stderr)
    sys.exit(2)

for p in sorted(files):
    try:
        txt = open(p, encoding="utf-8").read()
    except Exception as e:
        bad.append({"file": p, "why": f"unreadable: {e}"}); continue
    if not txt.startswith("---"):
        bad.append({"file": p, "why": "no frontmatter block"}); continue
    end = txt.find("\n---", 3)
    if end == -1:
        bad.append({"file": p, "why": "frontmatter block never closes"}); continue
    try:
        d = yaml.safe_load(txt[3:end])
    except Exception as e:
        bad.append({"file": p, "why": str(e).split("\n")[0][:200]}); continue
    if not isinstance(d, dict):
        bad.append({"file": p, "why": f"frontmatter is {type(d).__name__}, not a mapping"})
    else:
        ok += 1

scope = ("parse validity only — a clean result means every task still LOADS, "
         "not that its fields are correct or complete")

if fmt == "json":
    print(json.dumps({"ok": not bad, "checked": len(files), "parsed": ok,
                      "invalid": bad, "invalid_count": len(bad), "scope": scope}))
elif bad:
    print(f"check-task-frontmatter: FIRING — {len(bad)} of {len(files)} task file(s) do not parse")
    for b in bad[:25]:
        print(f"  {b['file']}\n      {b['why']}")
    if len(bad) > 25:
        print(f"  ... and {len(bad) - 25} more")
    print(f"\n  SCOPE: {scope}")
elif not quiet:
    print(f"check-task-frontmatter: clean — {ok}/{len(files)} task file(s) parse")
    print(f"  SCOPE: {scope}")

sys.exit(1 if bad else 0)
PY
