#!/usr/bin/env bash
# scripts/check-human-ac-escalation.sh — T-3186
#
# guard-layer: source warn  # human-AC backlog escalation, human-owned by definition
#
# THE ESCALATION BAR, made load-bearing.
#
# A criterion belongs in `### Human` when the agent is genuinely BLOCKED — when it cannot
# produce the evidence itself. It does NOT belong there merely because a ruling would be
# nice to have. Anything settled by a deterministic command belongs in `### Agent` as a
# `[REVIEWER]` AC with the check in `## Verification` (T-1811/T-1878 prefix routing,
# PD-173 precedent).
#
# Why this exists rather than being documented advice: the convention was discipline-only,
# and discipline lost. 832-Workflow-designer measured it at framework:pickup offset 192
# — "agents generate these faster than any human issues them, and that is ours to fix
# rather than theirs" — and this repo then demonstrated it. In one session, five units of
# work produced SEVEN new items in the review queue, taking it 128 -> 135. Net effect on
# the operator's backlog: negative.
#
# WHAT IT DOES NOT DO, and this is the load-bearing restraint: it DETECTS AND NEVER
# REASSIGNS. PD-173 records that converting an AC is a reclassification the operator
# authorises case by case, and that "batch-closing on the back of a reclassification would
# launder a sovereignty boundary". A check that auto-converted would do exactly that. It
# also closes nothing.
#
# Scope, stated so a green is not over-read (T-2680): this checks the PREFIX an author
# declared. It cannot tell whether a `[REVIEW]` AC genuinely needs judgement, and it does
# not audit task ownership. A clean run means no self-declared-mechanical criterion is
# sitting in the human queue unacknowledged — nothing more.
#
# Exit 0 = clean · 1 = unacknowledged escalation(s) · 2 = tooling (fail-closed).

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
TASKS_DIR="${HUMAN_AC_TASKS_DIR:-$PROJECT_ROOT/.tasks/active}"
ALLOWLIST="${HUMAN_AC_ESCALATION_ALLOWLIST:-$PROJECT_ROOT/.context/checks/human-ac-escalation-allowlist}"
JSON=0; QUIET=0

while [ $# -gt 0 ]; do
    case "$1" in
        --json)       JSON=1 ;;
        --quiet)      QUIET=1 ;;
        --tasks-dir)  TASKS_DIR="$2"; shift ;;
        --allowlist)  ALLOWLIST="$2"; shift ;;
        --no-heartbeat) : ;;
        -h|--help)
            echo "Usage: $0 [--json] [--quiet] [--tasks-dir DIR] [--allowlist FILE]"
            exit 0 ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

command -v python3 >/dev/null 2>&1 || { echo "TOOLING: python3 not found" >&2; exit 2; }
[ -d "$TASKS_DIR" ] || { echo "TOOLING: tasks dir not found: $TASKS_DIR" >&2; exit 2; }

export HAC_TASKS_DIR="$TASKS_DIR" HAC_ALLOWLIST="$ALLOWLIST" HAC_JSON="$JSON" HAC_QUIET="$QUIET"

python3 <<'PY'
import os, re, sys, json, glob

tasks_dir = os.environ['HAC_TASKS_DIR']
allow_path = os.environ['HAC_ALLOWLIST']
as_json = os.environ['HAC_JSON'] == '1'
quiet = os.environ['HAC_QUIET'] == '1'

files = sorted(glob.glob(os.path.join(tasks_dir, '*.md')))
if not files:
    # FAIL-CLOSED: an empty corpus is a tooling fault, never a clean bill. A check that
    # scanned nothing and reported PASS asserts coverage it does not have (T-3105).
    print("TOOLING: no task files found in %s" % tasks_dir, file=sys.stderr)
    sys.exit(2)

# ---- acknowledgement ledger -------------------------------------------------
# An ABSENT ledger acknowledges nothing rather than excusing everything.
acknowledged = {}
if os.path.exists(allow_path):
    try:
        for line in open(allow_path):
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            task, _, reason = line.partition('#')
            acknowledged[task.strip()] = reason.strip()
    except OSError as exc:
        print("TOOLING: allowlist unreadable: %s" % exc, file=sys.stderr)
        sys.exit(2)

COMMENT = re.compile(r'<!--.*?-->', re.S)
HUMAN = re.compile(r'^### Human\b(.*?)(?=^## |\Z)', re.S | re.M)
OPEN_AC = re.compile(r'^\s*-\s*\[ \]\s*(.*)$', re.M)

firing, ack_hits, reviewed = [], [], 0
for path in files:
    raw = open(path, encoding='utf-8', errors='replace').read()
    m = HUMAN.search(raw)
    if not m:
        continue
    # Strip HTML comment regions FIRST. The task template's own Human section carries
    # worked [RUBBER-STAMP] and [REVIEWER] EXAMPLES inside comments; without this every
    # task in the corpus fires. The same trap defeated five detectors in one session
    # (registered learning, T-3178) — prose about a pattern is not a use of it.
    body = COMMENT.sub('', m.group(1))
    tid_m = re.match(r'(T-\d+)', os.path.basename(path))
    tid = tid_m.group(1) if tid_m else os.path.basename(path)
    for ac in OPEN_AC.findall(body):
        # Both classes are SELF-DECLARED by the author, so the check never guesses at
        # whether something needs judgement. A [REVIEW] AC is never flagged.
        if '[RUBBER-STAMP]' in ac:
            why = 'RUBBER-STAMP: author declared it mechanical'
        elif '[REVIEWER]' in ac:
            why = 'REVIEWER prefix inside ### Human: belongs in ### Agent'
        else:
            continue
        reviewed += 1
        rec = {'task': tid, 'why': why, 'ac': ac.strip()[:110]}
        (ack_hits if tid in acknowledged else firing).append(rec)

if as_json:
    print(json.dumps({
        'ok': not firing,
        'firing': firing,
        'firing_count': len(firing),
        'acknowledged': ack_hits,
        'acknowledged_count': len(ack_hits),
        'escalations_total': reviewed,
        'tasks_scanned': len(files),
        'scope': 'declared prefixes only; does not judge whether a [REVIEW] AC needs judgement',
    }, indent=2))
elif firing:
    print("check-human-ac-escalation: %d unacknowledged escalation(s) of %d declared-mechanical "
          "criteri%s across %d task file(s)"
          % (len(firing), reviewed, 'on' if reviewed == 1 else 'a', len(files)))
    for r in firing:
        print("  %-9s %s" % (r['task'], r['why']))
        print("            %s" % r['ac'])
    if ack_hits:
        print("  (%d acknowledged, not counted above)" % len(ack_hits))
    print()
    print("  Each belongs in ### Agent as a [REVIEWER] AC with the check in ## Verification")
    print("  (T-1811/T-1878 routing, PD-173 precedent). CONVERTING IS THE OPERATOR'S CALL —")
    print("  this check detects and never reassigns, and reclassification closes nothing.")
    print("  If a listed one genuinely needs judgement, re-prefix it [REVIEW]; if it is a")
    print("  known-pending decision, acknowledge it in:")
    print("    %s" % allow_path)
elif not quiet:
    print("check-human-ac-escalation: clean — %d declared-mechanical criteri%s, %d acknowledged, "
          "%d task file(s) scanned"
          % (reviewed, 'on' if reviewed == 1 else 'a', len(ack_hits), len(files)))
    print("  Scope: declared prefixes only. It cannot tell whether a [REVIEW] AC genuinely")
    print("  needs judgement, and does not audit task ownership.")

sys.exit(1 if firing else 0)
PY
