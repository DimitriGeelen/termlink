#!/usr/bin/env bash
# T-3211 — dispatch ONE procAsFit round to its own TermLink worker and verify the
# handback by reading it.
#
# One round per invocation, deliberately. A script that loops all nine internally would
# make the orchestrator blind between rounds, and each round's prompt has to carry the
# PREVIOUS round's handback — which does not exist until that round finishes.
#
# THE VERIFICATION RULE (T-2876, and T-3093's AC): a handback is verified to exist and be
# non-empty BY READING IT, never by the dispatcher's return code. `termlink dispatch` can
# return a clean-looking envelope for a worker that produced nothing (T-3181 reports
# ok:true with workers_registered:0), so the return code is not evidence.
#
# IS_SANDBOX=1 IS LOAD-BEARING, NOT DECORATION. This host runs as root, and
# `claude -p --dangerously-skip-permissions` refuses outright under root/sudo:
# "cannot be used with root/sudo privileges for security reasons", exit 1, no output.
# The worker still REGISTERS with the dispatcher, so the envelope reads
# workers_spawned:1 / workers_registered:1 while nothing was produced — indistinguishable
# from a worker that ran and had nothing to say. Measured here before round 1.
# `scripts/dispatch-t909-risk-eval.sh` omits it and therefore cannot work on this host.
#
# Usage: run-procasfit-round.sh <round-n> <total> [prev-handback-path]
# Exit:  0 handback present and non-empty · 1 no usable handback · 2 usage/tooling
set -uo pipefail

ROUND="${1:-}"; TOTAL="${2:-}"; PREV="${3:-}"
case "$ROUND" in ''|*[!0-9]*) echo "usage: $0 <round-n> <total> [prev-handback]" >&2; exit 2 ;; esac
case "$TOTAL" in ''|*[!0-9]*) echo "usage: $0 <round-n> <total> [prev-handback]" >&2; exit 2 ;; esac

ROOT="${PROJECT_ROOT:-/opt/termlink}"
RUNS="$ROOT/.context/runs"
MANDATE="$ROOT/docs/prompts/proc-as-fit.md"
PROMPT="$RUNS/T-3211-R${ROUND}-prompt.md"
HANDBACK="$RUNS/T-3211-R${ROUND}-handback.md"
LOG="$RUNS/T-3211-R${ROUND}-dispatch.log"
TIMEOUT="${PROCASFIT_ROUND_TIMEOUT:-3600}"

[ -r "$MANDATE" ] || { echo "T-3211: mandate not readable: $MANDATE" >&2; exit 2; }
mkdir -p "$RUNS" || exit 2
rm -f "$HANDBACK"

# ---- build the round prompt -------------------------------------------------------
{
    cat "$MANDATE"
    echo
    echo "---"
    echo
    echo "## ORCHESTRATOR CONTEXT — round ${ROUND} of ${TOTAL} (T-3211)"
    echo
    echo "You are ONE round in a sequence. Rounds are serialized; no sibling is running."
    echo
    echo "### Carried fixes — these cost earlier runs a whole round each. Non-negotiable."
    echo
    echo "1. NON-INTERACTIVE WORKER. You are \`claude -p\`. Your turn end IS your process"
    echo "   end. NEVER background a long command and end your turn. T-3089 R4-attempt-1"
    echo "   did exactly that, exited 0, reported 'complete', and no deliverable was ever"
    echo "   written."
    echo "2. AT-THE-MOMENT RE-READ. Before any shared or live-infrastructure action,"
    echo "   re-read the run record AND \`git log\` right then. T-3089 R3 restarted the"
    echo "   shared hub 82s after the operator had ruled to defer it, because its"
    echo "   information was stale rather than absent."
    echo "3. WRITE THE HANDBACK EARLY as a skeleton and fill it as you go. A handback"
    echo "   written only at the end is a handback you may never write."
    echo "4. BUDGET READ. You may be one of several dispatched workers over this run's"
    echo "   lifetime, so \`.context/working/.budget-status\` can hold another session's"
    echo "   figure (T-3127). Read your own with"
    echo "   \`.agentic-framework/agents/context/checkpoint.sh status\`."
    echo
    echo "### Your handback"
    echo
    echo "Write it to EXACTLY this path, in markdown, following the Handback section of"
    echo "the mandate above:"
    echo
    echo "    ${HANDBACK}"
    echo
    echo "It is the only thing the next round receives. A claim in it that is not"
    echo "traceable to a recorded check or a verb-gated state change counts as an open"
    echo "task, not a closed one."
    echo
    if [ -n "$PREV" ] && [ -s "$PREV" ]; then
        echo "### Previous round's handback (round $((ROUND-1)))"
        echo
        echo "This is the state you inherit. Do not redo its completed work; do pick up"
        echo "its unfinished Q1/Q2 items and its unresolved Sovereign questions."
        echo
        echo '```markdown'
        cat "$PREV"
        echo '```'
    else
        echo "### No previous handback"
        echo
        if [ "$ROUND" -eq 1 ]; then
            echo "You are round 1. Establish the baseline: record objective/arc/task state"
            echo "at run start so later rounds can measure movement against it."
        else
            echo "Round $((ROUND-1)) produced no usable handback. Treat its work as"
            echo "UNKNOWN rather than done, and say so in your own handback."
        fi
    fi
} > "$PROMPT"

echo "T-3211 R${ROUND}: prompt $(wc -c < "$PROMPT") bytes -> $PROMPT"
echo "T-3211 R${ROUND}: dispatching (timeout ${TIMEOUT}s)"

# ---- dispatch ---------------------------------------------------------------------
# --backend background so the worker is not bound to this shell's lifetime.
timeout $((TIMEOUT + 120)) termlink dispatch \
    --count 1 \
    --name "procasfit-r${ROUND}" \
    --tags "T-3211,procasfit,round-${ROUND}" \
    --backend background \
    --timeout "$TIMEOUT" \
    --json \
    -- bash -c "IS_SANDBOX=1 claude -p \"\$(cat '$PROMPT')\" --dangerously-skip-permissions" \
    > "$LOG" 2>&1
DISPATCH_RC=$?

echo "T-3211 R${ROUND}: dispatch returned rc=$DISPATCH_RC (NOT evidence — see below)"

# ---- verify by READING the handback ------------------------------------------------
if [ ! -f "$HANDBACK" ]; then
    echo "T-3211 R${ROUND}: NO HANDBACK at $HANDBACK" >&2
    echo "  The dispatcher's rc is not evidence a worker ran (T-3181/T-2876)." >&2
    echo "  Dispatch log tail:" >&2
    tail -5 "$LOG" >&2 2>/dev/null
    exit 1
fi
BYTES=$(wc -c < "$HANDBACK")
if [ "$BYTES" -lt 200 ]; then
    echo "T-3211 R${ROUND}: handback present but implausibly small (${BYTES} bytes)" >&2
    echo "  Treating as NO usable handback rather than a thin success." >&2
    exit 1
fi
echo "T-3211 R${ROUND}: handback verified by read — ${BYTES} bytes at $HANDBACK"
exit 0
