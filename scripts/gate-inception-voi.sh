#!/usr/bin/env bash
# guard-layer: source --scan
#
# T-3174 — gate an unset voi_score on NEW inceptions, with a cited-reason override.
#
# WHY. An inception's `voi_score` (0..1) answers "how valuable is it to resolve this
# question at all", and it ranks inceptions against each other and against ordinary
# work. `.tasks/templates/inception.md` ships `voi_score: 0.5` pre-filled, and
# estimator.py::_score_inception_voi maps BOTH the 0.5 default and an ABSENT value
# to the same internal 2:
#
#     voi = fm.get("voi_score")
#     if voi is None:  return 2, ["->2 (voi-absent-grandfathered)"]
#     score = int(round(voi_f * 5))        # round(0.5*5) == 2, banker's rounding
#
# So "I judged this exactly middling" and "nobody ever looked at this" are the same
# number. Measured here: of 245 inception tasks, 103 carry the default, 141 carry
# nothing, and ONE carries a considered value. The field has discriminated once in
# the project's history, and all 244 land mid-rank AHEAD of work that was actually
# measured and scored low.
#
# WHY A GATE AND NOT A WARNING. 832-Workflow-designer tried a warning printed
# directly above the field and reported the figure did not move by one. Their
# phrase, and it is the whole argument: a comment is not a gate.
#
# WHY AN OVERRIDE. A gate with no escape hatch is friction that gets disabled, and a
# disabled gate protects nothing. The operator's instruction was explicit: the gate
# must have a way to proceed. So `voi_score_waived: "<reason>"` clears it — but an
# EMPTY reason does not, because a waiver without a reason is not a waiver, it is the
# unset field with extra steps. Every waiver is logged, because a SILENT override is
# just a disabled gate wearing a costume.
#
# FAIL-OPEN, DELIBERATELY. This protects a ranking number, not a safety property.
# An unparseable task file exits 0. Blocking the operator's work because this script
# has a bug would be a worse outcome than the defect it prevents — the opposite
# choice from the fail-CLOSED static checks, and made on purpose.
#
# Usage:
#   gate-inception-voi.sh --task <path>   gate one file (hook mode; non-zero = block)
#   gate-inception-voi.sh --scan          report every firing task (guard-layer mode)
#   gate-inception-voi.sh --json          machine-readable
# Exit: 0 clear/waived/grandfathered/not-an-inception - 1 FIRING - 2 usage.
set -uo pipefail

CUTOFF="${FW_VOI_GATE_CUTOFF:-2026-09-26}"
TASKS_DIR="${FW_VOI_GATE_TASKS_DIR:-.tasks/active}"
LEDGER="${FW_VOI_GATE_LEDGER:-.context/checks/voi-waiver-ledger}"
MODE=""; TASK=""; JSON=0

while [ $# -gt 0 ]; do
    case "$1" in
        --task)   MODE=task; TASK="${2:-}"; shift 2 ;;
        --scan)   MODE=scan; shift ;;
        --json)   JSON=1; shift ;;
        --cutoff) CUTOFF="${2:-}"; shift 2 ;;
        --tasks-dir) TASKS_DIR="${2:-}"; shift 2 ;;
        --ledger) LEDGER="${2:-}"; shift 2 ;;
        --no-heartbeat) shift ;;
        -h|--help) sed -n '2,44p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "gate-inception-voi: unknown argument: $1" >&2; exit 2 ;;
    esac
done
[ -n "$MODE" ] || MODE=scan

command -v python3 >/dev/null 2>&1 || exit 0   # fail-open: no python, no opinion

MODE="$MODE" TASK="$TASK" JSON="$JSON" CUTOFF="$CUTOFF" TASKS_DIR="$TASKS_DIR" LEDGER="$LEDGER" \
python3 <<'PYEOF'
import glob, json, os, re, sys, datetime

mode   = os.environ["MODE"]
one    = os.environ["TASK"]
as_json= os.environ["JSON"] == "1"
cutoff = os.environ["CUTOFF"].strip()
tdir   = os.environ["TASKS_DIR"]
ledger = os.environ["LEDGER"]

try:
    import yaml
except ImportError:
    sys.exit(0)                      # fail-open

FM = re.compile(r"^---\n(.*?)\n---", re.S)
DEFAULT = 0.5

def load(path):
    try:
        t = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        return None
    m = FM.search(t)
    if not m:
        return None
    try:
        fm = yaml.safe_load(m.group(1))
    except Exception:
        return None                  # fail-open on unparseable frontmatter
    return fm if isinstance(fm, dict) else None

def created_before_cutoff(fm):
    c = str(fm.get("created") or "")[:10]
    if not c or not cutoff:
        return False                 # unknown date -> treat as NEW, gate applies
    return c < cutoff

def waiver_of(fm):
    """Returns (waived, reason). An empty/whitespace reason is NOT a waiver."""
    w = fm.get("voi_score_waived")
    if w is None:
        return False, ""
    r = str(w).strip()
    if not r or r.lower() in ("true", "yes", "none", "null"):
        return False, r              # a bare truthy value states no reason
    return True, r

def classify(path):
    fm = load(path)
    if fm is None:
        return "unreadable", None, ""
    if str(fm.get("workflow_type", "")).strip() != "inception":
        return "not-inception", fm, ""
    waived, reason = waiver_of(fm)
    v = fm.get("voi_score")
    unset = v is None
    if not unset:
        try:
            unset = abs(float(v) - DEFAULT) < 1e-9
        except (TypeError, ValueError):
            unset = True             # malformed reads as unset
    if not unset:
        return "ok", fm, ""
    if waived:
        return "waived", fm, reason
    if created_before_cutoff(fm):
        return "grandfathered", fm, ""
    return "FIRING", fm, ""

def log_waiver(tid, reason):
    try:
        os.makedirs(os.path.dirname(ledger), exist_ok=True)
        new = not os.path.exists(ledger)
        with open(ledger, "a", encoding="utf-8") as fh:
            if new:
                fh.write("# voi_score waiver ledger (T-3174). Append-only.\n"
                         "# A waiver is legitimate; a waiver nobody can count is not.\n"
                         "# <iso-ts>\\t<task>\\t<reason>\n")
            fh.write("%s\t%s\t%s\n" % (
                datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
                tid, reason.replace("\t", " ")))
    except OSError:
        pass                         # never block on a logging failure

def tid_of(path, fm):
    return (fm or {}).get("id") or os.path.basename(path).split("-")[0:2] and os.path.basename(path)[:8]

if mode == "task":
    if not one or not os.path.isfile(one):
        sys.exit(0)                  # fail-open
    state, fm, reason = classify(one)
    tid = tid_of(one, fm)
    if state == "waived":
        log_waiver(tid, reason)
    if as_json:
        print(json.dumps({"ok": state != "FIRING", "state": state,
                          "task": one, "reason": reason}))
    elif state == "FIRING":
        print("BLOCKED: %s is an inception with no considered voi_score." % tid, file=sys.stderr)
        print("", file=sys.stderr)
        print("  voi_score is absent or still the template default (0.5). Both map to the", file=sys.stderr)
        print("  same internal 2 as 'nobody looked at this', so the task would rank ahead", file=sys.stderr)
        print("  of work that WAS measured. 244 of 245 inceptions here are in that state.", file=sys.stderr)
        print("", file=sys.stderr)
        print("  Set a considered value:   voi_score: 0.0 .. 1.0", file=sys.stderr)
        print("  Or waive it WITH a reason (logged, not silent):", file=sys.stderr)
        print('      voi_score_waived: "why answering this is not worth scoring"', file=sys.stderr)
        print("", file=sys.stderr)
        print("  An empty reason does not waive — that is the unset field with extra steps.", file=sys.stderr)
    sys.exit(1 if state == "FIRING" else 0)

# scan mode
rows = {"FIRING": [], "waived": [], "grandfathered": [], "ok": [], "not-inception": 0, "unreadable": 0}
for f in sorted(glob.glob(os.path.join(tdir, "*.md"))):
    state, fm, reason = classify(f)
    if state in ("not-inception", "unreadable"):
        rows[state] += 1
    else:
        rows[state].append((tid_of(f, fm), f, reason))

fires = len(rows["FIRING"])
if as_json:
    print(json.dumps({
        "ok": fires == 0, "cutoff": cutoff, "firing_count": fires,
        "firing": [{"id": i, "path": p} for i, p, _ in rows["FIRING"]],
        "waived_count": len(rows["waived"]),
        "waived": [{"id": i, "reason": r} for i, _, r in rows["waived"]],
        "grandfathered_count": len(rows["grandfathered"]),
        "ok_count": len(rows["ok"]),
        "scope": ("detects inceptions whose voi_score is absent or the 0.5 template default. "
                  "It does NOT judge whether a set value is a GOOD estimate."),
    }, indent=2))
    sys.exit(1 if fires else 0)

print("voi gate — cutoff %s (tasks created before it are grandfathered)" % cutoff)
print("  firing: %d   waived: %d   grandfathered: %d   set: %d"
      % (fires, len(rows["waived"]), len(rows["grandfathered"]), len(rows["ok"])))
for i, p, _ in rows["FIRING"]:
    print("  FIRING  %s" % p)
if rows["waived"]:
    print("\n  waived (counted and reported, never hidden):")
    for i, _, r in rows["waived"]:
        print("    %s: %s" % (i, r[:110]))
print("\nscope: detects an absent or default voi_score. It does NOT judge whether a set value")
print("is a good estimate — only that somebody answered the question.")
sys.exit(1 if fires else 0)
PYEOF
