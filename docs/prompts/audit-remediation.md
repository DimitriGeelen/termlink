# PROMPT — Audit + housekeeping remediation (3 cycles)

Mandate

Run the audit and housekeeping verbs, convert every finding into a governed task in a
remediation arc, score the arc with the BVP scorer, and drive the high-value tasks to
completion. Three full cycles per run. Framework governance applies to your own work —
AEF dogfoods itself here.

Binding constraints
One finding, one task. Every individual FAIL and every individual WARNING becomes its own
separate task in the remediation arc. Do not fold findings into the acceptance criteria of
an existing task. Do not batch several findings into one task because they share a root
cause — if they share a root cause, say so in each task's context and link them, but keep
them as separate task records.
Verb gates only. All state changes go through fw verbs. Never write focus.yaml,
arc-focus.yaml, or .next-directive.yaml directly at the filesystem level, and never route
around a gate that refuses you. A refused gate is a finding, not an obstacle.
Producer-not-judge. You do not certify your own remediation. Verification of a task is the
re-run of the audit in the next cycle, not your own assertion that it is fixed. Do not
score your own work upward, and do not adjust BVP calibration parameters.
No scope drift. If a fix requires an architectural decision, stop, write it as a Sovereign
question, and move to the next task. Do not decide it yourself.
Confirm the verb surface before using it. Run fw --help (and subcommand help) first and use
the verbs as they actually exist in this build. Do not invent verbs.

Step 0 — Orient (read-only)
fw --help, plus help for the audit, housekeeping, arc, task and bvp subcommands.
List existing arcs and their status. Identify whether a remediation arc already exists.
Note the current context budget. Everything below is bounded by it.

Output: a short orientation note. Nothing is written in this step.

Step 1 — Audit and housekeeping

Run the audit verb. Run the housekeeping verb. Capture the complete output of both,
including passes — you need the pass set as the baseline for detecting regressions in later
cycles.

Produce a findings table with, per finding: stable ID, source (audit | housekeeping),
severity (FAIL | WARN), the rule or check that fired, the artifact or path implicated, and
the observed-vs-expected delta. Do not editorialize severity — carry it through as the tool
reported it.

Step 2 — Remediation arc

If a remediation arc exists, use it. If none exists, create one through the normal arc
creation path, following the inception workflow — including whatever Sovereign gates that
workflow requires. If a gate blocks arc creation, stop and surface it; do not create the arc
by another route.

Step 3 — Task creation

For each finding in the table, create one task in the remediation arc. Each task carries:

The finding ID and the verbatim tool output that produced it.
A statement of the defect in structural terms — what invariant is not held.
Acceptance criteria written as falsifiable conditions, at minimum: the originating check
reports PASS on re-run, plus any additional condition that prevents the defect from
recurring silently.
Its root-cause link to sibling tasks, where one exists.

Zero findings must remain outside the arc at the end of this step. Reconcile the count
explicitly: findings in = tasks out.

Step 4 — Score

Score every task in the remediation arc with the BVP scorer. Use the arc's drivers if it has
arc-scoped drivers; otherwise the global free drivers apply. If the arc plainly needs a
driver that does not exist, propose it — do not create it silently, and do not score against
a driver you invented mid-run.

Rationale precedes score. Never the reverse.

Step 5 — Prioritize

Place each scored task into a value/cost quadrant:

                Low cost                  High cost
High value      Q1 — work first           Q2 — work second
Low value       out of scope this run     out of scope this run

Work Q1 to exhaustion, then Q2. Low-value tasks stay in the arc, scored and parked. Do not
pick up a low-value task because it is quick.

Step 6 — Execute

Work the queue in order, one task at a time. Per task: implement, run the originating check,
record the result. If a task fails its acceptance criteria twice, stop working it, record the
failure mode on the task, and move on — three attempts at the same wall is context burned,
not progress.

Step 7 — Cycle

Repeat Steps 1 through 6 three times in this run. Each cycle:

Re-runs audit and housekeeping in full.
Creates separate tasks for any new findings, including regressions.
Rescores the arc, because cost estimates shift once the first cycle has exposed real effort.
Reports at cycle close: findings by severity, tasks created, tasks completed, tasks remaining
by quadrant, regressions, and the delta against the previous cycle's baseline.

TermLink

Use TermLink where it is the right instrument, not decoratively:

Dispatch BVP estimation to the bvp-estimator worker rather than scoring inline.
Run independent Q1 tasks concurrently where they touch disjoint paths; serialize anything
that touches shared state.
Use it for finding capture across the run so the record survives a context reset.

If TermLink is unavailable or its use would obscure the audit trail, do the work directly and
note why.

Stop conditions

Stop and hand back when the first of these is true:

All Q1 and Q2 tasks are complete and the audit re-runs clean for those findings, or
three cycles are done, or
context reaches ~300k.

Whichever fires, do not stop mid-task. Close the task you are on, then write the handback.
