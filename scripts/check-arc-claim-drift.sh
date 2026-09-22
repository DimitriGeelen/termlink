#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# T-3066 — a closed arc asserts a capability. Is anything still checking it?
#
# THE INCIDENT THIS COMES FROM
# ----------------------------
# .context/arcs/reliable-comms.yaml (arc-003) closed 2026-07-02 with:
#   decision:          "GO — ... journaled confirmed receipt, no silent loss."
#   headline_mechanic: "...receives a confirmed delivery receipt ... instead of
#                       seeing it silently lost across non-federating hubs."
# Measured 82 days later: the 3-level confirm ladder stops at L2 (stage=read was
# deferred as slice S4 and never built, so "confirmed" means delivered-to-mailbox,
# never read); BOTH wake paths were dark; and a genuine silent-loss path was proven
# by construction. The arc's headline claim is a TESTABLE assertion that nothing
# tested, and it was false for nearly three months with no surface reporting it.
#
# WHY `demo_evidence` IS NOT THE ANSWER
# -------------------------------------
# Both closed arcs carry demo_evidence, and both were still wrong. demo_evidence is
# a markdown narrative written ONCE, at close time, describing a demo that ran that
# day. It cannot fail later. A claim whose only evidence is prose cannot decay
# loudly — it just quietly stops being true. That is the G-069 shipped-but-dark
# shape lifted from the capability layer to the ARC layer, where it is worse,
# because an arc's decision line is what everyone downstream quotes.
#
# WHY THERE WAS NOWHERE TO PUT THE DEFERRED SLICE
# -----------------------------------------------
# Measured across all 8 arcs, the schema key union is:
#   anchor_task blockers bvp_scores closed_at created decision demo_evidence
#   description headline_mechanic id name proposed_scoped_drivers scoped_drivers
#   slug status
# There is NO slices[] field. T-2300 named its successor as slice "S4"; there was
# no row for S4 to occupy, so it survived only as an adjective inside a task that
# then completed. The deferred work was not lost by accident — there was no field
# to lose it from. Filed upstream (framework:pickup offsets 141/142) because the
# arc schema and arc.sh are vendored (G-062).
#
# WHAT THIS CHECKS, AND WHAT IT DOES NOT
# --------------------------------------
# Fires on a CLOSED arc whose capability claim is not bound to an executable prover
# via a `prover:` field. Where a prover IS bound it RUNS it and fires on non-zero,
# so the binding cannot itself go stale (T-2683: a guard nothing executes).
#
# SCOPE — read a green narrowly (T-2680). It asks one question: is the claim bound
# to something runnable, and does that something still pass? It does NOT judge
# whether the prover is ADEQUATE to the claim, and it says nothing about arcs that
# are still in-progress. A bound-and-passing arc means someone can re-check the
# claim, not that the claim is true.
#
# Exit: 0 every closed arc's claim is bound and passing
#       1 a closed arc has an unbound claim, or its bound prover failed
#       2 tooling — FAIL-CLOSED: no arcs dir, no python3, no yaml, or zero arcs
#         parsed all exit 2, never a vacuous clean
set -u

ARCS_DIR="${ARC_CLAIM_ARCS_DIR:-.context/arcs}"
ALLOWLIST="${ARC_CLAIM_ALLOWLIST:-.context/checks/arc-claim-allowlist}"
RUN_PROVERS=1
JSON=0
QUIET=0
TIMEOUT="${ARC_CLAIM_PROVER_TIMEOUT:-300}"

usage() {
    cat <<'USAGE'
Usage: check-arc-claim-drift.sh [options]

Fires when a CLOSED arc's capability claim is not bound to an executable prover,
or when a bound prover fails. demo_evidence (a markdown narrative) does not count
as a binding — it cannot fail later.

Options:
  --arcs-dir DIR     arcs directory (default .context/arcs)
  --allowlist FILE   acknowledged arcs, one `<arc-id>  # reason` per line
  --no-run           do not execute bound provers, only check the binding exists
  --timeout SECS     per-prover timeout (default 300)
  --json             machine-readable envelope
  --quiet            print only when something fires
  --help             this text

Exit: 0 clean · 1 unbound claim or failing prover · 2 tooling (fail-closed)
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --arcs-dir)  ARCS_DIR="${2:-}"; shift 2 ;;
        --allowlist) ALLOWLIST="${2:-}"; shift 2 ;;
        --no-run)    RUN_PROVERS=0; shift ;;
        --timeout)   TIMEOUT="${2:-}"; shift 2 ;;
        --json)      JSON=1; shift ;;
        --quiet)     QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        --help|-h)   usage; exit 0 ;;
        *) echo "check-arc-claim-drift: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

command -v python3 >/dev/null 2>&1 || { echo "check-arc-claim-drift: python3 not found" >&2; exit 2; }
[ -d "$ARCS_DIR" ] || { echo "check-arc-claim-drift: arcs dir not found: $ARCS_DIR" >&2; exit 2; }

# Python does the parse + classification; bash runs any bound provers, because a
# prover is an arbitrary shell command and belongs outside the parser.
PLAN="$(ARCS_DIR="$ARCS_DIR" ALLOWLIST="$ALLOWLIST" python3 - <<'PY'
import os, sys, glob, json
try:
    import yaml
except ImportError:
    print("ERR\tPyYAML not installed", file=sys.stderr); sys.exit(2)

arcs_dir = os.environ["ARCS_DIR"]
allow_path = os.environ["ALLOWLIST"]

allow = {}
if os.path.exists(allow_path):
    try:
        for line in open(allow_path, encoding="utf-8"):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            arc, _, reason = line.partition("#")
            allow[arc.strip()] = reason.strip()
    except OSError:
        print("ERR\tallowlist unreadable", file=sys.stderr); sys.exit(2)

files = sorted(glob.glob(os.path.join(arcs_dir, "*.yaml")))
if not files:
    # Zero arcs is a tooling error, never a clean bill: a parse that silently
    # stopped matching would otherwise report green forever (T-2747).
    print("ERR\tno arc files found", file=sys.stderr); sys.exit(2)

parsed = 0
rows = []
for f in files:
    try:
        d = yaml.safe_load(open(f, encoding="utf-8")) or {}
    except Exception as e:
        print(f"ERR\tunparseable arc {os.path.basename(f)}: {e}", file=sys.stderr)
        sys.exit(2)
    parsed += 1
    arc_id = str(d.get("id") or os.path.basename(f))
    status = str(d.get("status") or "")
    if status != "closed":
        rows.append(("SKIP", arc_id, "", f"status={status or 'unset'}"))
        continue
    claim = (d.get("decision") or d.get("headline_mechanic") or "").strip()
    prover = (d.get("prover") or "").strip() if isinstance(d.get("prover"), str) else ""
    demo = (d.get("demo_evidence") or "").strip()
    if arc_id in allow:
        rows.append(("ALLOW", arc_id, prover, allow[arc_id] or "acknowledged"))
    elif prover:
        rows.append(("RUN", arc_id, prover, claim[:90]))
    else:
        why = "no prover bound"
        if demo:
            why += f"; demo_evidence={demo} is prose written once at close time, it cannot fail later"
        rows.append(("UNBOUND", arc_id, "", why))

if parsed == 0:
    print("ERR\tzero arcs parsed", file=sys.stderr); sys.exit(2)

# Empty fields are emitted as "-", never as "". Tab is an IFS *whitespace*
# character, so bash's `read` collapses a run of tabs into ONE delimiter and every
# field after an empty one silently shifts left. That is how the first run of this
# check printed "UNBOUND" with a blank reason: the explanation had shifted into the
# prover column. Silent field-shifting, same family as the other bugs this task
# found.
rows = [tuple((str(x) if str(x) != "" else "-") for x in r) for r in rows]
print(json.dumps({"parsed": parsed, "rows": rows}))
PY
)" || exit 2

python3 -c "import json,sys; json.loads(sys.argv[1])" "$PLAN" 2>/dev/null || {
    echo "check-arc-claim-drift: could not build plan" >&2; exit 2; }

PARSED="$(printf '%s' "$PLAN" | python3 -c 'import json,sys; print(json.load(sys.stdin)["parsed"])')"
FIRING=0; CLOSED=0; ALLOWED=0; BOUND_OK=0
FIRE_LINES=""

while IFS=$'\t' read -r kind arc prover detail; do
    [ -n "$kind" ] || continue
    [ "$prover" = "-" ] && prover=""
    [ "$detail" = "-" ] && detail=""
    case "$kind" in
        SKIP)    continue ;;
        ALLOW)   CLOSED=$((CLOSED+1)); ALLOWED=$((ALLOWED+1))
                 { [ "$QUIET" -eq 1 ] || [ "$JSON" -eq 1 ]; } || printf '  %-28s ACKNOWLEDGED  %s\n' "$arc" "$detail" ;;
        UNBOUND) CLOSED=$((CLOSED+1)); FIRING=$((FIRING+1))
                 FIRE_LINES="${FIRE_LINES}${arc}: ${detail}"$'\n'
                 [ "$JSON" -eq 1 ] || printf '  %-28s UNBOUND       %s\n' "$arc" "$detail" ;;
        RUN)     CLOSED=$((CLOSED+1))
                 if [ "$RUN_PROVERS" -eq 0 ]; then
                     BOUND_OK=$((BOUND_OK+1))
                     { [ "$QUIET" -eq 1 ] || [ "$JSON" -eq 1 ]; } || printf '  %-28s BOUND         %s (not run)\n' "$arc" "$prover"
                 else
                     if timeout "$TIMEOUT" bash -c "$prover" >/dev/null 2>&1; then
                         BOUND_OK=$((BOUND_OK+1))
                         { [ "$QUIET" -eq 1 ] || [ "$JSON" -eq 1 ]; } || printf '  %-28s VERIFIED      %s\n' "$arc" "$prover"
                     else
                         FIRING=$((FIRING+1))
                         FIRE_LINES="${FIRE_LINES}${arc}: bound prover FAILED: ${prover}"$'\n'
                         [ "$JSON" -eq 1 ] || printf '  %-28s CLAIM-FAILED  prover exited non-zero: %s\n' "$arc" "$prover"
                     fi
                 fi ;;
    esac
done < <(printf '%s' "$PLAN" | python3 -c '
import json,sys
d=json.load(sys.stdin)
for r in d["rows"]:
    print("\t".join(str(x) for x in r))
')

if [ "$JSON" -eq 1 ]; then
    printf '{"ok":%s,"arcs_parsed":%s,"closed_arcs":%s,"firing":%s,"acknowledged":%s,"verified":%s,"scope":"detects whether a CLOSED arc claim is bound to a runnable prover and whether it still passes; does NOT judge prover adequacy, and says nothing about in-progress arcs"}\n' \
        "$([ "$FIRING" -eq 0 ] && echo true || echo false)" "$PARSED" "$CLOSED" "$FIRING" "$ALLOWED" "$BOUND_OK"
elif [ "$QUIET" -eq 0 ] || [ "$FIRING" -gt 0 ]; then
    echo
    echo "arcs parsed: $PARSED   closed: $CLOSED   verified: $BOUND_OK   acknowledged: $ALLOWED   FIRING: $FIRING"
    echo "SCOPE: detects whether a closed arc's claim is bound to a runnable prover"
    echo "       and whether it still passes. It does NOT judge whether the prover is"
    echo "       ADEQUATE to the claim, and it says nothing about in-progress arcs."
    if [ "$FIRING" -gt 0 ]; then
        echo
        echo "Remediation: add a 'prover:' field to the arc naming a command that re-checks"
        echo "its capability claim, e.g. for arc-003 (reliable-comms):"
        echo "    prover: \"bash scripts/notify-rail-e2e.sh --experiment e4\""
        echo "or acknowledge in $ALLOWLIST with a cited reason."
    fi
fi

[ "$FIRING" -eq 0 ] || exit 1
exit 0
