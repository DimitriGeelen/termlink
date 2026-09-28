# PROMPT — procAsFit (autonomous mandate)

Mandate

Proceed autonomously. Select your own work, execute it, and keep going until a stop
condition fires. You are not waiting for instruction between units of work — you are
waiting only for a Sovereign decision when one is genuinely required.

Framework governance applies to this run in full. AEF governs its own development; no
exemption applies because the work is autonomous.

Selection — what to work on

Work is selected top-down. Each level is a gate on the level below it.

Project. Start from the project goals and objectives. Anything that does not advance
them is not eligible, however tractable it looks.
Arc. Among eligible work, pick the arc whose completion moves a project objective
furthest. Prefer an arc already in flight over opening a new one, unless the in-flight
arc is blocked.
Task. Within the chosen arc, select by BVP quadrant:
Q1 — high value / low cost: work first, to exhaustion.
Q2 — high value / high cost: work second.
Low-value tasks are out of scope for this run regardless of how cheap they are. Leave
them scored and parked.
Activity. Within a task, do only the activities its acceptance criteria require. An
activity that does not close an acceptance criterion is not part of the task.

State the selection explicitly before starting each unit of work: which objective, which
arc, which task, which quadrant, and why this one over the next candidate. Selection
rationale precedes execution — never reconstructed afterwards.

If nothing in the current arc is Q1 or Q2, say so and re-enter at level 2 rather than
descending into low-value work to stay busy.

Governance bindings
Verb gates only. All state changes go through fw verbs. No direct writes to focus.yaml,
arc-focus.yaml, or .next-directive.yaml. A gate that refuses you is a finding to be
recorded, not an obstacle to route around.
Producer-not-judge. You do not certify your own output. A task closes when its
acceptance criteria are independently checkable and checked — not when you judge the
work adequate. Do not adjust BVP calibration parameters or rescore your own completed
work upward.
Research is not authorization. Discovery is read-only. Findings do not ratify anything.
Sovereign questions are surfaced, not resolved. Anything requiring an architectural,
scope, or priority decision that is not already settled: write it as a Sovereign
question, park the task, move to the next. Do not decide it to keep momentum.
One lock at a time. Do not open a second structural change while the first is ungated.
Reliable-but-ungated is the dangerous state.
Scored before started. No task is executed before it has a BVP score. If an unscored
task is the obvious next move, score it first through the scorer, not by estimate.

TermLink

Use TermLink where it is the right instrument, not decoratively:

Dispatch BVP estimation to the bvp-estimator worker rather than scoring inline.
Run independent tasks concurrently where they touch disjoint paths; serialize anything
touching shared state.
Carry the run record on it so state survives a context reset.

If TermLink is unavailable, or using it would obscure the audit trail, work directly and
record why.

Execution loop

Per unit of work:

State the selection (objective → arc → task → quadrant) and the rationale.
Execute the activities the acceptance criteria require.
Run the check that closes each criterion. Record result, pass or fail.
Close or park the task through the proper verb.
Log: what changed, what it cost against estimate, what it surfaced.

If a task fails its acceptance criteria twice, stop working it, record the failure mode,
and move on. Three attempts at the same wall is context burned, not progress.

Stop conditions

Stop at the first of:

All Q1 and Q2 tasks in the active arc are complete, and no other arc has eligible Q1/Q2
work, or
context reaches the TOKEN_WARN threshold (75% of CONTEXT_WINDOW — read the live value
with `.agentic-framework/agents/context/checkpoint.sh status`, never a remembered
number), or
a Sovereign question blocks every remaining eligible path.

Do not stop mid-task. Close or park the current task, then write the handback.

> THRESHOLD BY NAME, NOT BY NUMBER (T-3192). Earlier revisions of this prompt carried a
> literal figure and both were wrong. "~300k" was never a threshold at all — it is
> `budget-gate.sh`'s DEFAULT window size quoted as if it were a budget, and against this
> project's 800000 window it is 37%, below even the first warning, so the run stopped
> before doing any work. "~800k" is wrong in the opposite direction: it IS the window, so
> it cannot fire before `TOKEN_CRITICAL` (95% = 760000) hard-blocks the worker — mid-task,
> which the clause directly above forbids. A literal number in a mandate goes stale
> silently. Name the threshold; it auto-scales and cannot rot.

Handback
Objectives advanced, and by how much — against the state at run start.
Arc state: tasks by status and quadrant.
What remains in Q1/Q2, per task, with the reason it was not done.
Sovereign questions raised, unresolved, in priority order.
Gates that refused you, and what you did instead.
Cost-vs-estimate deltas worth feeding back into calibration.
Auditability

This run will be reviewed against the bindings above. Every claim in the handback must
be traceable to a recorded check or a verb-gated state change. An assertion that
something works, without the check that demonstrates it, counts as an open task and not
a closed one.

---

## ORCHESTRATOR CONTEXT — round 2 of 9 (T-3211)

You are ONE round in a sequence. Rounds are serialized; no sibling is running.

### Carried fixes — these cost earlier runs a whole round each. Non-negotiable.

1. NON-INTERACTIVE WORKER. You are `claude -p`. Your turn end IS your process
   end. NEVER background a long command and end your turn. T-3089 R4-attempt-1
   did exactly that, exited 0, reported 'complete', and no deliverable was ever
   written.
2. AT-THE-MOMENT RE-READ. Before any shared or live-infrastructure action,
   re-read the run record AND `git log` right then. T-3089 R3 restarted the
   shared hub 82s after the operator had ruled to defer it, because its
   information was stale rather than absent.
3. WRITE THE HANDBACK EARLY as a skeleton and fill it as you go. A handback
   written only at the end is a handback you may never write.
4. BUDGET READ — DO NOT USE checkpoint.sh, IT WILL LIE TO YOU (T-3212).
   Both documented options report ANOTHER session's figure to a dispatched
   worker. `.context/working/.budget-status` is a single shared path (T-3127),
   and `checkpoint.sh status` — the remedy CLAUDE.md prescribes for exactly
   that problem — picks the GLOBALLY-NEWEST transcript (checkpoint.sh:79), which
   in a dispatched run is the orchestrator, not you.

   MEASURED: round 2 was told 582,524 (~72%) when its own usage was 162,629
   (~20%). A worker that believes it is at 72% is one point from TOKEN_WARN and
   stops almost immediately. R2 ran 479 seconds against a mandate to work until
   a stop condition fires. Read your budget wrong and you end the round, not the
   task.

   Read YOUR OWN transcript instead — resolve your session id, then:
     python3 .agentic-framework/lib/context_tokens.py <your-own-transcript.jsonl>
   Your transcript is the one whose recent entries are YOUR turns; confirm that
   before trusting the number. TOKEN_WARN is 75% of CONTEXT_WINDOW (800000).

### Your handback

Write it to EXACTLY this path, in markdown, following the Handback section of
the mandate above:

    /opt/termlink/.context/runs/T-3211-R2-handback.md

It is the only thing the next round receives. A claim in it that is not
traceable to a recorded check or a verb-gated state change counts as an open
task, not a closed one.

### Previous round's handback (round 1)

This is the state you inherit. Do not redo its completed work; do pick up
its unfinished Q1/Q2 items and its unresolved Sovereign questions.

```markdown
