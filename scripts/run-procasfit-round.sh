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
VERIFY_ONLY=0
[ "${3:-}" = "--verify" ] && { VERIFY_ONLY=1; PREV=""; }
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

# --verify skips straight to the handback check, for polling a detached round. It must
# NOT delete the handback it is about to inspect — hence the guard on the rm below.
if [ "$VERIFY_ONLY" -eq 0 ]; then
    rm -f "$HANDBACK"
fi

# --verify: skip building and dispatching; go straight to the handback check.
if [ "$VERIFY_ONLY" -eq 1 ]; then
    echo "T-3211 R${ROUND}: verify-only"
else

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
    echo "4. BUDGET READ — DO NOT USE checkpoint.sh, IT WILL LIE TO YOU (T-3212)."
    echo "   Both documented options report ANOTHER session's figure to a dispatched"
    echo "   worker. \`.context/working/.budget-status\` is a single shared path (T-3127),"
    echo "   and \`checkpoint.sh status\` — the remedy CLAUDE.md prescribes for exactly"
    echo "   that problem — picks the GLOBALLY-NEWEST transcript (checkpoint.sh:79), which"
    echo "   in a dispatched run is the orchestrator, not you."
    echo
    echo "   MEASURED: round 2 was told 582,524 (~72%) when its own usage was 162,629"
    echo "   (~20%). A worker that believes it is at 72% is one point from TOKEN_WARN and"
    echo "   stops almost immediately. R2 ran 479 seconds against a mandate to work until"
    echo "   a stop condition fires. Read your budget wrong and you end the round, not the"
    echo "   task."
    echo
    echo "   Read YOUR OWN transcript instead — resolve your session id, then feed the"
    echo "   transcript on STDIN (note the '<' redirect):"
    echo "     python3 .agentic-framework/lib/context_tokens.py < ~/.claude/projects/-opt-termlink/<session-id>.jsonl"
    echo "   Do NOT pass the path as an argument: argv[1] is a session-start TIMESTAMP,"
    echo "   so the argument form reads an empty stdin and prints 0 (T-3211 R5, measured:"
    echo "   argument form 0 vs stdin form 306,256 on the same transcript). A worker that"
    echo "   reads 0 never reaches TOKEN_WARN and runs on into TOKEN_CRITICAL mid-task."
    echo "   Your transcript is the one whose recent entries are YOUR turns; confirm that"
    echo "   before trusting the number. TOKEN_WARN is 75% of CONTEXT_WINDOW (800000)."
    echo
    # Operator directive for this round, if any. Passed verbatim so the worker reads the
    # human's instruction rather than the orchestrator's paraphrase of it.
    if [ -n "${PROCASFIT_OPERATOR_NOTE:-}" ]; then
        echo "### Operator directive for this round"
        echo
        printf '%s\n' "$PROCASFIT_OPERATOR_NOTE"
        echo
    fi
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

# DETACHED LAUNCH IS THE DEFAULT, and it is not a style choice. A shell that sits
# waiting for a 3600s dispatch is a long-lived idle process, and this host's low-memory
# guard reaps exactly those: two of the first three harness runs were killed while
# waiting, and one died BEFORE dispatch returned, so no worker ran at all. The worker
# writes its own handback and never needed the waiting shell — so there should not be
# one. nohup setsid matches how this repo launches every other durable process
# (notify-sidecar-supervisor). Poll for the handback with `--verify` instead.
if [ "${PROCASFIT_DETACH:-1}" = "1" ]; then
    nohup setsid bash -c "
        timeout $((TIMEOUT + 120)) termlink dispatch \
            --count 1 --name 'procasfit-r${ROUND}' \
            --tags 'T-3211,procasfit,round-${ROUND}' \
            --backend background --timeout '$TIMEOUT' --json \
            -- bash -c \"IS_SANDBOX=1 claude -p \\\"\\\$(cat '$PROMPT')\\\" --dangerously-skip-permissions\"
    " > "$LOG" 2>&1 < /dev/null &
    LAUNCH_PID=$!
    echo "T-3211 R${ROUND}: launched DETACHED (pid $LAUNCH_PID, timeout ${TIMEOUT}s)"
    echo "T-3211 R${ROUND}: verify later with: bash $0 $ROUND $TOTAL --verify"
    exit 0
fi

echo "T-3211 R${ROUND}: dispatching in foreground (timeout ${TIMEOUT}s)"

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

fi

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

# A SKELETON IS NOT A HANDBACK. Carried fix 3 tells the worker to write the handback
# early as a skeleton and fill it as it goes — which means an unfilled skeleton is the
# EXPECTED artefact of a round that ended prematurely, and it sails past a size check.
# R2 did exactly this: 1883 bytes, every section "_(pending)_", one ledger row, and the
# byte floor reported it verified. Size was measuring the wrong property.
# `grep -c` already prints 0 on no match (and exits 1); `|| echo 0` would append a SECOND
# 0, making "0\n0" and turning the -gt test into an error that silently reads as false.
PENDING=$(grep -cE '^_\((pending|none yet|filled per unit below|TBD)\)_[[:space:]]*$' "$HANDBACK" 2>/dev/null); PENDING=${PENDING:-0}
FILLED=$(grep -cE '^## ' "$HANDBACK" 2>/dev/null); FILLED=${FILLED:-0}
if [ "$PENDING" -gt 0 ]; then
    echo "T-3211 R${ROUND}: handback is an UNFILLED SKELETON — ${PENDING} of ${FILLED} sections still placeholders" >&2
    echo "  ${BYTES} bytes, which is why a size check passes it. The round ended before" >&2
    echo "  filling it; its work (if any) may be real but is unreported and undisposed." >&2
    echo "  Do not feed this forward as a completed round." >&2
    exit 1
fi
if grep -qiE '^\*\*Status:[[:space:]]*IN PROGRESS' "$HANDBACK" 2>/dev/null; then
    echo "T-3211 R${ROUND}: handback still declares itself IN PROGRESS" >&2
    exit 1
fi
echo "T-3211 R${ROUND}: handback verified by read — ${BYTES} bytes, ${FILLED} sections, 0 placeholders"
exit 0
