#!/usr/bin/env bash
# check-budget-ladder-drift.sh — does the DOCUMENTED context-budget ladder match the LIVE gate?
# guard-layer: source --no-heartbeat
#
# T-3029. CLAUDE.md states the budget ladder twice: as absolute token counts
# ("**120K** ok->warn ... **170K** urgent->critical (**BLOCK**)") and as percentage bands
# ("Above 85% (170K+): handover immediately"). Enforcement lives in budget-gate.sh and
# checkpoint.sh, which compute thresholds as percentages of CONTEXT_WINDOW. Nothing ever
# compared the two.
#
# WHY THIS EXISTS. T-131 (2026-03-14) replaced a hardcoded 200000 in BOTH scripts, with an
# acceptance criterion reading "No remaining hardcoded 200000 or 200K references" — verified
# by `! grep -q '200000' <script>` against the two files it had just edited. The identical
# constant lived on in the prose that TELLS AN AGENT WHEN TO STOP WORKING, which was never in
# the grep's scope. The AC claimed a global property; the check proved a local one. The prose
# stayed 200K-shaped for six months and two further threshold migrations.
#
# The failure direction is a productivity tax, not a safety hole: the documented numbers are
# CONSERVATIVE, so an agent obeying them stops early. Measured cost is real — the 2026-09-20
# arc-008 run parked a task unexecuted citing "~80% context" at ~178K, which is 59% of the
# real window and level "ok".
#
# Scope — read a green narrowly (T-2680): this compares the NUMBERS the prose asserts against
# the NUMBERS the gate computes, plus whether the two enforcing scripts agree with each other.
# It does NOT verify that the ladder is well chosen, that the surrounding prose is otherwise
# accurate, or that any other document agrees.
#
# Exit 0 = no unacknowledged drift | 1 = drift | 2 = tooling (fail-closed).

set -uo pipefail

CLAUDE_MD="${BUDGET_LADDER_CLAUDE_MD:-CLAUDE.md}"
GATE="${BUDGET_LADDER_GATE:-.agentic-framework/agents/context/budget-gate.sh}"
CHECKPOINT="${BUDGET_LADDER_CHECKPOINT:-.agentic-framework/agents/context/checkpoint.sh}"
ALLOWLIST="${BUDGET_LADDER_ALLOWLIST:-.context/checks/budget-ladder-allowlist}"
JSON=0; QUIET=0

while [ $# -gt 0 ]; do
    case "$1" in
        --json)          JSON=1 ;;
        --quiet)         QUIET=1 ;;
        --no-heartbeat)  : ;;
        --claude-md)     CLAUDE_MD="$2"; shift ;;
        --gate)          GATE="$2"; shift ;;
        --checkpoint)    CHECKPOINT="$2"; shift ;;
        --allowlist)     ALLOWLIST="$2"; shift ;;
        -h|--help)
            echo "Usage: check-budget-ladder-drift.sh [--json] [--quiet] [--claude-md P] [--gate P] [--checkpoint P] [--allowlist P]"
            exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

command -v python3 >/dev/null 2>&1 || { echo "check-budget-ladder-drift: python3 not found" >&2; exit 2; }

BUDGET_LADDER_CLAUDE_MD="$CLAUDE_MD" \
BUDGET_LADDER_GATE="$GATE" \
BUDGET_LADDER_CHECKPOINT="$CHECKPOINT" \
BUDGET_LADDER_ALLOWLIST="$ALLOWLIST" \
BUDGET_LADDER_JSON="$JSON" \
BUDGET_LADDER_QUIET="$QUIET" \
python3 - <<'PY'
import json, os, re, sys

claude_md  = os.environ["BUDGET_LADDER_CLAUDE_MD"]
gate       = os.environ["BUDGET_LADDER_GATE"]
checkpoint = os.environ["BUDGET_LADDER_CHECKPOINT"]
allowlist  = os.environ["BUDGET_LADDER_ALLOWLIST"]
as_json    = os.environ["BUDGET_LADDER_JSON"] == "1"
quiet      = os.environ["BUDGET_LADDER_QUIET"] == "1"

def tooling(msg):
    # Fail-closed: never report clean because we could not look (T-2818).
    if as_json:
        print(json.dumps({"ok": False, "verdict": "tooling", "error": msg}))
    else:
        print("check-budget-ladder-drift: %s" % msg, file=sys.stderr)
    sys.exit(2)

def read(path):
    try:
        with open(path, encoding="utf-8") as fh:
            return fh.read()
    except OSError as exc:
        tooling("cannot read %s: %s" % (path, exc))

gate_src = read(gate)
ckpt_src = read(checkpoint)
doc_src  = read(claude_md)

def parse_gate(src, path):
    m = re.search(r'CONTEXT_WINDOW=\$\(fw_config_int\s+"CONTEXT_WINDOW"\s+(\d+)\)', src)
    if not m:
        tooling("cannot find CONTEXT_WINDOW default in %s" % path)
    window = int(m.group(1))
    pct = {}
    for level in ("WARN", "URGENT", "CRITICAL"):
        p = re.search(r'TOKEN_%s=\$\(\(CONTEXT_WINDOW\s*\*\s*(\d+)\s*/\s*100\)\)' % level, src)
        if not p:
            tooling("cannot find TOKEN_%s percentage in %s" % (level, path))
        pct[level.lower()] = int(p.group(1))
    return window, pct

gate_window, gate_pct = parse_gate(gate_src, gate)
ckpt_window, ckpt_pct = parse_gate(ckpt_src, checkpoint)

live = {k: gate_window * v // 100 for k, v in gate_pct.items()}

findings = []

# --- code vs code: the two enforcing scripts must agree ---------------------------------
if (gate_window, gate_pct) != (ckpt_window, ckpt_pct):
    findings.append({
        "signature": "code::gate-vs-checkpoint",
        "why": "budget-gate.sh (window=%d pct=%s) and checkpoint.sh (window=%d pct=%s) disagree; "
               "an agent reading `checkpoint.sh status` is told a different ladder than the one that blocks it"
               % (gate_window, gate_pct, ckpt_window, ckpt_pct),
    })

# --- prose absolutes: the escalation-ladder line ----------------------------------------
lad = re.search(
    r'Escalation ladder:.*?\*\*(\d+)K\*\*\s*ok.{0,4}warn.*?\*\*(\d+)K\*\*\s*warn.{0,4}urgent.*?\*\*(\d+)K\*\*\s*urgent.{0,4}critical',
    doc_src, re.S)
if lad:
    prose_abs = {"warn": int(lad.group(1)) * 1000,
                 "urgent": int(lad.group(2)) * 1000,
                 "critical": int(lad.group(3)) * 1000}
    if prose_abs != live:
        findings.append({
            "signature": "%s::escalation-ladder-absolutes" % os.path.basename(claude_md),
            "why": "prose says warn=%d urgent=%d critical=%d; gate computes warn=%d urgent=%d critical=%d "
                   "(window=%d)" % (prose_abs["warn"], prose_abs["urgent"], prose_abs["critical"],
                                    live["warn"], live["urgent"], live["critical"], gate_window),
        })
else:
    prose_abs = None

# --- prose percentage bands: the Work Proposal Rule -------------------------------------
bands = re.findall(r'^-\s*(?:Below|Above)\s+(\d+)%\s*\((\d+)K', doc_src, re.M)
if bands:
    stated_pcts = sorted({int(p) for p, _ in bands})
    live_pcts   = sorted({gate_pct["warn"], gate_pct["urgent"], gate_pct["critical"]})
    band_abs    = sorted({int(t) * 1000 for _, t in bands})
    live_abs    = sorted({live["warn"], live["urgent"], live["critical"]})
    if stated_pcts != live_pcts or band_abs != live_abs:
        findings.append({
            "signature": "%s::work-proposal-bands" % os.path.basename(claude_md),
            "why": "prose bands %s%% (%s tokens) vs gate %s%% (%s tokens)"
                   % (stated_pcts, band_abs, live_pcts, live_abs),
        })

# --- prose self-consistency: the structural-enforcement sentence ------------------------
crit = re.search(r'critical level\s*\(>=\s*(\d+)K\s*tokens,\s*~(\d+)%\)', doc_src)
if crit:
    c_abs, c_pct = int(crit.group(1)) * 1000, int(crit.group(2))
    if c_abs != live["critical"] or c_pct != gate_pct["critical"]:
        extra = ""
        if prose_abs and c_abs != prose_abs["critical"]:
            extra = (" — and it contradicts this same file's own escalation ladder, which puts "
                     "critical at %d" % prose_abs["critical"])
        findings.append({
            "signature": "%s::structural-enforcement-critical" % os.path.basename(claude_md),
            "why": "prose says the gate blocks at >=%d tokens (~%d%%); it blocks at %d (%d%%)%s"
                   % (c_abs, c_pct, live["critical"], gate_pct["critical"], extra),
        })

if prose_abs is None and not bands and not crit:
    # Every anchor missing at once means the prose was restructured and this check went blind.
    tooling("found no budget-ladder statement at all in %s — anchors stale, check is blind" % claude_md)

# --- acknowledgement ledger --------------------------------------------------------------
ack = {}
if os.path.exists(allowlist):
    try:
        with open(allowlist, encoding="utf-8") as fh:
            for line in fh:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                sig, _, reason = line.partition("#")
                ack[sig.strip()] = reason.strip()
    except OSError as exc:
        tooling("cannot read allowlist %s: %s" % (allowlist, exc))

firing       = [f for f in findings if f["signature"] not in ack]
acknowledged = [f for f in findings if f["signature"] in ack]

scope = ("compares the numbers the prose asserts against the numbers the gate computes; "
         "does not judge whether the ladder is well chosen")

if as_json:
    print(json.dumps({
        "ok": not firing,
        "firing": firing,
        "acknowledged": [dict(f, reason=ack[f["signature"]]) for f in acknowledged],
        "drift_total": len(findings),
        "live": {"window": gate_window, "pct": gate_pct, "tokens": live},
        "scope": scope,
    }, indent=2))
elif firing:
    print("BUDGET LADDER DRIFT — %d unacknowledged" % len(firing))
    for f in firing:
        print("  %s\n      %s" % (f["signature"], f["why"]))
    print("\nLive gate: window=%d warn=%d(%d%%) urgent=%d(%d%%) critical=%d(%d%%)"
          % (gate_window, live["warn"], gate_pct["warn"], live["urgent"], gate_pct["urgent"],
             live["critical"], gate_pct["critical"]))
    print("Scope: %s" % scope)
    print("Fix the prose, or acknowledge in %s with a cited reason." % allowlist)
elif not quiet:
    print("budget ladder: no unacknowledged drift (%d acknowledged)" % len(acknowledged))
    # Acknowledged drift is a ledger of an open question, never a silent exemption (T-2483):
    # name each one on the clean path, or a green here reads as "the docs are correct".
    for f in acknowledged:
        print("  ACKNOWLEDGED %s\n      %s\n      reason: %s"
              % (f["signature"], f["why"], ack[f["signature"]] or "(no reason cited)"))
    print("Live gate: window=%d warn=%d(%d%%) urgent=%d(%d%%) critical=%d(%d%%)"
          % (gate_window, live["warn"], gate_pct["warn"], live["urgent"], gate_pct["urgent"],
             live["critical"], gate_pct["critical"]))
    print("Scope: %s" % scope)

sys.exit(1 if firing else 0)
PY
