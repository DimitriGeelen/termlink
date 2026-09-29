#!/usr/bin/env bash
# guard-layer: source warn --check --quiet  # the operator's open voi_score question (T-3200), not a code defect; header says it only reports
#
# T-3175 — ask about an unset voi_score, remember the answer, re-ask after N runs.
#
# SUPERSEDES the T-3174 blocking gate. Operator's instruction: "I don't want the gate
# to block. It should stop and give me a choice. And then record the choice so I'm not
# being asked every time about it. Then maybe after 5 or 10 runs you can ask me again."
#
# THE CONSTRAINT THAT SHAPES THIS. A PreToolUse hook cannot ask anything — hooks run
# non-interactively and can only exit and print. So the thing that asks is the AGENT,
# in conversation. This script is therefore not a gate: it is the MEMORY behind a
# conversation. It answers "is there anything I should raise right now, and what did
# the operator already say about it?" The agent reads that, asks if needed, and records
# the reply. No settings.json wiring is involved, which is why none is proposed.
#
# WHY A SNOOZE AND NOT JUST A WAIVER. A permanent waiver and "not now" are different
# answers, and collapsing them is how a prompt becomes a nag or a silence. A waiver is
# a judgement that the question does not apply. A snooze is a judgement that it is not
# worth answering YET — and that one should expire, because circumstances change and an
# unexpiring "not now" is indistinguishable from having disabled the thing.
#
# WHY RUNS AND NOT DAYS. The operator said "after 5 or 10 runs". A run is a --check
# invocation, so the snooze decays with how often the question would actually have come
# up, not with wall-clock time. A project left alone for a month does not greet you with
# expired snoozes on return.
#
# TWO SCOPES, because they are different requests:
#   --task T-XXX   stop asking about THIS inception
#   --global       stop asking about voi_score at all
#
# Store: .context/checks/voi-decisions.yaml (git-tracked). Holds current state per key
# plus an append-only `history:` so "what did we decide and when" survives.
#
# FAILS OPEN, ALWAYS. It asks; it does not gate. Any error exits 0 with nothing pending.
#
# Usage:
#   voi-prompt.sh --check [--json] [--quiet]
#   voi-prompt.sh --record --task T-XXX --choice set    --value 0.8
#   voi-prompt.sh --record --task T-XXX --choice waive  --reason "..."
#   voi-prompt.sh --record --task T-XXX --choice snooze [--snooze 10]
#   voi-prompt.sh --record --global     --choice snooze [--snooze 10]
#   voi-prompt.sh --status
# Exit: 0 nothing to ask / recorded ok · 1 something needs a decision · 2 usage.
set -uo pipefail

STORE="${FW_VOI_STORE:-.context/checks/voi-decisions.yaml}"
TASKS_DIR="${FW_VOI_TASKS_DIR:-.tasks/active}"
CUTOFF="${FW_VOI_CUTOFF:-2026-09-26}"
DEFAULT_SNOOZE="${FW_VOI_DEFAULT_SNOOZE:-10}"
MODE=""; TASK=""; CHOICE=""; VALUE=""; REASON=""; SNOOZE=""; SCOPE=""; JSON=0; QUIET=0

while [ $# -gt 0 ]; do
    case "$1" in
        --check)   MODE=check; shift ;;
        --record)  MODE=record; shift ;;
        --status)  MODE=status; shift ;;
        --task)    TASK="${2:-}"; SCOPE=task; shift 2 ;;
        --global)  SCOPE=global; shift ;;
        --choice)  CHOICE="${2:-}"; shift 2 ;;
        --value)   VALUE="${2:-}"; shift 2 ;;
        --reason)  REASON="${2:-}"; shift 2 ;;
        --snooze)  SNOOZE="${2:-}"; shift 2 ;;
        --store)   STORE="${2:-}"; shift 2 ;;
        --tasks-dir) TASKS_DIR="${2:-}"; shift 2 ;;
        --cutoff)  CUTOFF="${2:-}"; shift 2 ;;
        --json)    JSON=1; shift ;;
        --quiet)   QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        -h|--help) sed -n '2,48p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "voi-prompt: unknown argument: $1" >&2; exit 2 ;;
    esac
done
[ -n "$MODE" ] || MODE=check
command -v python3 >/dev/null 2>&1 || exit 0     # fail-open

MODE="$MODE" TASK="$TASK" CHOICE="$CHOICE" VALUE="$VALUE" REASON="$REASON" \
SNOOZE="$SNOOZE" SCOPE="$SCOPE" JSON="$JSON" QUIET="$QUIET" STORE="$STORE" \
TASKS_DIR="$TASKS_DIR" CUTOFF="$CUTOFF" DEFAULT_SNOOZE="$DEFAULT_SNOOZE" \
python3 <<'PYEOF'
import glob, json, os, re, sys, datetime

E = os.environ
mode, task, choice = E["MODE"], E["TASK"], E["CHOICE"]
value, reason, scope = E["VALUE"], E["REASON"], E["SCOPE"]
as_json, quiet = E["JSON"] == "1", E["QUIET"] == "1"
store, tdir, cutoff = E["STORE"], E["TASKS_DIR"], E["CUTOFF"].strip()
try:
    default_snooze = max(1, int(E["DEFAULT_SNOOZE"]))
except ValueError:
    default_snooze = 10
snooze_n = E["SNOOZE"]

def die_open(msg=""):
    if as_json: print(json.dumps({"ok": True, "pending": [], "note": msg or "fail-open"}))
    elif msg and not quiet: print("voi-prompt: %s (failing open)" % msg, file=sys.stderr)
    sys.exit(0)

try:
    import yaml
except ImportError:
    die_open("PyYAML unavailable")

NOW = datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")
FM = re.compile(r"^---\n(.*?)\n---", re.S)
DEFAULT_VOI = 0.5

def load_store():
    if not os.path.exists(store):
        return {"runs": 0, "decisions": {}, "history": []}
    try:
        d = yaml.safe_load(open(store, encoding="utf-8")) or {}
    except Exception:
        return None
    if not isinstance(d, dict):
        return None
    d.setdefault("runs", 0)
    d.setdefault("decisions", {})
    d.setdefault("history", [])
    if not isinstance(d["decisions"], dict): d["decisions"] = {}
    if not isinstance(d["history"], list): d["history"] = []
    try: d["runs"] = int(d["runs"])
    except (TypeError, ValueError): d["runs"] = 0
    return d

def save_store(d):
    try:
        os.makedirs(os.path.dirname(store), exist_ok=True)
        tmp = store + ".tmp"
        with open(tmp, "w", encoding="utf-8") as fh:
            fh.write("# voi_score decision store (T-3175). Managed by scripts/voi-prompt.sh.\n"
                     "# `runs` counts --check invocations; a snooze recorded at run R is\n"
                     "# askable again at R + snooze_runs. `history` is append-only.\n")
            yaml.safe_dump(d, fh, sort_keys=False, allow_unicode=True, width=100)
        os.replace(tmp, store)
        return True
    except OSError:
        return False

def task_files():
    return sorted(glob.glob(os.path.join(tdir, "*.md")))

def read_fm(path):
    try: t = open(path, encoding="utf-8", errors="replace").read()
    except OSError: return None
    m = FM.search(t)
    if not m: return None
    try: fm = yaml.safe_load(m.group(1))
    except Exception: return None
    return fm if isinstance(fm, dict) else None

def needs_answer(fm):
    """True when voi_score is absent or the untouched template default."""
    v = fm.get("voi_score")
    if v is None: return True
    try: return abs(float(v) - DEFAULT_VOI) < 1e-9
    except (TypeError, ValueError): return True

def is_new(fm):
    c = str(fm.get("created") or "")[:10]
    return not (c and cutoff and c < cutoff)

d = load_store()
if d is None:
    die_open("decision store unparseable")

# ---------- record ----------
if mode == "record":
    if scope == "task":
        if not task: print("voi-prompt: --task needs a task id", file=sys.stderr); sys.exit(2)
        key = "task:%s" % task
    elif scope == "global":
        key = "global"
    else:
        print("voi-prompt: --record needs --task <id> or --global", file=sys.stderr); sys.exit(2)
    if choice not in ("set", "waive", "snooze"):
        print("voi-prompt: --choice must be set|waive|snooze", file=sys.stderr); sys.exit(2)
    if choice == "waive" and not reason.strip():
        print("voi-prompt: a permanent waiver needs --reason. An unreasoned waiver is", file=sys.stderr)
        print("  just the unset field with extra steps — use --choice snooze for 'not now'.", file=sys.stderr)
        sys.exit(2)
    if choice == "set" and not value.strip():
        print("voi-prompt: --choice set needs --value <0..1>", file=sys.stderr); sys.exit(2)

    try: n = max(1, int(snooze_n)) if snooze_n else default_snooze
    except ValueError: n = default_snooze

    entry = {"choice": choice, "recorded_at": NOW, "recorded_at_run": d["runs"]}
    if choice == "snooze":
        entry["snooze_runs"] = n
        # +1 because the NEXT check is run R+1: N runs of silence are runs
        # R+1..R+N, so the question returns at R+N+1. Without the +1 a
        # `--snooze 10` goes quiet for 9 runs, which is not what was asked for.
        # Caught by the fixture, not by reading it back.
        entry["askable_at_run"] = d["runs"] + n + 1
    if reason.strip(): entry["reason"] = reason.strip()
    if value.strip(): entry["value"] = value.strip()
    d["decisions"][key] = entry
    d["history"].append(dict(key=key, **entry))
    ok = save_store(d)
    if as_json:
        print(json.dumps({"ok": ok, "recorded": key, "entry": entry}))
    elif not quiet:
        if choice == "snooze":
            print("recorded: %s snoozed for %d run(s) — askable again at run %d (now %d)"
                  % (key, n, entry["askable_at_run"], d["runs"]))
        elif choice == "waive":
            print("recorded: %s waived permanently — reason: %s" % (key, reason.strip()))
        else:
            print("recorded: %s set to %s" % (key, value.strip()))
        if not ok: print("  WARNING: store could not be written; the decision will not persist", file=sys.stderr)
    sys.exit(0)

# ---------- status ----------
if mode == "status":
    out = {"runs": d["runs"], "decisions": d["decisions"], "history_len": len(d["history"])}
    if as_json: print(json.dumps(out, indent=2)); sys.exit(0)
    print("voi-prompt store: %s" % store)
    print("check runs so far: %d" % d["runs"])
    if not d["decisions"]:
        print("no decisions recorded yet")
    for k, e in d["decisions"].items():
        if e.get("choice") == "snooze":
            left = e.get("askable_at_run", 0) - d["runs"]
            print("  %-26s snoozed, %s"
                  % (k, ("%d run(s) left" % left) if left > 0 else "EXPIRED — will be re-asked"))
        elif e.get("choice") == "waive":
            print("  %-26s waived — %s" % (k, e.get("reason", "")[:70]))
        else:
            print("  %-26s set to %s" % (k, e.get("value")))
    print("history entries: %d (append-only)" % len(d["history"]))
    sys.exit(0)

# ---------- check ----------
d["runs"] += 1
run = d["runs"]

g = d["decisions"].get("global")
global_suppressed = False
global_note = ""
if g:
    if g.get("choice") == "waive":
        global_suppressed, global_note = True, "voi_score waived globally: %s" % g.get("reason", "")
    elif g.get("choice") == "snooze":
        left = g.get("askable_at_run", 0) - run
        if left > 0:
            global_suppressed = True
            global_note = "snoozed globally, %d run(s) left" % left
        else:
            global_note = "GLOBAL SNOOZE EXPIRED (was %d runs) — re-asking" % g.get("snooze_runs", 0)

pending, suppressed = [], []
if not global_suppressed:
    for f in task_files():
        fm = read_fm(f)
        if fm is None: continue
        if str(fm.get("workflow_type", "")).strip() != "inception": continue
        if not needs_answer(fm): continue
        if not is_new(fm): continue
        tid = fm.get("id") or os.path.basename(f)
        e = d["decisions"].get("task:%s" % tid)
        if e:
            if e.get("choice") == "waive":
                suppressed.append((tid, "waived: %s" % e.get("reason", "")[:60])); continue
            if e.get("choice") == "snooze":
                left = e.get("askable_at_run", 0) - run
                if left > 0:
                    suppressed.append((tid, "snoozed, %d run(s) left" % left)); continue
                pending.append((tid, f, "re-ask: snooze of %d run(s) expired" % e.get("snooze_runs", 0)))
                continue
            if e.get("choice") == "set":
                suppressed.append((tid, "answered %s (file not yet updated)" % e.get("value"))); continue
        pending.append((tid, f, "never asked"))

save_store(d)

if as_json:
    print(json.dumps({
        "ok": not pending, "run": run,
        "global_suppressed": global_suppressed, "global_note": global_note,
        "pending_count": len(pending),
        "pending": [{"id": i, "path": p, "why": w} for i, p, w in pending],
        "suppressed_count": len(suppressed),
        "suppressed": [{"id": i, "state": s} for i, s in suppressed],
        "default_snooze": default_snooze,
        "scope": ("asks whether somebody answered voi_score. It does NOT judge whether a set "
                  "value is a good estimate, and it never blocks — it only reports."),
    }, indent=2))
    sys.exit(1 if pending else 0)

if quiet and not pending:
    sys.exit(0)

print("voi-prompt — check run %d" % run)
if global_note:
    print("  global: %s" % global_note)
if not pending:
    print("  nothing to ask." + ("  (%d suppressed by an earlier decision)" % len(suppressed) if suppressed else ""))
else:
    print("  %d inception(s) need a decision on voi_score:" % len(pending))
    for i, p, w in pending:
        print("    %-10s %s   [%s]" % (i, p, w))
    print("")
    print("  Ask the operator, then record the reply — one of:")
    print("    scripts/voi-prompt.sh --record --task <ID> --choice set --value 0.8")
    print('    scripts/voi-prompt.sh --record --task <ID> --choice waive --reason "..."')
    print("    scripts/voi-prompt.sh --record --task <ID> --choice snooze            # default %d runs" % default_snooze)
    print("    scripts/voi-prompt.sh --record --global --choice snooze --snooze 10   # stop asking at all")
if suppressed and not quiet:
    print("\n  suppressed by an earlier decision (counted, never hidden):")
    for i, s in suppressed:
        print("    %-10s %s" % (i, s))
print("\nscope: asks whether somebody answered voi_score. It does not judge whether a set value")
print("is a good estimate, and it never blocks — it only reports.")
sys.exit(1 if pending else 0)
PYEOF
